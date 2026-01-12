import { Button, Chip } from '@heroui/react';
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
  paused: { label: 'Pausado', color: 'warning' },
  completed: { label: 'Concluído', color: 'success' },
  error: { label: 'Erro', color: 'danger' },
};

export function FileQueueItem({ file, index }: Props) {
  const { removeFile } = useTranslationStore();
  const config = STATUS_CONFIG[file.status];

  const isProcessing = file.status === 'extracting' || file.status === 'translating';

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
        </div>

        {isProcessing && (
          <div className="mt-2 h-2 bg-default-200 rounded-full overflow-hidden">
            <div
              className="h-full bg-primary transition-all duration-300"
              style={{ width: `${file.progress}%` }}
            />
          </div>
        )}

        {file.status === 'translating' && file.totalLines > 0 && (
          <p className="text-xs text-default-400 mt-1">
            {file.translatedLines}/{file.totalLines} linhas
          </p>
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
