import { Button, Chip, Tabs } from '@heroui/react';
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
    <nav className="fixed top-0 left-0 right-0 z-50 bg-default border-b border-divider h-16">
      <div className="container mx-auto px-4 h-full flex items-center justify-between">
        <div className="flex items-center gap-6">
          <h1 className="text-xl font-bold">SubTranslator</h1>

          <Tabs.Root
            selectedKey={activeTab}
            onSelectionChange={(key) => onTabChange(key as string)}
            className="hidden sm:flex"
          >
            <Tabs.ListContainer>
              <Tabs.List>
                <Tabs.Tab id="translation">
                  Tradução
                  <Tabs.Indicator />
                </Tabs.Tab>
                <Tabs.Tab id="config">
                  Configurações
                  <Tabs.Indicator />
                </Tabs.Tab>
              </Tabs.List>
            </Tabs.ListContainer>
          </Tabs.Root>
        </div>

        <div className="flex items-center gap-3">
          {ffmpegInstalled !== null && (
            <Chip
              color={ffmpegInstalled ? 'success' : 'danger'}
              size="sm"
              variant="soft"
            >
              FFmpeg {ffmpegInstalled ? 'OK' : 'N/A'}
            </Chip>
          )}

          {pendingCount > 0 && (
            <Chip color="accent" size="sm" variant="soft">
              {pendingCount} na fila
            </Chip>
          )}

          <Button
            variant={isProcessing && !isPaused ? 'secondary' : 'primary'}
            onPress={handleTranslateClick}
            isDisabled={queue.length === 0 && !isTranslating}
          >
            {getTranslateButtonText()}
          </Button>

          <Button
            variant="ghost"
            onPress={toggleDrawer}
            className="relative"
          >
            Logs
            {errorCount > 0 && (
              <span className="absolute -top-1 -right-1 bg-danger text-white text-xs rounded-full w-5 h-5 flex items-center justify-center">
                {errorCount}
              </span>
            )}
          </Button>
        </div>
      </div>
    </nav>
  );
}
