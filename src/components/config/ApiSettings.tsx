import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/components/ui/accordion';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useSettingsStore } from '../../stores/settingsStore';
import { useModels } from '../../hooks/useModels';
import { Header, ApiFormat } from '../../types';

export function ApiSettings() {
  const { t } = useTranslation();
  const { settings, updateSetting } = useSettingsStore();
  const { models, isLoading, error, refetch, detectedFormat } = useModels();
  const [modelSearch, setModelSearch] = useState('');

  const filteredModels = useMemo(() => {
    const query = modelSearch.trim().toLowerCase();
    if (!query) {
      return models;
    }
    return models.filter((model) => {
      const name = model.name?.toLowerCase() || '';
      const id = model.id.toLowerCase();
      return name.includes(query) || id.includes(query);
    });
  }, [modelSearch, models]);

  const addHeader = () => {
    const newHeader: Header = {
      id: crypto.randomUUID(),
      key: '',
      value: '',
    };
    updateSetting('headers', [...settings.headers, newHeader]);
  };

  const updateHeader = (id: string, field: 'key' | 'value', value: string) => {
    const newHeaders = settings.headers.map((h) =>
      h.id === id ? { ...h, [field]: value } : h
    );
    updateSetting('headers', newHeaders);
  };

  const removeHeader = (id: string) => {
    updateSetting('headers', settings.headers.filter((h) => h.id !== id));
  };

  const handleModelChange = (value: string) => {
    updateSetting('model', value);
  };

  const handleFormatChange = (value: string) => {
    updateSetting('apiFormat', value as ApiFormat);
  };

  const formatDisplayName = detectedFormat === 'anthropic' ? 'Anthropic' : 'OpenAI';

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">{t('settings.api.title')}</h3>

      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">{t('settings.api.baseUrl')}</label>
          <Input
            placeholder="http://localhost:8045/v1"
            value={settings.baseUrl}
            onChange={(e) => updateSetting('baseUrl', e.target.value)}
            className="w-full"
          />
          <p className="text-xs text-muted-foreground mt-1">
            {t('settings.api.baseUrlHint')}
            <span className="ml-2 px-2 py-0.5 rounded bg-muted text-muted-foreground font-medium">
              {t('settings.api.format')}: {formatDisplayName}
            </span>
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">{t('settings.api.apiFormat')}</label>
          <Select value={settings.apiFormat} onValueChange={handleFormatChange}>
            <Label className="sr-only">{t('settings.api.apiFormat')}</Label>
            <SelectTrigger className="w-full">
              <SelectValue placeholder={t('settings.api.selectMode')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="auto">{t('settings.api.autoDetect')}</SelectItem>
              <SelectItem value="openai">OpenAI Compatible</SelectItem>
              <SelectItem value="anthropic">Anthropic</SelectItem>
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground mt-1">
            {t('settings.api.apiFormatHint')}
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">{t('settings.api.apiKey')}</label>
          <Input
            type="password"
            placeholder={detectedFormat === 'anthropic' ? 'sk-ant-...' : 'sk-...'}
            value={settings.apiKey}
            onChange={(e) => updateSetting('apiKey', e.target.value)}
            className="w-full"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">{t('settings.api.model')}</label>
          <Input
            placeholder={t('settings.api.searchModels')}
            value={modelSearch}
            onChange={(e) => setModelSearch(e.target.value)}
            className="w-full mb-2"
            disabled={isLoading || models.length === 0}
          />
          <div className="flex gap-2">
            <Select
              value={settings.model}
              onValueChange={handleModelChange}
              disabled={isLoading || models.length === 0}
            >
              <Label className="sr-only">{t('settings.api.model')}</Label>
              <SelectTrigger className="flex-1">
                <SelectValue placeholder={isLoading ? t('common.loading') : t('settings.api.selectModel')} />
              </SelectTrigger>
              <SelectContent>
                {filteredModels.length === 0 ? (
                  <SelectItem value="no-results" disabled>
                    {t('settings.api.noModelsFound')}
                  </SelectItem>
                ) : (
                  filteredModels.map((model) => {
                    const displayName = model.name || model.id;
                    const contextInfo = model.context_length
                      ? ` (${Math.round(model.context_length / 1000)}k ctx)`
                      : '';
                    return (
                      <SelectItem key={model.id} value={model.id}>
                        <div className="flex flex-col">
                          <span>
                            {displayName}
                            {contextInfo}
                          </span>
                          {model.name && (
                            <span className="text-xs text-muted-foreground">{model.id}</span>
                          )}
                        </div>
                      </SelectItem>
                    );
                  })
                )}
              </SelectContent>
            </Select>
            <Button variant="ghost" onClick={refetch} disabled={isLoading}>
              {String.fromCodePoint(0x21bb)}
            </Button>
          </div>
          {error && <p className="text-xs text-destructive mt-1">{error}</p>}
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">{t('settings.api.customModel')}</label>
          <Input
            placeholder={detectedFormat === 'anthropic' ? 'claude-sonnet-4-5-20250929' : 'gpt-4-turbo'}
            value={settings.customModel}
            onChange={(e) => updateSetting('customModel', e.target.value)}
            className="w-full"
          />
          <p className="text-xs text-muted-foreground mt-1">
            {t('settings.api.customModelHint')}
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">{t('settings.api.languageDetectionModel')}</label>
          <Select
            value={settings.languageDetectionModel || 'none'}
            onValueChange={(value) =>
              updateSetting('languageDetectionModel', value === 'none' ? '' : value)
            }
            disabled={isLoading || models.length === 0}
          >
            <Label className="sr-only">{t('settings.api.languageDetectionModel')}</Label>
            <SelectTrigger className="w-full">
              <SelectValue placeholder={t('settings.api.none')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="none">{t('settings.api.none')}</SelectItem>
              {models.map((model) => {
                const displayName = model.name || model.id;
                return (
                  <SelectItem key={model.id} value={model.id}>
                    <div className="flex flex-col">
                      <span>{displayName}</span>
                      {model.name && (
                        <span className="text-xs text-muted-foreground">{model.id}</span>
                      )}
                    </div>
                  </SelectItem>
                );
              })}
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground mt-1">
            {t('settings.api.languageDetectionModelHint')}
          </p>
        </div>

        <Accordion type="single" collapsible>
          <AccordionItem value="headers">
            <AccordionTrigger>{t('settings.api.advancedHeaders')}</AccordionTrigger>
            <AccordionContent>
              <div className="space-y-2">
                {settings.headers.map((header) => (
                  <div key={header.id} className="flex gap-2">
                    <Input
                      placeholder="Header-Name"
                      value={header.key}
                      onChange={(e) => updateHeader(header.id, 'key', e.target.value)}
                      className="flex-1"
                    />
                    <Input
                      placeholder="Value"
                      value={header.value}
                      onChange={(e) => updateHeader(header.id, 'value', e.target.value)}
                      className="flex-1"
                    />
                    <Button
                      variant="destructive"
                      size="sm"
                      onClick={() => removeHeader(header.id)}
                    >
                      {String.fromCodePoint(0x2715)}
                    </Button>
                  </div>
                ))}
                <Button variant="default" size="sm" onClick={addHeader}>
                  {t('settings.api.addHeader')}
                </Button>
              </div>
            </AccordionContent>
          </AccordionItem>
        </Accordion>
      </div>
    </Card>
  );
}
