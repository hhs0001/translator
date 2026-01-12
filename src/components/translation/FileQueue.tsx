import { Button, Card, Select, Label, ListBox } from '@heroui/react';
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

  const handleQuickSelect = (key: React.Key | null) => {
    if (key !== null) {
      setAllVideoTracks(Number(key));
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
            <Select
              aria-label="Aplicar faixa em todos"
              placeholder="Aplicar faixa em todos"
              onSelectionChange={handleQuickSelect}
              className="w-48"
            >
              <Label className="sr-only">Faixa para todos os vídeos</Label>
              <Select.Trigger>
                <Select.Value />
                <Select.Indicator />
              </Select.Trigger>
              <Select.Popover>
                <ListBox>
                  {trackOptions.map((trackIndex) => (
                    <ListBox.Item id={String(trackIndex)} key={trackIndex} textValue={`Faixa ${trackIndex + 1}`}>
                      Faixa {trackIndex + 1} em todos
                      <ListBox.ItemIndicator />
                    </ListBox.Item>
                  ))}
                </ListBox>
              </Select.Popover>
            </Select>
          )}
          <Button size="sm" variant="danger-soft" onPress={clearQueue}>
            Limpar
          </Button>
        </div>
      </div>

      <div className="space-y-2">
        {visibleQueue.map((file, index) => (
          <FileQueueItem key={file.id} file={file} index={index} />
        ))}

        {hiddenCount > 0 && (
          <p className="text-center text-sm text-default-500 py-2">
            +{hiddenCount} arquivo(s) na fila
          </p>
        )}
      </div>
    </Card>
  );
}
