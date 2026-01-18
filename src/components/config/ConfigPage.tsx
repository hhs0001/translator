import { ApiSettings } from './ApiSettings';
import { FfmpegStatus } from './FfmpegStatus';
import { PromptEditor } from './PromptEditor';
import { TemplateManager } from './TemplateManager';
import { TranslationSettings } from './TranslationSettings';
import { OutputSettings } from './OutputSettings';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { FolderOpen } from 'lucide-react';
import { getAppDataDir, openFolder } from '@/utils/tauri';

export function ConfigPage() {
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
      <h2 className="text-2xl font-bold">Configurações</h2>

      <div className="grid gap-6">
        <ApiSettings />
        <FfmpegStatus />
        <PromptEditor />
        <TemplateManager />
        <TranslationSettings />
        <OutputSettings />

        <Card>
          <CardHeader>
            <CardTitle>Dados do Aplicativo</CardTitle>
          </CardHeader>
          <CardContent>
            <Button variant="outline" onClick={handleOpenConfigFolder}>
              <FolderOpen className="w-4 h-4 mr-2" />
              Abrir pasta de configurações
            </Button>
            <p className="text-xs text-muted-foreground mt-2">
              Contém settings.json e templates.json
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
