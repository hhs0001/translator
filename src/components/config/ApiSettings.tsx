import { useMemo, useState } from 'react';
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
      <h3 className="text-lg font-semibold mb-4">Conexao com API</h3>

      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">Base URL</label>
          <Input
            placeholder="http://localhost:8045/v1"
            value={settings.baseUrl}
            onChange={(e) => updateSetting('baseUrl', e.target.value)}
            className="w-full"
          />
          <p className="text-xs text-muted-foreground mt-1">
            O endpoint sera adicionado automaticamente.
            <span className="ml-2 px-2 py-0.5 rounded bg-muted text-muted-foreground font-medium">
              Formato: {formatDisplayName}
            </span>
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Formato da API</label>
          <Select value={settings.apiFormat} onValueChange={handleFormatChange}>
            <Label className="sr-only">Formato da API</Label>
            <SelectTrigger className="w-full">
              <SelectValue placeholder="Selecione o formato" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="auto">Auto-detectar</SelectItem>
              <SelectItem value="openai">OpenAI Compatible</SelectItem>
              <SelectItem value="anthropic">Anthropic</SelectItem>
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground mt-1">
            Selecione o formato da API ou deixe em auto-detectar
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">API Key</label>
          <Input
            type="password"
            placeholder={detectedFormat === 'anthropic' ? 'sk-ant-...' : 'sk-...'}
            value={settings.apiKey}
            onChange={(e) => updateSetting('apiKey', e.target.value)}
            className="w-full"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Modelo</label>
          <Input
            placeholder="Pesquisar modelos"
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
              <Label className="sr-only">Modelo</Label>
              <SelectTrigger className="flex-1">
                <SelectValue placeholder={isLoading ? 'Carregando...' : 'Selecione um modelo'} />
              </SelectTrigger>
              <SelectContent>
                {filteredModels.length === 0 ? (
                  <SelectItem value="no-results" disabled>
                    Nenhum modelo encontrado
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
          <label className="block text-sm font-medium mb-1">Modelo Customizado (opcional)</label>
          <Input
            placeholder={detectedFormat === 'anthropic' ? 'claude-sonnet-4-5-20250929' : 'gpt-4-turbo'}
            value={settings.customModel}
            onChange={(e) => updateSetting('customModel', e.target.value)}
            className="w-full"
          />
          <p className="text-xs text-muted-foreground mt-1">
            Se preenchido, sera usado em vez do modelo selecionado acima
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Modelo para Deteccao de Idioma (opcional)</label>
          <Select
            value={settings.languageDetectionModel || 'none'}
            onValueChange={(value) =>
              updateSetting('languageDetectionModel', value === 'none' ? '' : value)
            }
            disabled={isLoading || models.length === 0}
          >
            <Label className="sr-only">Modelo para Deteccao</Label>
            <SelectTrigger className="w-full">
              <SelectValue placeholder="Nenhum (usar configuracao manual)" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="none">Nenhum (usar configuracao manual)</SelectItem>
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
            Modelo leve para detectar o idioma da traducao e usar no mux (ex: gemma-3-1b)
          </p>
        </div>

        <Accordion type="single" collapsible>
          <AccordionItem value="headers">
            <AccordionTrigger>Headers Avancados</AccordionTrigger>
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
                  + Adicionar Header
                </Button>
              </div>
            </AccordionContent>
          </AccordionItem>
        </Accordion>
      </div>
    </Card>
  );
}
