import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  FolderOpen, 
  X,
  FilmStrip,
  ListDashes
} from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { FileQueueItem } from './FileQueueItem';
import { useTranslationStore } from '../../stores/translationStore';
import { useShallow } from 'zustand/shallow';

interface Props {
  maxVisible?: number;
  showCancelAll?: boolean;
}

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.05,
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
  exit: {
    opacity: 0,
    x: 8,
    transition: {
      duration: 0.2,
    },
  },
};

export function FileQueue({ maxVisible, showCancelAll }: Props) {
  const { t } = useTranslation();
  const { 
    queue, 
    clearQueue, 
    setAllVideoTracks, 
    cancelAllTranslations 
  } = useTranslationStore(
    useShallow((s) => ({
      queue: s.queue,
      clearQueue: s.clearQueue,
      setAllVideoTracks: s.setAllVideoTracks,
      cancelAllTranslations: s.cancelAllTranslations,
    }))
  );

  const visibleQueue = useMemo(
    () => (maxVisible ? queue.slice(0, maxVisible) : queue),
    [queue, maxVisible]
  );
  const hiddenCount = useMemo(
    () => (maxVisible ? Math.max(0, queue.length - maxVisible) : 0),
    [queue.length, maxVisible]
  );

  // Find videos with available tracks
  const videosWithTracks = useMemo(
    () => queue.filter((f) => f.type === 'video' && f.subtitleTracks && f.subtitleTracks.length > 0),
    [queue]
  );
  const hasMultipleVideos = videosWithTracks.length > 1;

  // Get the maximum number of tracks available among all videos
  const maxTracks = useMemo(
    () => Math.max(...videosWithTracks.map((f) => f.subtitleTracks?.length ?? 0), 0),
    [videosWithTracks]
  );
  const trackOptions = useMemo(
    () => Array.from({ length: maxTracks }, (_, i) => i),
    [maxTracks]
  );
  const canCancelAll = useMemo(
    () => queue.some((file) => (
      file.status === 'pending' || ['extracting', 'translating', 'detecting_language', 'saving', 'muxing'].includes(file.status)
    )),
    [queue]
  );

  // Count file types
  const videoCount = queue.filter(f => f.type === 'video').length;
  const subtitleCount = queue.filter(f => f.type === 'subtitle').length;

  const handleQuickSelect = (value: string) => {
    if (value !== '') {
      setAllVideoTracks(Number(value));
    }
  };

  if (queue.length === 0) {
    return null;
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ type: 'spring', stiffness: 100, damping: 20 }}
      className="card-bento"
    >
      {/* Header */}
      <div className="flex items-start justify-between mb-5">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
              <ListDashes className="w-4 h-4 text-primary" />
            </div>
            <h3 className="text-title">{t('translation.queue.title')}</h3>
          </div>
          <div className="flex items-center gap-3 text-xs text-muted-foreground ml-10">
            <span>{queue.length} {queue.length === 1 ? t('translation.queue.file') : t('translation.queue.files')}</span>
            {videoCount > 0 && (
              <span className="flex items-center gap-1">
                <FilmStrip className="w-3 h-3" />
                {videoCount} video{videoCount > 1 ? 's' : ''}
              </span>
            )}
            {subtitleCount > 0 && (
              <span className="flex items-center gap-1">
                <ListDashes className="w-3 h-3" />
                {subtitleCount} subtitle{subtitleCount > 1 ? 's' : ''}
              </span>
            )}
          </div>
        </div>

        <div className="flex items-center gap-2">
          {showCancelAll && (
            <motion.div whileTap={{ scale: 0.98 }}>
              <Button
                size="sm"
                variant="outline"
                onClick={cancelAllTranslations}
                disabled={!canCancelAll}
                className="text-xs h-8 border-warning/30 text-warning hover:bg-warning/10"
              >
                {t('translation.queue.cancelAll')}
              </Button>
            </motion.div>
          )}
          
          {hasMultipleVideos && maxTracks > 0 && (
            <Select onValueChange={handleQuickSelect}>
              <Label className="sr-only">{t('translation.queue.applyTrackAll')}</Label>
              <SelectTrigger className="w-44 h-8 text-xs">
                <SelectValue placeholder={t('translation.queue.applyTrackAll')} />
              </SelectTrigger>
              <SelectContent>
                {trackOptions.map((trackIndex) => (
                  <SelectItem key={trackIndex} value={String(trackIndex)} className="text-xs">
                    {t('translation.queue.trackForAll', { index: trackIndex + 1 })}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
          
          <motion.div whileTap={{ scale: 0.98 }}>
            <Button
              size="sm"
              variant="ghost"
              onClick={clearQueue}
              className="h-8 w-8 p-0 text-muted-foreground hover:text-destructive"
            >
              <X className="w-4 h-4" />
            </Button>
          </motion.div>
        </div>
      </div>

      {/* File list */}
      <motion.div
        variants={containerVariants}
        initial="hidden"
        animate="visible"
        className="space-y-2"
      >
        <AnimatePresence mode="popLayout">
          {visibleQueue.map((file, index) => (
            <motion.div
              key={file.id}
              variants={itemVariants}
              layout
              exit="exit"
            >
              <FileQueueItem file={file} index={index} />
            </motion.div>
          ))}
        </AnimatePresence>

        {hiddenCount > 0 && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="flex items-center justify-center gap-2 py-3 px-4 rounded-xl bg-muted/50 text-sm text-muted-foreground"
          >
            <FolderOpen className="w-4 h-4" />
            <span>{t('translation.queue.moreFiles', { count: hiddenCount })}</span>
          </motion.div>
        )}
      </motion.div>
    </motion.div>
  );
}
