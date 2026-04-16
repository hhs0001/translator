import { memo, type ChangeEvent } from 'react';
import { motion } from 'framer-motion';
import { Check } from '@phosphor-icons/react';
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
    <motion.div
      data-index={index}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className={`
        grid grid-cols-2 gap-4 p-3 rounded-xl border transition-colors duration-200
        ${isTranslated 
          ? 'bg-success/5 border-success/20 hover:bg-success/10' 
          : 'bg-muted/30 border-transparent hover:bg-muted/50'
        }
      `}
    >
      {/* Original */}
      <div className="space-y-1.5">
        <div className="flex items-center gap-2">
          <div className="w-1.5 h-1.5 rounded-full bg-muted-foreground/40" />
          <span className="text-[10px] font-mono text-muted-foreground tracking-tight">
            {original.start_time} → {original.end_time}
          </span>
        </div>
        <p className="text-sm text-foreground leading-relaxed whitespace-pre-wrap">
          {original.text}
        </p>
      </div>

      {/* Translation */}
      <div className="space-y-1.5">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className={`w-1.5 h-1.5 rounded-full ${isTranslated ? 'bg-success' : 'bg-muted-foreground/30'}`} />
            <span className="text-[10px] font-medium text-muted-foreground">
              {isTranslated ? 'Translated' : 'Pending'}
            </span>
          </div>
          {isTranslated && (
            <motion.div
              initial={{ scale: 0 }}
              animate={{ scale: 1 }}
              className="flex items-center justify-center w-4 h-4 rounded-full bg-success/20"
            >
              <Check className="w-2.5 h-2.5 text-success" weight="bold" />
            </motion.div>
          )}
        </div>
        <Textarea
          aria-label={`Translation line ${index + 1}`}
          value={translated?.text || ''}
          onChange={(e: ChangeEvent<HTMLTextAreaElement>) => onTranslationChange(index, e.target.value)}
          placeholder={isTranslated ? '' : 'Waiting for translation...'}
          className={`
            w-full min-h-[60px] text-sm resize-none transition-all duration-200
            bg-background/50 border-border/50
            focus:bg-background focus:border-primary/50
            disabled:bg-transparent disabled:border-transparent
            ${isTranslated ? 'text-foreground' : 'text-muted-foreground italic'}
          `}
          disabled={!isTranslated}
        />
      </div>
    </motion.div>
  );
}

export const SubtitleLine = memo(SubtitleLineBase, (prev, next) => {
  // Deep comparison to avoid unnecessary re-renders
  if (prev.index !== next.index) return false;
  if (prev.original.text !== next.original.text) return false;
  if (prev.original.start_time !== next.original.start_time) return false;
  if (prev.original.end_time !== next.original.end_time) return false;

  // Compare translated
  const prevTranslated = prev.translated;
  const nextTranslated = next.translated;
  if (!prevTranslated && !nextTranslated) return true;
  if (!prevTranslated || !nextTranslated) return false;
  if (prevTranslated.text !== nextTranslated.text) return false;

  return true;
});
