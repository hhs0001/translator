import { Button, Card } from '@heroui/react';
import { FileQueueItem } from './FileQueueItem';
import { useTranslationStore } from '../../stores/translationStore';

interface Props {
  maxVisible?: number;
}

export function FileQueue({ maxVisible }: Props) {
  const { queue, clearQueue } = useTranslationStore();

  const visibleQueue = maxVisible ? queue.slice(0, maxVisible) : queue;
  const hiddenCount = maxVisible ? Math.max(0, queue.length - maxVisible) : 0;

  if (queue.length === 0) {
    return null;
  }

  return (
    <Card className="p-4">
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold">
          Fila ({queue.length} {queue.length === 1 ? 'arquivo' : 'arquivos'})
        </h3>
        <Button size="sm" variant="danger-soft" onPress={clearQueue}>
          Limpar
        </Button>
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
