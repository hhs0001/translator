import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { FileQueueItem } from './FileQueueItem';
import { useTranslationStore } from '../../stores/translationStore';

interface Props {
  maxVisible?: number;
}

export function FileQueue({ maxVisible }: Props) {
  const { queue, clearQueue, setAllVideoTracks } = useTranslationStore();

  const visibleQueue = maxVisible ? queue.slice(0, maxVisible) : queue;
  const hiddenCount = maxVisible ? Math.max(0, queue.length - maxVisible) : 0;

  // Encontrar vídeos com faixas disponíveis
  const videosWithTracks = queue.filter(f => f.type === 'video' && f.subtitleTracks && f.subtitleTracks.length > 0);
  const hasMultipleVideos = videosWithTracks.length > 1;
  
  // Pegar o máximo de faixas disponíveis entre todos os vídeos
  const maxTracks = Math.max(...videosWithTracks.map(f => f.subtitleTracks?.length ?? 0), 0);
  const trackOptions = Array.from({ length: maxTracks }, (_, i) => i);

  const handleQuickSelect = (value: string) => {
    if (value !== '') {
      setAllVideoTracks(Number(value));
    }
  };

  if (queue.length === 0) {
    return null;
  }

  return (
    <Card className="p-4">
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold">
          Fila ({queue.length} {queue.length === 1 ? 'arquivo' : 'arquivos'})
        </h3>
        <div className="flex items-center gap-2">
          {hasMultipleVideos && maxTracks > 0 && (
            <Select onValueChange={handleQuickSelect}>
              <Label className="sr-only">Faixa para todos os vídeos</Label>
              <SelectTrigger className="w-48">
                <SelectValue placeholder="Aplicar faixa em todos" />
              </SelectTrigger>
              <SelectContent>
                {trackOptions.map((trackIndex) => (
                  <SelectItem key={trackIndex} value={String(trackIndex)}>
                    Faixa {trackIndex + 1} em todos
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
          <Button
            size="sm"
            variant="outline"
            className="border-destructive/30 text-destructive hover:bg-destructive/10"
            onClick={clearQueue}
          >
            Limpar
          </Button>
        </div>
      </div>

      <div className="space-y-2">
        {visibleQueue.map((file, index) => (
          <FileQueueItem key={file.id} file={file} index={index} />
        ))}

        {hiddenCount > 0 && (
          <p className="text-center text-sm text-muted-foreground py-2">
            +{hiddenCount} arquivo(s) na fila
          </p>
        )}
      </div>
    </Card>
  );
}
