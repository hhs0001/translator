import { useTranslation } from 'react-i18next';
import { motion } from 'framer-motion';
import { 
  FileText, 
  X,
  Prohibit,
  CheckCircle
} from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { QueueFile } from '../../types';
import { SubtitleLine } from './SubtitleLine';
import { useTranslationStore } from '../../stores/translationStore';
import { memo, useCallback, useEffect, useMemo, useRef } from 'react';

// Debounce delay for scroll updates (in milliseconds)
const SCROLL_DEBOUNCE_MS = 150;

interface Props {
  file: QueueFile;
}

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.01,
      delayChildren: 0.05,
    },
  },
};

const itemVariants = {
  hidden: { opacity: 0, x: -8 },
  visible: {
    opacity: 1,
    x: 0,
    transition: {
      type: 'spring' as const,
      stiffness: 100,
      damping: 20,
    },
  },
};

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
  const isCompleted = file.status === 'completed';
  const isError = file.status === 'error';

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
    <motion.div 
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ type: 'spring', stiffness: 100, damping: 20 }}
      className="card-bento h-[600px] flex flex-col"
    >
      {/* Header */}
      <div className="flex items-start justify-between mb-5">
        <div className="flex items-start gap-3">
          <div className={`w-10 h-10 rounded-xl flex items-center justify-center flex-shrink-0 ${
            isCompleted 
              ? 'bg-success/10' 
              : isError 
                ? 'bg-error/10' 
                : 'bg-primary/10'
          }`}>
            {isCompleted ? (
              <CheckCircle className="w-5 h-5 text-success" />
            ) : isError ? (
              <X className="w-5 h-5 text-error" />
            ) : (
              <FileText className="w-5 h-5 text-primary" />
            )}
          </div>
          <div className="min-w-0">
            <h3 className="text-title truncate" title={file.name}>{file.name}</h3>
            <p className="text-caption">
              {file.originalSubtitle?.format.toUpperCase()} • {entries.length} {t('translation.editor.lines')}
            </p>
          </div>
        </div>
        
        <div className="flex items-center gap-3 flex-shrink-0">
          {/* Progress */}
          <div className="text-right">
            <p className="text-sm font-semibold">{progress}%</p>
            <div className="w-28 h-1.5 bg-muted rounded-full overflow-hidden mt-1">
              <motion.div
                className={`h-full rounded-full ${
                  isCompleted 
                    ? 'bg-success' 
                    : isError 
                      ? 'bg-error' 
                      : 'bg-primary'
                }`}
                initial={{ width: 0 }}
                animate={{ width: `${progress}%` }}
                transition={{ type: 'spring', stiffness: 100, damping: 20 }}
              />
            </div>
          </div>
          
          {canCancel && (
            <motion.div whileTap={{ scale: 0.95 }}>
              <Button
                size="sm"
                variant="ghost"
                className="h-8 px-2 text-warning hover:text-warning hover:bg-warning/10"
                onClick={() => cancelFileTranslation(file.id)}
                aria-label={t('translation.queue.cancelFile')}
              >
                <Prohibit className="w-4 h-4" />
              </Button>
            </motion.div>
          )}
        </div>
      </div>

      {/* Column Headers */}
      <div className="grid grid-cols-2 gap-4 text-xs font-medium text-muted-foreground px-3 pb-2 border-b border-border/50 mb-2">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 rounded-full bg-muted-foreground/30" />
          {t('translation.editor.original')}
        </div>
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 rounded-full bg-primary" />
          {t('translation.editor.translated')}
        </div>
      </div>

      {/* Subtitle Lines */}
      <motion.div 
        ref={scrollRef} 
        variants={containerVariants}
        initial="hidden"
        animate="visible"
        className="flex-1 overflow-y-auto space-y-1 -mx-2 px-2"
      >
        {entries.map((entry, index) => (
          <motion.div key={entry.index} variants={itemVariants}>
            <SubtitleLine
              index={index}
              original={entry}
              translated={translated[index]}
              onTranslationChange={handleTranslationChange}
            />
          </motion.div>
        ))}
      </motion.div>
    </motion.div>
  );
}

export const SubtitleEditor = memo(SubtitleEditorBase, (prev, next) => (
  prev.file.id === next.file.id &&
  prev.file.name === next.file.name &&
  prev.file.originalSubtitle === next.file.originalSubtitle &&
  prev.file.translatedEntries === next.file.translatedEntries &&
  prev.file.translatedLines === next.file.translatedLines
));
