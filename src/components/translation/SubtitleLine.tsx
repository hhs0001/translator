import { memo, type ChangeEvent } from 'react';
import { Textarea } from '@/components/ui/textarea';
import { SubtitleEntry } from '../../types';

interface Props {
  index: number;
  original: SubtitleEntry;
  translated?: SubtitleEntry;
  onTranslationChange: (index: number, text: string) => void;
}

function SubtitleLineBase({ index, original, translated, onTranslationChange }: Props) {
  const isTranslated = !!translated;

  return (
    <div
      data-index={index}
      className={`
        grid grid-cols-2 gap-2 p-2 rounded-lg
        ${isTranslated ? 'bg-emerald-500/10' : 'bg-muted/50'}
      `}
    >
      {/* Original */}
      <div className="text-sm">
        <p className="text-xs text-muted-foreground mb-1">
          {original.start_time} → {original.end_time}
        </p>
        <p className="whitespace-pre-wrap">{original.text}</p>
      </div>

      {/* Translation */}
      <div>
        <Textarea
          aria-label={`Tradução linha ${index + 1}`}
          value={translated?.text || ''}
          onChange={(e: ChangeEvent<HTMLTextAreaElement>) => onTranslationChange(index, e.target.value)}
          placeholder={isTranslated ? '' : 'Aguardando tradução...'}
          className="w-full text-sm h-16"
          disabled={!isTranslated}
        />
      </div>
    </div>
  );
}

export const SubtitleLine = memo(SubtitleLineBase, (prev, next) => {
  // Comparação profunda para evitar re-renders desnecessários
  if (prev.index !== next.index) return false;
  if (prev.original.text !== next.original.text) return false;
  if (prev.original.start_time !== next.original.start_time) return false;
  if (prev.original.end_time !== next.original.end_time) return false;

  // Comparar translated
  const prevTranslated = prev.translated;
  const nextTranslated = next.translated;
  if (!prevTranslated && !nextTranslated) return true;
  if (!prevTranslated || !nextTranslated) return false;
  if (prevTranslated.text !== nextTranslated.text) return false;

  return true;
});
