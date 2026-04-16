import { memo } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'framer-motion';
import { 
  FilmStrip, 
  FileText, 
  X,
  Prohibit,
  CheckCircle,
  WarningCircle,
  Spinner
} from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { QueueFile, FileStatus } from '../../types';
import { useTranslationStore } from '../../stores/translationStore';
import { SegmentedProgress } from './SegmentedProgress';

interface Props {
  file: QueueFile;
  index: number;
}

const STATUS_CONFIG: Record<FileStatus, { 
  icon: React.ComponentType<{ className?: string }>;
  className: string;
  bgClass: string;
  label: string;
}> = {
  pending: { 
    icon: FileText, 
    className: 'text-muted-foreground',
    bgClass: 'bg-muted',
    label: 'translation.status.pending'
  },
  extracting: { 
    icon: Spinner, 
    className: 'text-info animate-spin',
    bgClass: 'bg-info/10',
    label: 'translation.status.extracting'
  },
  translating: { 
    icon: Spinner, 
    className: 'text-info animate-spin',
    bgClass: 'bg-info/10',
    label: 'translation.status.translating'
  },
  detecting_language: { 
    icon: Spinner, 
    className: 'text-info animate-spin',
    bgClass: 'bg-info/10',
    label: 'translation.status.detecting_language'
  },
  saving: { 
    icon: Spinner, 
    className: 'text-info animate-spin',
    bgClass: 'bg-info/10',
    label: 'translation.status.saving'
  },
  muxing: { 
    icon: Spinner, 
    className: 'text-info animate-spin',
    bgClass: 'bg-info/10',
    label: 'translation.status.muxing'
  },
  paused: { 
    icon: Prohibit, 
    className: 'text-warning',
    bgClass: 'bg-warning/10',
    label: 'translation.status.paused'
  },
  cancelled: { 
    icon: X, 
    className: 'text-muted-foreground',
    bgClass: 'bg-muted',
    label: 'translation.status.cancelled'
  },
  completed: { 
    icon: CheckCircle, 
    className: 'text-success',
    bgClass: 'bg-success/10',
    label: 'translation.status.completed'
  },
  error: { 
    icon: WarningCircle, 
    className: 'text-error',
    bgClass: 'bg-error/10',
    label: 'translation.status.error'
  },
};

function getTrackLabel(track: { index: number; codec: string; language?: string; title?: string }, trackText: string) {
  const parts = [`${trackText} ${track.index + 1}`];
  if (track.language) parts.push(`(${track.language})`);
  if (track.title) parts.push(`- ${track.title}`);
  parts.push(`[${track.codec}]`);
  return parts.join(' ');
}

export const FileQueueItem = memo(function FileQueueItem({ file, index }: Props) {
  const { t } = useTranslation();
  const removeFile = useTranslationStore((s) => s.removeFile);
  const setSelectedTrack = useTranslationStore((s) => s.setSelectedTrack);
  const cancelFileTranslation = useTranslationStore((s) => s.cancelFileTranslation);
  
  const statusConfig = STATUS_CONFIG[file.status];
  const StatusIcon = statusConfig.icon;
  const statusLabel = t(statusConfig.label);

  const isProcessing = ['extracting', 'translating', 'detecting_language', 'saving', 'muxing'].includes(file.status);
  const canCancel = file.status !== 'completed' && file.status !== 'error' && file.status !== 'cancelled';
  const isVideo = file.type === 'video';
  const hasTracks = file.subtitleTracks && file.subtitleTracks.length > 0;
  const noTracks = file.subtitleTracks && file.subtitleTracks.length === 0 && !file.isLoadingTracks;
  const showProgress = isProcessing && file.progress > 0;

  const handleTrackChange = (value: string) => {
    if (value !== '') {
      setSelectedTrack(file.id, Number(value));
    }
  };

  return (
    <div className="group relative flex items-start gap-3 p-4 rounded-xl bg-muted/40 hover:bg-muted/60 transition-colors duration-200">
      {/* Index / Status Icon */}
      <div className={`flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center ${statusConfig.bgClass}`}>
        {isProcessing ? (
          <StatusIcon className={`w-4 h-4 ${statusConfig.className}`} />
        ) : (
          <span className="text-sm font-medium text-muted-foreground">
            {index + 1}
          </span>
        )}
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0 space-y-2">
        {/* File name and type */}
        <div className="flex items-center gap-2">
          {isVideo ? (
            <FilmStrip className="w-4 h-4 text-muted-foreground flex-shrink-0" />
          ) : (
            <FileText className="w-4 h-4 text-muted-foreground flex-shrink-0" />
          )}
          <span className="text-sm font-medium truncate" title={file.name}>
            {file.name}
          </span>
          <span className={`text-[10px] px-1.5 py-0.5 rounded-full ${statusConfig.bgClass} ${statusConfig.className}`}>
            {statusLabel}
          </span>
        </div>

        {/* Track selection for videos */}
        {isVideo && file.isLoadingTracks && (
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <Spinner className="w-3 h-3 animate-spin" />
            <span>{t('translation.video.detectingTracks')}</span>
          </div>
        )}

        {isVideo && noTracks && (
          <div className="flex items-center gap-2 text-xs text-warning">
            <WarningCircle className="w-3.5 h-3.5" />
            <span>{t('translation.video.noTracksFound')}</span>
          </div>
        )}

        {isVideo && hasTracks && (
          <div>
            <Select
              value={String(file.selectedTrackIndex ?? 0)}
              onValueChange={handleTrackChange}
              disabled={isProcessing}
            >
              <Label className="sr-only">{t('translation.video.subtitleTrack')}</Label>
              <SelectTrigger className="w-full max-w-sm h-8 text-xs bg-background/50">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {file.subtitleTracks!.map((track) => (
                  <SelectItem key={track.index} value={String(track.index)} className="text-xs">
                    {getTrackLabel(track, t('translation.video.track', { index: '' }).replace('{{index}}', '').trim())}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}

        {/* Progress bar */}
        {showProgress && (
          <div className="pt-1">
            {file.status === 'translating' && file.parallelProgress && file.parallelProgress.totalBatches > 1 ? (
              <SegmentedProgress
                progress={file.progress}
                totalLines={file.totalLines}
                translatedLines={file.translatedLines}
                batchProgress={file.parallelProgress}
              />
            ) : (
              <div className="space-y-1.5">
                <div className="flex items-center justify-between text-xs">
                  <span className="text-muted-foreground">
                    {file.status === 'translating' && file.totalLines > 0
                      ? `${file.translatedLines}/${file.totalLines} ${t('common.lines')}`
                      : statusLabel}
                  </span>
                  <span className="font-medium text-foreground">{Math.round(file.progress)}%</span>
                </div>
                <div className="h-1.5 w-full overflow-hidden rounded-full bg-muted">
                  <motion.div
                    className="h-full rounded-full bg-primary"
                    initial={{ width: 0 }}
                    animate={{ width: `${file.progress}%` }}
                    transition={{ type: 'spring', stiffness: 100, damping: 20 }}
                  />
                </div>
              </div>
            )}
          </div>
        )}

        {/* Error message */}
        {file.error && (
          <div className="flex items-start gap-2 text-xs text-error bg-error/5 rounded-lg p-2">
            <WarningCircle className="w-3.5 h-3.5 flex-shrink-0 mt-0.5" />
            <span className="line-clamp-2">{file.error}</span>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
        {canCancel && (
          <motion.div whileTap={{ scale: 0.95 }}>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => cancelFileTranslation(file.id)}
              className="h-7 px-2 text-xs text-warning hover:text-warning hover:bg-warning/10"
            >
              {t('common.cancel')}
            </Button>
          </motion.div>
        )}
        <motion.div whileTap={{ scale: 0.95 }}>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => removeFile(file.id)}
            disabled={isProcessing}
            className="h-7 w-7 p-0 text-muted-foreground hover:text-destructive hover:bg-destructive/10"
          >
            <X className="w-4 h-4" />
          </Button>
        </motion.div>
      </div>
    </div>
  );
});
