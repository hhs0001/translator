import { useTranslation } from 'react-i18next';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { QueueFile } from '../../types';
import { SubtitleLine } from './SubtitleLine';
import { useTranslationStore } from '../../stores/translationStore';
import { memo, useCallback, useEffect, useMemo, useRef } from 'react';

// Debounce delay for scroll updates (in milliseconds)
const SCROLL_DEBOUNCE_MS = 150;

interface Props {
  file: QueueFile;
}

function SubtitleEditorBase({ file }: Props) {
  const { t } = useTranslation();
  const updateFile = useTranslationStore((s) => s.updateFile);
  const cancelFileTranslation = useTranslationStore((s) => s.cancelFileTranslation);
  const scrollRef = useRef<HTMLDivElement>(null);

  const entries = file.originalSubtitle?.entries || [];
  const translated = file.translatedEntries || [];
  const translatedRef = useRef(translated);

  useEffect(() => {
    translatedRef.current = translated;
  }, [translated]);

  const translatedCount = file.translatedLines;
  const progress = useMemo(() => (
    entries.length > 0
      ? Math.round((translatedCount / entries.length) * 100)
      : 0
  ), [entries.length, translatedCount]);
  const canCancel = file.status !== 'completed' && file.status !== 'error' && file.status !== 'cancelled';

  const handleTranslationChange = useCallback((index: number, newText: string) => {
    const current = translatedRef.current;
    const existing = current[index];
    if (!existing) return;
    const newTranslated = [...current];
    newTranslated[index] = { ...existing, text: newText };
    updateFile(file.id, { translatedEntries: newTranslated });
  }, [file.id, updateFile]);

  // Auto-scroll to current translation with debounce to avoid lag
  const scrollTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(() => {
    if (scrollRef.current && translatedCount > 0) {
      // Cancel pending scroll
      if (scrollTimeoutRef.current) {
        clearTimeout(scrollTimeoutRef.current);
      }
      // Debounce scroll to batch rapid updates
      scrollTimeoutRef.current = setTimeout(() => {
        const lastIndex = Math.min(translatedCount - 1, entries.length - 1);
        const lastItem = scrollRef.current?.querySelector(`[data-index="${lastIndex}"]`);
        lastItem?.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }, SCROLL_DEBOUNCE_MS);
    }
    return () => {
      if (scrollTimeoutRef.current) {
        clearTimeout(scrollTimeoutRef.current);
      }
    };
  }, [entries.length, translatedCount]);

  return (
    <Card className="p-4 h-[600px] flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div>
          <h3 className="font-semibold">{file.name}</h3>
          <p className="text-sm text-muted-foreground">
            {file.originalSubtitle?.format.toUpperCase()} • {entries.length} {t('translation.editor.lines')}
          </p>
        </div>
        <div className="flex items-center gap-3">
          <div className="text-right">
            <p className="text-sm font-medium">{progress}%</p>
            <div className="w-24 h-2 bg-muted rounded-full overflow-hidden">
              <div
                className="h-full bg-primary transition-all duration-300"
                style={{ width: `${progress}%` }}
              />
            </div>
          </div>
          {canCancel && (
            <Button
              size="sm"
              variant="outline"
              className="border-destructive/40 text-destructive hover:bg-destructive/10"
              onClick={() => cancelFileTranslation(file.id)}
              aria-label={t('translation.queue.cancelFile')}
            >
              {t('common.cancel')}
            </Button>
          )}
        </div>
      </div>

      <div className="grid grid-cols-2 gap-2 text-xs font-medium text-muted-foreground px-2 mb-2">
        <div>{t('translation.editor.original')}</div>
        <div>{t('translation.editor.translated')}</div>
      </div>

      <div ref={scrollRef} className="flex-1 overflow-y-auto space-y-1">
        {entries.map((entry, index) => (
          <SubtitleLine
            key={entry.index}
            index={index}
            original={entry}
            translated={translated[index]}
            onTranslationChange={handleTranslationChange}
          />
        ))}
      </div>
    </Card>
  );
}

export const SubtitleEditor = memo(SubtitleEditorBase, (prev, next) => (
  prev.file.id === next.file.id &&
  prev.file.name === next.file.name &&
  prev.file.originalSubtitle === next.file.originalSubtitle &&
  prev.file.translatedEntries === next.file.translatedEntries &&
  prev.file.translatedLines === next.file.translatedLines
));
