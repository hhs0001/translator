import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useTranslationStore } from '../../stores/translationStore';
import { useLogsStore } from '../../stores/logsStore';
import { useSettingsStore } from '../../stores/settingsStore';

interface Props {
  activeTab: string;
  onTabChange: (tab: string) => void;
}

export function Navbar({ activeTab, onTabChange }: Props) {
  const { isTranslating, isPaused, startTranslation, pauseTranslation, resumeTranslation, queue } = useTranslationStore();
  const { toggleDrawer, logs } = useLogsStore();
  const { ffmpegInstalled } = useSettingsStore();

  const pendingCount = queue.filter((f) => f.status === 'pending').length;
  const errorCount = logs.filter((l) => l.level === 'error').length;
  const isProcessing = queue.some(f => f.status === 'translating' || f.status === 'extracting');

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
    if (!isTranslating && !isProcessing) return 'Traduzir';
    if (isPaused) return 'Retomar';
    if (isProcessing) return 'Pausar';
    return 'Traduzir';
  };

  return (
    <nav className="fixed top-0 left-0 right-0 z-50 bg-background border-b border-border h-16">
      <div className="container mx-auto px-4 h-full flex items-center justify-between">
        <div className="flex items-center gap-6">
          <h1 className="text-xl font-bold">SubTranslator</h1>

          <Tabs value={activeTab} onValueChange={onTabChange} className="hidden sm:flex">
            <TabsList>
              <TabsTrigger value="translation">Tradução</TabsTrigger>
              <TabsTrigger value="config">Configurações</TabsTrigger>
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
              {pendingCount} na fila
            </Badge>
          )}

          <Button
            variant={isProcessing && !isPaused ? 'secondary' : 'default'}
            onClick={handleTranslateClick}
            disabled={queue.length === 0 && !isTranslating}
          >
            {getTranslateButtonText()}
          </Button>

          <Button
            variant="ghost"
            onClick={toggleDrawer}
            className="relative"
          >
            Logs
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
