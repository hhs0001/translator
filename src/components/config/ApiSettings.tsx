import { Card, Input, Select, Label, ListBox, Accordion, Button } from '@heroui/react';
import { useSettingsStore } from '../../stores/settingsStore';
import { useModels } from '../../hooks/useModels';
import { Header } from '../../types';

export function ApiSettings() {
  const { settings, updateSetting } = useSettingsStore();
  const { models, isLoading, error, refetch } = useModels();

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

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Conexão com API</h3>

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
            O endpoint /chat/completions será adicionado automaticamente
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">API Key</label>
          <Input
            type="password"
            placeholder="sk-..."
            value={settings.apiKey}
            onChange={(e) => updateSetting('apiKey', e.target.value)}
            className="w-full"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Modelo</label>
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
                  {models.map((model) => (
                    <ListBox.Item id={model.id} key={model.id} textValue={model.id}>
                      {model.id}
                      <ListBox.ItemIndicator />
                    </ListBox.Item>
                  ))}
                </ListBox>
              </Select.Popover>
            </Select>
            <Button variant="ghost" onPress={refetch} isDisabled={isLoading}>
              ↻
            </Button>
          </div>
          {error && <p className="text-xs text-danger mt-1">{error}</p>}
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Modelo Customizado (opcional)</label>
          <Input
            placeholder="gpt-4-turbo"
            value={settings.customModel}
            onChange={(e) => updateSetting('customModel', e.target.value)}
            className="w-full"
          />
          <p className="text-xs text-default-500 mt-1">
            Se preenchido, será usado em vez do modelo selecionado acima
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Modelo para Detecção de Idioma (opcional)</label>
          <Select
            aria-label="Modelo para detecção de idioma"
            selectedKey={settings.languageDetectionModel || undefined}
            onSelectionChange={(key) => updateSetting('languageDetectionModel', key ? String(key) : '')}
            className="w-full"
            isDisabled={isLoading || models.length === 0}
            placeholder="Nenhum (usar configuração manual)"
          >
            <Label className="sr-only">Modelo para Detecção</Label>
            <Select.Trigger>
              <Select.Value />
              <Select.Indicator />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                <ListBox.Item id="" key="none" textValue="Nenhum">
                  Nenhum (usar configuração manual)
                  <ListBox.ItemIndicator />
                </ListBox.Item>
                {models.map((model) => (
                  <ListBox.Item id={model.id} key={model.id} textValue={model.id}>
                    {model.id}
                    <ListBox.ItemIndicator />
                  </ListBox.Item>
                ))}
              </ListBox>
            </Select.Popover>
          </Select>
          <p className="text-xs text-default-500 mt-1">
            Modelo leve para detectar o idioma da tradução e usar no mux (ex: gemma-3-1b)
          </p>
        </div>

        <Accordion.Root variant="surface">
          <Accordion.Item id="headers">
            <Accordion.Heading>
              <Accordion.Trigger>
                Headers Avançados
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
                        ✕
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
