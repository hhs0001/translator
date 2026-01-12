import { TextArea } from '@heroui/react';
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
        ${isTranslated ? 'bg-success/10' : 'bg-default-100'}
      `}
    >
      {/* Original */}
      <div className="text-sm">
        <p className="text-xs text-default-500 mb-1">
          {original.start_time} → {original.end_time}
        </p>
        <p className="whitespace-pre-wrap">{original.text}</p>
      </div>

      {/* Translation */}
      <div>
        <TextArea
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
