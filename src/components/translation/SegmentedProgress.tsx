import { memo } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'framer-motion';
import { BatchProgress } from '../../types';

interface SegmentedProgressProps {
  progress: number;
  totalLines: number;
  translatedLines: number;
  batchProgress?: {
    totalBatches: number;
    activeBatches: number;
    completedBatches: number;
    batchProgresses: BatchProgress[];
  };
  className?: string;
}

export const SegmentedProgress = memo(function SegmentedProgress({
  progress,
  totalLines,
  translatedLines,
  batchProgress,
  className = ''
}: SegmentedProgressProps) {
  const { t } = useTranslation();
  
  // Se não temos info de batches ou só tem 1 batch, mostra barra simples
  if (!batchProgress || batchProgress.totalBatches <= 1) {
    return (
      <div className={`space-y-1.5 ${className}`}>
        <div className="flex items-center justify-between text-xs">
          <span className="text-muted-foreground">
            {translatedLines}/{totalLines} {t('common.lines')}
          </span>
          <span className="font-medium text-foreground">{Math.round(progress)}%</span>
        </div>
        <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
          <motion.div
            className="h-full rounded-full bg-primary"
            initial={{ width: 0 }}
            animate={{ width: `${progress}%` }}
            transition={{ type: 'spring', stiffness: 100, damping: 20 }}
          />
        </div>
      </div>
    );
  }

  const { totalBatches, activeBatches, completedBatches, batchProgresses } = batchProgress;
  const gap = 2; // gap em pixels entre segmentos
  const segmentWidth = `calc((100% - ${(totalBatches - 1) * gap}px) / ${totalBatches})`;

  return (
    <div className={`space-y-2 ${className}`}>
      {/* Header com stats */}
      <div className="flex items-center justify-between text-xs">
        <div className="flex items-center gap-2">
          <span className="text-muted-foreground">
            {translatedLines}/{totalLines} {t('common.lines')}
          </span>
          <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-muted text-muted-foreground">
            {completedBatches}/{totalBatches} {t('common.batches')}
          </span>
          {activeBatches > 0 && (
            <motion.span 
              className="text-[10px] px-1.5 py-0.5 rounded-full bg-info/20 text-info"
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
            >
              {activeBatches} {t('common.active')}
            </motion.span>
          )}
        </div>
        <span className="font-medium text-foreground">{Math.round(progress)}%</span>
      </div>

      {/* Barra segmentada */}
      <div 
        className="flex h-2.5 w-full"
        style={{ gap: `${gap}px` }}
      >
        {batchProgresses.map((batch, idx) => {
          const isCompleted = batch.status === 'completed';
          const isActive = batch.status === 'active';
          const isError = batch.status === 'error';
          const isPending = batch.status === 'pending';
          
          // Calcula o progresso dentro deste batch (0-100%)
          const batchPercent = batch.totalInBatch > 0 
            ? (batch.completedInBatch / batch.totalInBatch) * 100 
            : 0;

          return (
            <motion.div
              key={batch.batchIndex}
              className="relative h-full overflow-hidden rounded-sm"
              style={{ width: segmentWidth }}
              initial={{ opacity: 0, y: 4 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: idx * 0.03 }}
            >
              {/* Background base */}
              <div className={`absolute inset-0 rounded-sm ${
                isError ? 'bg-destructive/30' :
                isCompleted ? 'bg-success/30' :
                isActive ? 'bg-muted' :
                'bg-muted'
              }`} />
              
              {/* Progresso preenchido */}
              <motion.div
                className={`absolute inset-y-0 left-0 rounded-sm ${
                  isError ? 'bg-destructive' :
                  isCompleted ? 'bg-success' :
                  isActive ? 'bg-primary' :
                  'bg-transparent'
                }`}
                initial={{ width: 0 }}
                animate={{ width: isPending ? 0 : `${batchPercent}%` }}
                transition={{ 
                  type: 'spring', 
                  stiffness: isActive ? 150 : 100, 
                  damping: 15 
                }}
              />

              {/* Indicador de atividade (pulsing para batches ativos) */}
              {isActive && (
                <motion.div
                  className="absolute inset-0 bg-primary/30 rounded-sm"
                  animate={{ 
                    opacity: [0.3, 0.6, 0.3],
                  }}
                  transition={{ 
                    duration: 1.5, 
                    repeat: Infinity, 
                    ease: 'easeInOut',
                    delay: idx * 0.1 
                  }}
                />
              )}

              {/* Tooltip/hover indicator */}
              <div className="absolute inset-0 opacity-0 hover:opacity-100 transition-opacity">
                <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-1 px-1.5 py-0.5 bg-popover text-popover-foreground text-[9px] rounded shadow-lg whitespace-nowrap z-10">
                  {t('translation.batchTooltip', { 
                    number: idx + 1, 
                    completed: batch.completedInBatch, 
                    total: batch.totalInBatch 
                  })}
                </div>
              </div>
            </motion.div>
          );
        })}
      </div>

      {/* Mini legenda */}
      <div className="flex items-center gap-3 text-[10px] text-muted-foreground">
        <div className="flex items-center gap-1">
          <div className="w-2 h-2 rounded-sm bg-success" />
          <span>{t('common.done')}</span>
        </div>
        <div className="flex items-center gap-1">
          <div className="w-2 h-2 rounded-sm bg-primary animate-pulse" />
          <span>{t('common.active')}</span>
        </div>
        <div className="flex items-center gap-1">
          <div className="w-2 h-2 rounded-sm bg-muted" />
          <span>{t('common.pending')}</span>
        </div>
      </div>
    </div>
  );
});
