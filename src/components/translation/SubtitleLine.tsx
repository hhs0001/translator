import { Textarea } from '@/components/ui/textarea';
import { SubtitleEntry } from '../../types';

interface Props {
  index: number;
  original: SubtitleEntry;
  translated?: SubtitleEntry;
  onTranslationChange: (text: string) => void;
}

export function SubtitleLine({ index, original, translated, onTranslationChange }: Props) {
  const isTranslated = !!translated;

  return (
    <div
      data-index={index}
      className={`
        grid grid-cols-2 gap-2 p-2 rounded-lg transition-colors
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
          onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => onTranslationChange(e.target.value)}
          placeholder={isTranslated ? '' : 'Aguardando tradução...'}
          className="w-full text-sm h-16"
          disabled={!isTranslated}
        />
      </div>
    </div>
  );
}
