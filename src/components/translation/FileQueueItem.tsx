import { Button, Chip, Select, Label, ListBox, Spinner } from '@heroui/react';
import { QueueFile, FileStatus } from '../../types';
import { useTranslationStore } from '../../stores/translationStore';

interface Props {
  file: QueueFile;
  index: number;
}

const STATUS_CONFIG: Record<FileStatus, { label: string; color: 'default' | 'accent' | 'success' | 'warning' | 'danger' }> = {
  pending: { label: 'Pendente', color: 'default' },
  extracting: { label: 'Extraindo', color: 'accent' },
  translating: { label: 'Traduzindo', color: 'accent' },
  detecting_language: { label: 'Detectando Idioma', color: 'accent' },
  saving: { label: 'Salvando', color: 'accent' },
  muxing: { label: 'Muxando', color: 'accent' },
  paused: { label: 'Pausado', color: 'warning' },
  completed: { label: 'Concluído', color: 'success' },
  error: { label: 'Erro', color: 'danger' },
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

  const handleTrackChange = (key: React.Key | null) => {
    if (key !== null) {
      setSelectedTrack(file.id, Number(key));
    }
  };

  return (
    <div className="flex items-center gap-3 p-3 bg-default-100 rounded-lg">
      <div className="w-6 text-center text-default-400 text-sm">
        {index + 1}
      </div>

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium truncate">{file.name}</span>
          <Chip size="sm" color={config.color} variant="soft">
            {config.label}
          </Chip>
          {isVideo && <Chip size="sm" variant="soft">Video</Chip>}
        </div>

        {isVideo && file.isLoadingTracks && (
          <div className="flex items-center gap-2 mt-2 text-xs text-default-500">
            <Spinner size="sm" />
            <span>Detectando faixas...</span>
          </div>
        )}

        {isVideo && noTracks && (
          <p className="text-xs text-warning mt-2">
            Nenhuma faixa de legenda encontrada
          </p>
        )}

        {isVideo && hasTracks && (
          <div className="mt-2">
            <Select
              aria-label="Selecionar faixa de legenda"
              selectedKey={String(file.selectedTrackIndex ?? 0)}
              onSelectionChange={handleTrackChange}
              className="w-full max-w-xs"
              isDisabled={isProcessing}
            >
              <Label className="sr-only">Faixa de legenda</Label>
              <Select.Trigger>
                <Select.Value />
                <Select.Indicator />
              </Select.Trigger>
              <Select.Popover>
                <ListBox>
                  {file.subtitleTracks!.map((track) => (
                    <ListBox.Item id={String(track.index)} key={track.index} textValue={getTrackLabel(track)}>
                      {getTrackLabel(track)}
                      <ListBox.ItemIndicator />
                    </ListBox.Item>
                  ))}
                </ListBox>
              </Select.Popover>
            </Select>
          </div>
        )}

        {isProcessing && (
          <div className="mt-2">
            <div className="flex items-center justify-between text-xs text-default-500 mb-1">
              <span>
                {file.status === 'translating' && file.totalLines > 0
                  ? `${file.translatedLines}/${file.totalLines} linhas`
                  : config.label}
              </span>
              <span className="font-medium">{Math.round(file.progress)}%</span>
            </div>
            <div className="h-2 bg-default-200 rounded-full overflow-hidden">
              <div
                className="h-full bg-primary transition-all duration-300 ease-out"
                style={{ width: `${file.progress}%` }}
              />
            </div>
          </div>
        )}

        {file.error && (
          <p className="text-xs text-danger mt-1 truncate">
            {file.error}
          </p>
        )}
      </div>

      <Button
        size="sm"
        variant="danger-soft"
        isIconOnly
        onPress={() => removeFile(file.id)}
        isDisabled={isProcessing}
      >
        ✕
      </Button>
    </div>
  );
}
