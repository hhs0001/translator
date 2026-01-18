import { useTranslation } from 'react-i18next';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
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

  const getTranslateButtonText = () => {
    if (!isTranslating && !isProcessing) return t('navbar.translate');
    if (isPaused) return t('navbar.resume');
    if (isProcessing) return t('navbar.pause');
    return t('navbar.translate');
  };

  return (
    <nav className="fixed top-0 left-0 right-0 z-50 bg-background border-b border-border h-16">
      <div className="container mx-auto px-4 h-full flex items-center justify-between">
        <div className="flex items-center gap-6">
          <h1 className="text-xl font-bold">{t('navbar.title')}</h1>

          <Tabs value={activeTab} onValueChange={onTabChange} className="hidden sm:flex">
            <TabsList>
              <TabsTrigger value="translation">{t('navbar.translation')}</TabsTrigger>
              <TabsTrigger value="config">{t('navbar.settings')}</TabsTrigger>
            </TabsList>
          </Tabs>
        </div>

        <div className="flex items-center gap-3">
          {ffmpegInstalled !== null && (
            <Badge
              variant="outline"
              className={
                ffmpegInstalled
                  ? 'border-transparent bg-emerald-500/15 text-emerald-700 dark:text-emerald-300'
                  : 'border-transparent bg-destructive/15 text-destructive'
              }
            >
              FFmpeg {ffmpegInstalled ? 'OK' : 'N/A'}
            </Badge>
          )}

          {pendingCount > 0 && (
            <Badge
              variant="outline"
              className="border-transparent bg-sky-500/15 text-sky-700 dark:text-sky-300"
            >
              {pendingCount} {t('navbar.inQueue')}
            </Badge>
          )}

          <Button
            variant={isProcessing && !isPaused ? 'secondary' : 'default'}
            onClick={handleTranslateClick}
            disabled={queueLength === 0 && !isTranslating}
          >
            {getTranslateButtonText()}
          </Button>

          <Button
            variant="ghost"
            onClick={toggleDrawer}
            className="relative"
          >
            {t('navbar.logs')}
            {errorCount > 0 && (
              <span className="absolute -top-1 -right-1 bg-destructive text-white text-xs rounded-full w-5 h-5 flex items-center justify-center">
                {errorCount}
              </span>
            )}
          </Button>
        </div>
      </div>
    </nav>
  );
}
