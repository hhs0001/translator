import { useTranslation } from 'react-i18next';
import { Card } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { useSettingsStore } from '../../stores/settingsStore';

export function TranslationSettings() {
  const { t } = useTranslation();
  const { settings, updateSetting } = useSettingsStore();

  // Streaming is only supported with OpenAI-compatible APIs (not direct Anthropic)
  const isAnthropicDirect = settings.apiFormat === 'anthropic';
  const streamingDisabled = isAnthropicDirect;

  const handleBatchSelect = (value: string) => {
    if (value) {
      const parsed = Number(value);
      if (!isNaN(parsed) && parsed > 0) {
        updateSetting('batchSize', parsed);
      }
    }
  };

  const handleNumberInput = (field: 'batchSize' | 'parallelRequests' | 'concurrency' | 'maxRetries', value: string) => {
    const num = parseInt(value, 10);
    if (!isNaN(num) && num >= 0) {
      updateSetting(field, num);
    }
  };

  const batchPresets = [
    { value: 25, label: `25 ${t('settings.translationSettings.lines')}` },
    { value: 50, label: `50 ${t('settings.translationSettings.lines')}` },
    { value: 100, label: `100 ${t('settings.translationSettings.lines')}` },
    { value: 150, label: `150 ${t('settings.translationSettings.lines')}` },
  ];

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">{t('settings.translationSettings.title')}</h3>

      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">{t('settings.translationSettings.batchSize')}</label>
          <div className="flex gap-2">
            <Select
              value={String(settings.batchSize)}
              onValueChange={handleBatchSelect}
            >
              <Label className="sr-only">{t('settings.translationSettings.batchSize')}</Label>
              <SelectTrigger className="flex-1">
                <SelectValue placeholder={t('common.select')} />
              </SelectTrigger>
              <SelectContent>
                {batchPresets.map((preset) => (
                  <SelectItem key={String(preset.value)} value={String(preset.value)}>
                    {preset.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Input
              type="number"
              value={String(settings.batchSize || 50)}
              onChange={(e) => handleNumberInput('batchSize', e.target.value)}
              className="w-24"
              min={1}
              max={500}
            />
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            {t('settings.translationSettings.batchSizeHint')}
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">{t('settings.translationSettings.parallelRequests')}</label>
          <Input
            type="number"
            value={String(settings.parallelRequests || 1)}
            onChange={(e) => handleNumberInput('parallelRequests', e.target.value)}
            className="w-24"
            min={1}
            max={10}
          />
          <p className="text-xs text-muted-foreground mt-1">
            {t('settings.translationSettings.parallelRequestsHint')}
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">{t('settings.translationSettings.concurrency')}</label>
          <Input
            type="number"
            value={String(settings.concurrency || 1)}
            onChange={(e) => handleNumberInput('concurrency', e.target.value)}
            className="w-24"
            min={1}
            max={10}
          />
          <p className="text-xs text-muted-foreground mt-1">
            {t('settings.translationSettings.concurrencyHint')}
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">{t('settings.translationSettings.maxRetries')}</label>
          <Input
            type="number"
            value={String(settings.maxRetries || 3)}
            onChange={(e) => handleNumberInput('maxRetries', e.target.value)}
            className="w-24"
            min={0}
            max={10}
          />
        </div>

        <div className="space-y-3 pt-2">
          <div className="flex items-center gap-2">
            <Switch
              id="streaming"
              checked={settings.streaming && !streamingDisabled}
              onCheckedChange={(checked) => updateSetting('streaming', checked)}
              disabled={streamingDisabled}
            />
            <Label htmlFor="streaming" className={streamingDisabled ? 'text-muted-foreground' : ''}>
              {t('settings.translationSettings.streaming')}
            </Label>
          </div>
          <p className="text-xs text-muted-foreground ml-10">
            {streamingDisabled
              ? t('settings.translationSettings.streamingDisabled')
              : t('settings.translationSettings.streamingHint')}
          </p>

          <div className="flex items-center gap-2">
            <Switch
              id="auto-continue"
              checked={settings.autoContinue}
              onCheckedChange={(checked) => updateSetting('autoContinue', checked)}
            />
            <Label htmlFor="auto-continue">{t('settings.translationSettings.autoContinue')}</Label>
          </div>

          <div className="flex items-center gap-2">
            <Switch
              id="continue-on-error"
              checked={settings.continueOnError}
              onCheckedChange={(checked) => updateSetting('continueOnError', checked)}
            />
            <Label htmlFor="continue-on-error">{t('settings.translationSettings.continueOnError')}</Label>
          </div>
        </div>
      </div>
    </Card>
  );
}
