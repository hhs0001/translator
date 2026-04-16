import { useTranslation } from 'react-i18next';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  Translate, 
  Gear, 
  FileText,
  Play,
  Pause,
  WarningCircle,
  CheckCircle,
  X
} from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { useTranslationStore } from '../../stores/translationStore';
import { useLogsStore } from '../../stores/logsStore';
import { useSettingsStore } from '../../stores/settingsStore';
import { useShallow } from 'zustand/shallow';

interface Props {
  activeTab: string;
  onTabChange: (tab: string) => void;
}

export function Navbar({ activeTab, onTabChange }: Props) {
  const { t } = useTranslation();
  const {
    isTranslating,
    isPaused,
    startTranslation,
    pauseTranslation,
    resumeTranslation,
    queueLength,
    pendingCount,
    isProcessing,
  } = useTranslationStore(useShallow((s) => {
    let pending = 0;
    let processing = false;
    for (const file of s.queue) {
      if (file.status === 'pending') pending += 1;
      if (file.status === 'translating' || file.status === 'extracting') {
        processing = true;
      }
    }
    return {
      isTranslating: s.isTranslating,
      isPaused: s.isPaused,
      startTranslation: s.startTranslation,
      pauseTranslation: s.pauseTranslation,
      resumeTranslation: s.resumeTranslation,
      queueLength: s.queue.length,
      pendingCount: pending,
      isProcessing: processing,
    };
  }));

  const { toggleDrawer, errorCount } = useLogsStore(useShallow((s) => ({
    toggleDrawer: s.toggleDrawer,
    errorCount: s.errorCount,
  })));

  const ffmpegInstalled = useSettingsStore((s) => s.ffmpegInstalled);

  const handleTranslateClick = () => {
    if (isProcessing) {
      if (isPaused) {
        resumeTranslation();
      } else {
        pauseTranslation();
      }
    } else {
      startTranslation();
    }
  };

  const getTranslateButtonState = () => {
    if (!isTranslating && !isProcessing) {
      return { text: t('navbar.translate'), icon: Play, variant: 'default' as const };
    }
    if (isPaused) {
      return { text: t('navbar.resume'), icon: Play, variant: 'default' as const };
    }
    if (isProcessing) {
      return { text: t('navbar.pause'), icon: Pause, variant: 'secondary' as const };
    }
    return { text: t('navbar.translate'), icon: Play, variant: 'default' as const };
  };

  const buttonState = getTranslateButtonState();
  const ButtonIcon = buttonState.icon;

  return (
    <nav className="fixed top-0 left-0 right-0 z-50">
      {/* Glass background */}
      <div className="absolute inset-0 glass-strong border-b border-border/50" />
      
      <div className="relative container-premium h-16 flex items-center justify-between">
        {/* Left: Logo + Navigation */}
        <div className="flex items-center gap-6">
          <motion.div 
            className="flex items-center gap-2"
            whileHover={{ scale: 1.02 }}
            transition={{ type: 'spring', stiffness: 400, damping: 25 }}
          >
            <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
              <Translate className="w-4 h-4 text-primary-foreground" weight="bold" />
            </div>
            <span className="text-lg font-semibold tracking-tight hidden sm:block">
              {t('navbar.title')}
            </span>
          </motion.div>

          {/* Tab navigation */}
          <div className="hidden sm:flex items-center gap-1 bg-muted/50 rounded-xl p-1">
            <motion.button
              onClick={() => onTabChange('translation')}
              className={`nav-pill ${activeTab === 'translation' ? 'data-[active=true]' : ''}`}
              data-active={activeTab === 'translation'}
              whileTap={{ scale: 0.98 }}
            >
              <FileText className="w-4 h-4" />
              <span className="hidden md:inline">{t('navbar.translation')}</span>
            </motion.button>
            <motion.button
              onClick={() => onTabChange('config')}
              className={`nav-pill ${activeTab === 'config' ? 'data-[active=true]' : ''}`}
              data-active={activeTab === 'config'}
              whileTap={{ scale: 0.98 }}
            >
              <Gear className="w-4 h-4" />
              <span className="hidden md:inline">{t('navbar.settings')}</span>
            </motion.button>
          </div>
        </div>

        {/* Right: Status + Actions */}
        <div className="flex items-center gap-3">
          {/* FFmpeg Status */}
          <AnimatePresence mode="wait">
            {ffmpegInstalled !== null && (
              <motion.div
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.9 }}
                transition={{ type: 'spring', stiffness: 400, damping: 25 }}
                className={`hidden md:flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-medium ${
                  ffmpegInstalled 
                    ? 'bg-[var(--success-subtle)] text-[var(--success)]' 
                    : 'bg-[var(--error-subtle)] text-[var(--error)]'
                }`}
              >
                {ffmpegInstalled ? (
                  <CheckCircle className="w-3.5 h-3.5" weight="fill" />
                ) : (
                  <X className="w-3.5 h-3.5" weight="bold" />
                )}
                <span>FFmpeg</span>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Queue count */}
          <AnimatePresence mode="wait">
            {pendingCount > 0 && (
              <motion.div
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.9 }}
                transition={{ type: 'spring', stiffness: 400, damping: 25 }}
                className="hidden md:flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-medium bg-[var(--info-subtle)] text-[var(--info)]"
              >
                <FileText className="w-3.5 h-3.5" />
                <span>{pendingCount} {t('navbar.inQueue')}</span>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Translate Button */}
          <motion.div whileTap={{ scale: 0.98 }}>
            <Button
              variant={buttonState.variant}
              onClick={handleTranslateClick}
              disabled={queueLength === 0 && !isTranslating}
              className="gap-2"
            >
              <ButtonIcon className="w-4 h-4" weight="fill" />
              <span className="hidden sm:inline">{buttonState.text}</span>
            </Button>
          </motion.div>

          {/* Logs Button */}
          <motion.div whileTap={{ scale: 0.98 }}>
            <Button
              variant="ghost"
              size="icon"
              onClick={toggleDrawer}
              className="relative"
            >
              <WarningCircle className="w-5 h-5" />
              <AnimatePresence>
                {errorCount > 0 && (
                  <motion.span
                    initial={{ scale: 0 }}
                    animate={{ scale: 1 }}
                    exit={{ scale: 0 }}
                    transition={{ type: 'spring', stiffness: 500, damping: 25 }}
                    className="absolute -top-1 -right-1 min-w-[18px] h-[18px] px-1 flex items-center justify-center bg-destructive text-destructive-foreground text-[10px] font-semibold rounded-full"
                  >
                    {errorCount > 9 ? '9+' : errorCount}
                  </motion.span>
                )}
              </AnimatePresence>
            </Button>
          </motion.div>
        </div>
      </div>
    </nav>
  );
}
