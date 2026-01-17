import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Spinner } from '@/components/ui/spinner';
import { QueueFile, FileStatus } from '../../types';
import { useTranslationStore } from '../../stores/translationStore';

interface Props {
  file: QueueFile;
  index: number;
}

const STATUS_CONFIG: Record<FileStatus, { label: string; className: string }> = {
  pending: { label: 'Pendente', className: 'bg-muted text-muted-foreground' },
  extracting: { label: 'Extraindo', className: 'bg-sky-500/15 text-sky-700 dark:text-sky-300' },
  translating: { label: 'Traduzindo', className: 'bg-sky-500/15 text-sky-700 dark:text-sky-300' },
  detecting_language: { label: 'Detectando Idioma', className: 'bg-sky-500/15 text-sky-700 dark:text-sky-300' },
  saving: { label: 'Salvando', className: 'bg-sky-500/15 text-sky-700 dark:text-sky-300' },
  muxing: { label: 'Muxando', className: 'bg-sky-500/15 text-sky-700 dark:text-sky-300' },
  paused: { label: 'Pausado', className: 'bg-amber-500/15 text-amber-700 dark:text-amber-300' },
  completed: { label: 'Concluído', className: 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300' },
  error: { label: 'Erro', className: 'bg-destructive/15 text-destructive' },
};

function getTrackLabel(track: { index: number; codec: string; language?: string; title?: string }) {
  const parts = [`Faixa ${track.index + 1}`];
  if (track.language) parts.push(`(${track.language})`);
  if (track.title) parts.push(`- ${track.title}`);
  parts.push(`[${track.codec}]`);
  return parts.join(' ');
}

export function FileQueueItem({ file, index }: Props) {
  const { removeFile, setSelectedTrack } = useTranslationStore();
  const config = STATUS_CONFIG[file.status];

  const isProcessing = ['extracting', 'translating', 'detecting_language', 'saving', 'muxing'].includes(file.status);
  const isVideo = file.type === 'video';
  const hasTracks = file.subtitleTracks && file.subtitleTracks.length > 0;
  const noTracks = file.subtitleTracks && file.subtitleTracks.length === 0 && !file.isLoadingTracks;

  const handleTrackChange = (value: string) => {
    if (value !== '') {
      setSelectedTrack(file.id, Number(value));
    }
  };

  return (
    <div className="flex items-center gap-3 p-3 bg-muted/50 rounded-lg">
      <div className="w-6 text-center text-muted-foreground text-sm">
        {index + 1}
      </div>

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium truncate">{file.name}</span>
          <Badge variant="outline" className={`border-transparent ${config.className}`}>
            {config.label}
          </Badge>
          {isVideo && (
            <Badge variant="outline" className="border-transparent bg-muted text-muted-foreground">
              Video
            </Badge>
          )}
        </div>

        {isVideo && file.isLoadingTracks && (
          <div className="flex items-center gap-2 mt-2 text-xs text-muted-foreground">
            <Spinner className="size-3" />
            <span>Detectando faixas...</span>
          </div>
        )}

        {isVideo && noTracks && (
          <p className="text-xs text-amber-600 mt-2">
            Nenhuma faixa de legenda encontrada
          </p>
        )}

        {isVideo && hasTracks && (
          <div className="mt-2">
            <Select
              value={String(file.selectedTrackIndex ?? 0)}
              onValueChange={handleTrackChange}
              disabled={isProcessing}
            >
              <Label className="sr-only">Faixa de legenda</Label>
              <SelectTrigger className="w-full max-w-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {file.subtitleTracks!.map((track) => (
                  <SelectItem key={track.index} value={String(track.index)}>
                    {getTrackLabel(track)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}

        {isProcessing && (
          <div className="mt-2">
            <div className="flex items-center justify-between text-xs text-muted-foreground mb-1">
              <span>
                {file.status === 'translating' && file.totalLines > 0
                  ? `${file.translatedLines}/${file.totalLines} linhas`
                  : config.label}
              </span>
              <span className="font-medium">{Math.round(file.progress)}%</span>
            </div>
            <div className="h-2 bg-muted rounded-full overflow-hidden">
              <div
                className="h-full bg-primary transition-all duration-300 ease-out"
                style={{ width: `${file.progress}%` }}
              />
            </div>
          </div>
        )}

        {file.error && (
          <p className="text-xs text-destructive mt-1 truncate">
            {file.error}
          </p>
        )}
      </div>

      <Button
        size="icon-sm"
        variant="ghost"
        className="text-destructive hover:text-destructive"
        onClick={() => removeFile(file.id)}
        disabled={isProcessing}
        aria-label="Remover arquivo"
      >
        ✕
      </Button>
    </div>
  );
}
