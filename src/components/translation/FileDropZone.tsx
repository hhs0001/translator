import { useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '@/components/ui/button';
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { useFileHandler } from '../../hooks/useFileHandler';

export function FileDropZone() {
  const { t } = useTranslation();
  const [isDragging, setIsDragging] = useState(false);
  const { handleFileDrop, openFilePicker } = useFileHandler();

  // Listen for Tauri file drop events
  useEffect(() => {
    const unlisten = listen<{ paths: string[] }>('tauri://drop', (event) => {
      handleFileDrop(event.payload.paths);
    });

    const unlistenHover = listen('tauri://drop-hover', () => {
      setIsDragging(true);
    });

    const unlistenCancel = listen('tauri://drop-cancelled', () => {
      setIsDragging(false);
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenHover.then((fn) => fn());
      unlistenCancel.then((fn) => fn());
    };
  }, [handleFileDrop]);

  // Also handle HTML5 drag events for visual feedback
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback(() => {
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    // Tauri handles the actual file paths
  }, []);

  return (
    <div
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      className={`
        border-2 border-dashed rounded-xl p-8 text-center transition-all duration-200
        ${isDragging
          ? 'border-primary bg-primary/10 scale-[1.02]'
          : 'border-border hover:border-primary/50'
        }
      `}
    >
      <div className="space-y-4">
        <div className="text-4xl">📁</div>
        <div>
          <p className="text-lg font-medium">
            {t('translation.dropzone.title')}
          </p>
          <p className="text-sm text-muted-foreground mt-1">
            {t('translation.dropzone.subtitle')}
          </p>
        </div>
        <div className="flex items-center gap-4 justify-center">
          <div className="h-px bg-border flex-1 max-w-16" />
          <span className="text-muted-foreground">{t('common.or')}</span>
          <div className="h-px bg-border flex-1 max-w-16" />
        </div>
        <Button variant="ghost" onClick={openFilePicker}>
          {t('translation.dropzone.selectFiles')}
        </Button>
      </div>
    </div>
  );
}
