import { useMemo, useState } from 'react';
import { Card, Input, Select, Label, ListBox, Accordion, Button } from '@heroui/react';
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

  const handleModelChange = (key: React.Key | null) => {
    if (key) {
      updateSetting('model', String(key));
    }
  };

  const handleFormatChange = (key: React.Key | null) => {
    if (key) {
      updateSetting('apiFormat', key as ApiFormat);
    }
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
          <p className="text-xs text-default-500 mt-1">
            O endpoint sera adicionado automaticamente.
            <span className="ml-2 px-2 py-0.5 rounded bg-default-100 text-default-600 font-medium">
              Formato: {formatDisplayName}
            </span>
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Formato da API</label>
          <Select
            aria-label="Formato da API"
            selectedKey={settings.apiFormat}
            onSelectionChange={handleFormatChange}
            className="w-full"
          >
            <Label className="sr-only">Formato da API</Label>
            <Select.Trigger>
              <Select.Value />
              <Select.Indicator />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                <ListBox.Item id="auto" key="auto" textValue="Auto-detectar">
                  Auto-detectar
                  <ListBox.ItemIndicator />
                </ListBox.Item>
                <ListBox.Item id="openai" key="openai" textValue="OpenAI Compatible">
                  OpenAI Compatible
                  <ListBox.ItemIndicator />
                </ListBox.Item>
                <ListBox.Item id="anthropic" key="anthropic" textValue="Anthropic">
                  Anthropic
                  <ListBox.ItemIndicator />
                </ListBox.Item>
              </ListBox>
            </Select.Popover>
          </Select>
          <p className="text-xs text-default-500 mt-1">
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
              aria-label="Selecionar modelo"
              selectedKey={settings.model || undefined}
              onSelectionChange={handleModelChange}
              className="flex-1"
              isDisabled={isLoading || models.length === 0}
              placeholder={isLoading ? 'Carregando...' : 'Selecione um modelo'}
            >
              <Label className="sr-only">Modelo</Label>
              <Select.Trigger>
                <Select.Value />
                <Select.Indicator />
              </Select.Trigger>
              <Select.Popover>
                <ListBox>
                  {filteredModels.length === 0 ? (
                    <ListBox.Item
                      id="no-results"
                      key="no-results"
                      textValue="Nenhum modelo encontrado"
                      isDisabled
                    >
                      Nenhum modelo encontrado
                    </ListBox.Item>
                  ) : (
                    filteredModels.map((model) => {
                      const displayName = model.name || model.id;
                      const contextInfo = model.context_length
                        ? ` (${Math.round(model.context_length / 1000)}k ctx)`
                        : '';
                      return (
                        <ListBox.Item id={model.id} key={model.id} textValue={displayName}>
                          <div className="flex flex-col">
                            <span>
                              {displayName}
                              {contextInfo}
                            </span>
                            {model.name && (
                              <span className="text-xs text-default-400">{model.id}</span>
                            )}
                          </div>
                          <ListBox.ItemIndicator />
                        </ListBox.Item>
                      );
                    })
                  )}
                </ListBox>
              </Select.Popover>
            </Select>
            <Button variant="ghost" onPress={refetch} isDisabled={isLoading}>
              {String.fromCodePoint(0x21bb)}
            </Button>
          </div>
          {error && <p className="text-xs text-danger mt-1">{error}</p>}
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Modelo Customizado (opcional)</label>
          <Input
            placeholder={detectedFormat === 'anthropic' ? 'claude-sonnet-4-5-20250929' : 'gpt-4-turbo'}
            value={settings.customModel}
            onChange={(e) => updateSetting('customModel', e.target.value)}
            className="w-full"
          />
          <p className="text-xs text-default-500 mt-1">
            Se preenchido, sera usado em vez do modelo selecionado acima
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Modelo para Deteccao de Idioma (opcional)</label>
          <Select
            aria-label="Modelo para deteccao de idioma"
            selectedKey={settings.languageDetectionModel || undefined}
            onSelectionChange={(key) => updateSetting('languageDetectionModel', key ? String(key) : '')}
            className="w-full"
            isDisabled={isLoading || models.length === 0}
            placeholder="Nenhum (usar configuracao manual)"
          >
            <Label className="sr-only">Modelo para Deteccao</Label>
            <Select.Trigger>
              <Select.Value />
              <Select.Indicator />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                <ListBox.Item id="" key="none" textValue="Nenhum">
                  Nenhum (usar configuracao manual)
                  <ListBox.ItemIndicator />
                </ListBox.Item>
                {models.map((model) => {
                  const displayName = model.name || model.id;
                  return (
                    <ListBox.Item id={model.id} key={model.id} textValue={displayName}>
                      <div className="flex flex-col">
                        <span>{displayName}</span>
                        {model.name && (
                          <span className="text-xs text-default-400">{model.id}</span>
                        )}
                      </div>
                      <ListBox.ItemIndicator />
                    </ListBox.Item>
                  );
                })}
              </ListBox>
            </Select.Popover>
          </Select>
          <p className="text-xs text-default-500 mt-1">
            Modelo leve para detectar o idioma da traducao e usar no mux (ex: gemma-3-1b)
          </p>
        </div>

        <Accordion.Root variant="surface">
          <Accordion.Item id="headers">
            <Accordion.Heading>
              <Accordion.Trigger>
                Headers Avancados
                <Accordion.Indicator />
              </Accordion.Trigger>
            </Accordion.Heading>
            <Accordion.Panel>
              <Accordion.Body>
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
                        variant="danger"
                        size="sm"
                        onPress={() => removeHeader(header.id)}
                      >
                        {String.fromCodePoint(0x2715)}
                      </Button>
                    </div>
                  ))}
                  <Button variant="primary" size="sm" onPress={addHeader}>
                    + Adicionar Header
                  </Button>
                </div>
              </Accordion.Body>
            </Accordion.Panel>
          </Accordion.Item>
        </Accordion.Root>
      </div>
    </Card>
  );
}
