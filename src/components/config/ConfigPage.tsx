import { useTranslation } from 'react-i18next';
import { ApiSettings } from './ApiSettings';
import { FfmpegStatus } from './FfmpegStatus';
import { PromptEditor } from './PromptEditor';
import { TemplateManager } from './TemplateManager';
import { TranslationSettings } from './TranslationSettings';
import { OutputSettings } from './OutputSettings';
import { LanguageSettings } from './LanguageSettings';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { FolderOpen } from 'lucide-react';
import { getAppDataDir, openFolder } from '@/utils/tauri';

export function ConfigPage() {
  const { t } = useTranslation();

  const handleOpenConfigFolder = async () => {
    try {
      const path = await getAppDataDir();
      await openFolder(path);
    } catch (error) {
      console.error('Failed to open config folder:', error);
    }
  };

  return (
    <div className="space-y-6 max-w-4xl mx-auto">
      <h2 className="text-2xl font-bold">{t('settings.title')}</h2>

      <div className="grid gap-6">
        <LanguageSettings />
        <ApiSettings />
        <FfmpegStatus />
        <PromptEditor />
        <TemplateManager />
        <TranslationSettings />
        <OutputSettings />

        <Card>
          <CardHeader>
            <CardTitle>{t('settings.appData.title')}</CardTitle>
          </CardHeader>
          <CardContent>
            <Button variant="outline" onClick={handleOpenConfigFolder}>
              <FolderOpen className="w-4 h-4 mr-2" />
              {t('settings.appData.openConfigFolder')}
            </Button>
            <p className="text-xs text-muted-foreground mt-2">
              {t('settings.appData.configFilesHint')}
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
