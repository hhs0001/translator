import { Card } from '@/components/ui/card';
import { QueueFile } from '../../types';
import { SubtitleLine } from './SubtitleLine';
import { useTranslationStore } from '../../stores/translationStore';
import { memo, useCallback, useEffect, useMemo, useRef } from 'react';

interface Props {
  file: QueueFile;
}

function SubtitleEditorBase({ file }: Props) {
  const updateFile = useTranslationStore((s) => s.updateFile);
  const scrollRef = useRef<HTMLDivElement>(null);

  const entries = file.originalSubtitle?.entries || [];
  const translated = file.translatedEntries || [];
  const translatedRef = useRef(translated);

  useEffect(() => {
    translatedRef.current = translated;
  }, [translated]);

  const progress = useMemo(() => (
    entries.length > 0
      ? Math.round((translated.length / entries.length) * 100)
      : 0
  ), [entries.length, translated.length]);

  const handleTranslationChange = useCallback((index: number, newText: string) => {
    const current = translatedRef.current;
    const existing = current[index];
    if (!existing) return;
    const newTranslated = [...current];
    newTranslated[index] = { ...existing, text: newText };
    updateFile(file.id, { translatedEntries: newTranslated });
  }, [file.id, updateFile]);

  // Auto-scroll to current translation
  useEffect(() => {
    if (scrollRef.current && translated.length > 0) {
      const lastItem = scrollRef.current.querySelector(`[data-index="${translated.length - 1}"]`);
      lastItem?.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }, [translated.length]);

  return (
    <Card className="p-4 h-[600px] flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div>
          <h3 className="font-semibold">{file.name}</h3>
          <p className="text-sm text-muted-foreground">
            {file.originalSubtitle?.format.toUpperCase()} • {entries.length} linhas
          </p>
        </div>
        <div className="text-right">
          <p className="text-sm font-medium">{progress}%</p>
          <div className="w-24 h-2 bg-muted rounded-full overflow-hidden">
            <div
              className="h-full bg-primary transition-all duration-300"
              style={{ width: `${progress}%` }}
            />
          </div>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-2 text-xs font-medium text-muted-foreground px-2 mb-2">
        <div>Original</div>
        <div>Tradução</div>
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
  prev.file.translatedEntries === next.file.translatedEntries
));
