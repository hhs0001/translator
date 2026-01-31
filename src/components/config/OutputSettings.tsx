import { useTranslation } from 'react-i18next';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { useSettingsStore } from '../../stores/settingsStore';

export function OutputSettings() {
  const { t } = useTranslation();
  const { settings, updateSetting } = useSettingsStore();

  const selectOutputDir = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const dir = await open({
        directory: true,
        multiple: false,
      });
      if (dir) {
        updateSetting('separateOutputDir', dir as string);
      }
    } catch (err) {
      console.error('Error opening dialog:', err);
    }
  };

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">{t('settings.output.title')}</h3>

      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">{t('settings.output.outputMode')}</label>
          <Select
            value={settings.outputMode}
            onValueChange={(value) => updateSetting('outputMode', value as 'mux' | 'separate')}
          >
            <Label className="sr-only">{t('settings.output.outputMode')}</Label>
            <SelectTrigger className="w-full">
              <SelectValue placeholder={t('settings.output.selectMode')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="separate">{t('settings.output.separateFile')}</SelectItem>
              <SelectItem value="mux">{t('settings.output.muxVideo')}</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {settings.outputMode === 'separate' && (
          <div>
            <label className="block text-sm font-medium mb-1">{t('settings.output.outputFolder')}</label>
            <div className="flex gap-2">
              <Input
                value={settings.separateOutputDir}
                onChange={(e) => updateSetting('separateOutputDir', e.target.value)}
                placeholder={t('settings.output.sameAsOriginal')}
                className="flex-1"
              />
              <Button variant="ghost" onClick={selectOutputDir}>
                {t('common.select')}
              </Button>
            </div>
          </div>
        )}

        {settings.outputMode === 'mux' && (
          <>
            <div>
              <label className="block text-sm font-medium mb-1">{t('settings.output.languageCode')}</label>
              <Input
                value={settings.muxLanguage}
                onChange={(e) => updateSetting('muxLanguage', e.target.value)}
                placeholder="por"
                className="w-32"
              />
              <p className="text-xs text-muted-foreground mt-1">
                {t('settings.output.languageCodeHint')}
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium mb-1">{t('settings.output.trackTitle')}</label>
              <Input
                value={settings.muxTitle}
                onChange={(e) => updateSetting('muxTitle', e.target.value)}
                placeholder="Portuguese"
                className="w-48"
              />
            </div>
          </>
        )}

        <div className="space-y-3 pt-2">
          <div className="flex items-center gap-2">
            <Switch
              id="cleanup-extracted"
              checked={settings.cleanupExtractedSubtitles}
              onCheckedChange={(checked) => updateSetting('cleanupExtractedSubtitles', checked)}
            />
            <Label htmlFor="cleanup-extracted">{t('settings.output.cleanupExtracted')}</Label>
          </div>
          <p className="text-xs text-muted-foreground ml-10">
            {t('settings.output.cleanupExtractedHint')}
          </p>

          {settings.outputMode === 'mux' && (
            <>
              <div className="flex items-center gap-2">
                <Switch
                  id="cleanup-mux"
                  checked={settings.cleanupMuxArtifacts}
                  onCheckedChange={(checked) => updateSetting('cleanupMuxArtifacts', checked)}
                />
                <Label htmlFor="cleanup-mux">{t('settings.output.cleanupMux')}</Label>
              </div>
              <p className="text-xs text-muted-foreground ml-10">
                {t('settings.output.cleanupMuxHint')}
              </p>
            </>
          )}
        </div>

        <div className="p-3 bg-muted/50 rounded-lg">
          <p className="text-sm">
            <span className="font-medium">{t('settings.output.originalFiles')}</span> {t('settings.output.preserved')}
          </p>
          <p className="text-xs text-muted-foreground mt-1">
            {settings.outputMode === 'mux'
              ? t('settings.output.muxOutputHint')
              : t('settings.output.separateOutputHint')}
          </p>
        </div>
      </div>
    </Card>
  );
}
