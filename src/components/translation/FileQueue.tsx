import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { FileQueueItem } from './FileQueueItem';
import { useTranslationStore } from '../../stores/translationStore';
import { useShallow } from 'zustand/shallow';

interface Props {
  maxVisible?: number;
  showCancelAll?: boolean;
}

export function FileQueue({ maxVisible, showCancelAll }: Props) {
  const { t } = useTranslation();
  const { queue, clearQueue, setAllVideoTracks, cancelAllTranslations } = useTranslationStore(
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

  const handleQuickSelect = (value: string) => {
    if (value !== '') {
      setAllVideoTracks(Number(value));
    }
  };

  if (queue.length === 0) {
    return null;
  }

  const fileLabel = queue.length === 1 ? t('translation.queue.file') : t('translation.queue.files');

  return (
    <Card className="p-4">
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold">
          {t('translation.queue.title')} ({queue.length} {fileLabel})
        </h3>
        <div className="flex items-center gap-2">
          {showCancelAll && (
            <Button
              size="sm"
              variant="outline"
              className="border-amber-500/30 text-amber-700 hover:bg-amber-500/10 dark:text-amber-300"
              onClick={cancelAllTranslations}
              disabled={!canCancelAll}
            >
              {t('translation.queue.cancelAll')}
            </Button>
          )}
          {hasMultipleVideos && maxTracks > 0 && (
            <Select onValueChange={handleQuickSelect}>
              <Label className="sr-only">{t('translation.queue.applyTrackAll')}</Label>
              <SelectTrigger className="w-48">
                <SelectValue placeholder={t('translation.queue.applyTrackAll')} />
              </SelectTrigger>
              <SelectContent>
                {trackOptions.map((trackIndex) => (
                  <SelectItem key={trackIndex} value={String(trackIndex)}>
                    {t('translation.queue.trackForAll', { index: trackIndex + 1 })}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
          <Button
            size="sm"
            variant="outline"
            className="border-destructive/30 text-destructive hover:bg-destructive/10"
            onClick={clearQueue}
          >
            {t('common.clear')}
          </Button>
        </div>
      </div>

      <div className="space-y-2">
        {visibleQueue.map((file, index) => (
          <FileQueueItem key={file.id} file={file} index={index} />
        ))}

        {hiddenCount > 0 && (
          <p className="text-center text-sm text-muted-foreground py-2">
            {t('translation.queue.moreFiles', { count: hiddenCount })}
          </p>
        )}
      </div>
    </Card>
  );
}
