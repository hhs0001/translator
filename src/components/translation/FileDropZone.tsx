import { useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  Upload, 
  FileVideo, 
  FileText,
  FolderOpen
} from '@phosphor-icons/react';
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { useFileHandler } from '../../hooks/useFileHandler';

const dropzoneVariants = {
  idle: {
    scale: 1,
    borderColor: 'rgba(228, 228, 231, 1)',
    backgroundColor: 'rgba(255, 255, 255, 0)',
  },
  dragging: {
    scale: 1.02,
    borderColor: 'rgba(79, 70, 229, 1)',
    backgroundColor: 'rgba(79, 70, 229, 0.05)',
  },
};

const iconVariants = {
  idle: { y: 0, rotate: 0 },
  dragging: { 
    y: [-4, 4, -4], 
    rotate: [0, -5, 5, 0],
    transition: {
      y: { repeat: Infinity, duration: 1.5, ease: 'easeInOut' as const },
      rotate: { repeat: Infinity, duration: 2, ease: 'easeInOut' as const },
    }
  },
};

export function FileDropZone() {
  const { t } = useTranslation();
  const [isDragging, setIsDragging] = useState(false);
  const { handleFileDrop, openFilePicker } = useFileHandler();

  // Listen for Tauri file drop events
  useEffect(() => {
    const unlisten = listen<{ paths: string[] }>('tauri://drop', (event) => {
      handleFileDrop(event.payload.paths);
      setIsDragging(false);
    });

    const unlistenHover = listen('tauri://drop-hover', () => {
      setIsDragging(true);
    });

    const unlistenCancel = listen('tauri://drop-cancelled', () => {
      setIsDragging(false);
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenHover.then((fn) => fn());
      unlistenCancel.then((fn) => fn());
    };
  }, [handleFileDrop]);

  // Also handle HTML5 drag events for visual feedback
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback(() => {
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    // Tauri handles the actual file paths
  }, []);

  return (
    <motion.div
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      variants={dropzoneVariants}
      initial="idle"
      animate={isDragging ? 'dragging' : 'idle'}
      transition={{ type: 'spring', stiffness: 300, damping: 25 }}
      className="relative overflow-hidden rounded-2xl border-2 border-dashed cursor-pointer group"
      onClick={openFilePicker}
    >
      {/* Background gradient animation on drag */}
      <AnimatePresence>
        {isDragging && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 bg-gradient-to-br from-primary/[0.05] to-transparent"
          />
        )}
      </AnimatePresence>

      {/* Animated border glow */}
      <motion.div
        className="absolute inset-0 rounded-2xl pointer-events-none"
        animate={{
          boxShadow: isDragging 
            ? '0 0 0 4px rgba(79, 70, 229, 0.1), inset 0 0 20px rgba(79, 70, 229, 0.05)' 
            : '0 0 0 0px rgba(79, 70, 229, 0)',
        }}
        transition={{ duration: 0.3 }}
      />

      <div className="relative p-8 md:p-10">
        <div className="flex flex-col items-center text-center gap-4">
          {/* Icon with animation */}
          <motion.div
            variants={iconVariants}
            animate={isDragging ? 'dragging' : 'idle'}
            className="relative"
          >
            <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-muted to-muted/50 flex items-center justify-center group-hover:from-primary/10 group-hover:to-primary/5 transition-colors duration-300">
              <Upload className="w-7 h-7 text-muted-foreground group-hover:text-primary transition-colors duration-300" />
            </div>
            
            {/* Floating file icons decoration */}
            <motion.div
              className="absolute -top-1 -right-1"
              animate={{
                y: isDragging ? [0, -8, 0] : 0,
                opacity: isDragging ? 1 : 0,
              }}
              transition={{ duration: 1, repeat: isDragging ? Infinity : 0 }}
            >
              <div className="w-6 h-6 rounded-lg bg-primary/10 flex items-center justify-center">
                <FileText className="w-3.5 h-3.5 text-primary" />
              </div>
            </motion.div>
            
            <motion.div
              className="absolute -bottom-1 -left-1"
              animate={{
                y: isDragging ? [0, 8, 0] : 0,
                opacity: isDragging ? 1 : 0,
              }}
              transition={{ duration: 1, delay: 0.2, repeat: isDragging ? Infinity : 0 }}
            >
              <div className="w-6 h-6 rounded-lg bg-primary/10 flex items-center justify-center">
                <FileVideo className="w-3.5 h-3.5 text-primary" />
              </div>
            </motion.div>
          </motion.div>

          {/* Text content */}
          <div className="space-y-1">
            <h3 className="text-title text-foreground">
              {t('translation.dropzone.title')}
            </h3>
            <p className="text-caption max-w-xs mx-auto">
              {t('translation.dropzone.subtitle')}
            </p>
          </div>

          {/* Divider with or */}
          <div className="flex items-center gap-3 w-full max-w-[200px]">
            <div className="h-px flex-1 bg-border/60" />
            <span className="text-xs text-muted-foreground uppercase tracking-wider">
              {t('common.or')}
            </span>
            <div className="h-px flex-1 bg-border/60" />
          </div>

          {/* Browse button */}
          <motion.button
            type="button"
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            onClick={(e) => {
              e.stopPropagation();
              openFilePicker();
            }}
            className="inline-flex items-center gap-2 px-4 py-2.5 rounded-lg text-sm font-medium text-foreground bg-background border border-border hover:bg-muted hover:border-border/80 transition-all elevation-1"
          >
            <FolderOpen className="w-4 h-4" />
            {t('translation.dropzone.selectFiles')}
          </motion.button>
        </div>
      </div>
    </motion.div>
  );
}
