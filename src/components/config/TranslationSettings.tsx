import { Card, Input, Switch, Select, Label, ListBox } from '@heroui/react';
import { useSettingsStore } from '../../stores/settingsStore';

const BATCH_PRESETS = [
  { value: 25, label: '25 linhas' },
  { value: 50, label: '50 linhas' },
  { value: 100, label: '100 linhas' },
  { value: 150, label: '150 linhas' },
];

export function TranslationSettings() {
  const { settings, updateSetting } = useSettingsStore();

  const handleBatchSelect = (key: React.Key | null) => {
    if (key) {
      const value = Number(key);
      if (!isNaN(value) && value > 0) {
        updateSetting('batchSize', value);
      }
    }
  };

  const handleNumberInput = (field: 'batchSize' | 'parallelRequests' | 'concurrency' | 'maxRetries', value: string) => {
    const num = parseInt(value, 10);
    if (!isNaN(num) && num >= 0) {
      updateSetting(field, num);
    }
  };

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Configurações de Tradução</h3>

      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">Tamanho do Batch</label>
          <div className="flex gap-2">
            <Select
              aria-label="Tamanho do batch"
              selectedKey={String(settings.batchSize)}
              onSelectionChange={handleBatchSelect}
              className="flex-1"
              placeholder="Selecione"
            >
              <Label className="sr-only">Tamanho</Label>
              <Select.Trigger>
                <Select.Value />
                <Select.Indicator />
              </Select.Trigger>
              <Select.Popover>
                <ListBox>
                  {BATCH_PRESETS.map((preset) => (
                    <ListBox.Item id={String(preset.value)} key={String(preset.value)} textValue={preset.label}>
                      {preset.label}
                      <ListBox.ItemIndicator />
                    </ListBox.Item>
                  ))}
                </ListBox>
              </Select.Popover>
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
          <p className="text-xs text-default-500 mt-1">
            Quantidade de linhas enviadas por requisição
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Requisições Paralelas</label>
          <Input
            type="number"
            value={String(settings.parallelRequests || 1)}
            onChange={(e) => handleNumberInput('parallelRequests', e.target.value)}
            className="w-24"
            min={1}
            max={10}
          />
          <p className="text-xs text-default-500 mt-1">
            Número de batches enviados em paralelo por arquivo (ex: 4 x 50 linhas = 200 linhas simultâneas)
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Concorrência</label>
          <Input
            type="number"
            value={String(settings.concurrency || 1)}
            onChange={(e) => handleNumberInput('concurrency', e.target.value)}
            className="w-24"
            min={1}
            max={10}
          />
          <p className="text-xs text-default-500 mt-1">
            Número de arquivos processados simultaneamente
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Máximo de Retentativas</label>
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
          <Switch.Root
            isSelected={settings.autoContinue}
            onChange={(checked: boolean) => updateSetting('autoContinue', checked)}
          >
            <Switch.Control>
              <Switch.Thumb />
            </Switch.Control>
            <Label>Continuar automaticamente (respostas parciais)</Label>
          </Switch.Root>

          <Switch.Root
            isSelected={settings.continueOnError}
            onChange={(checked: boolean) => updateSetting('continueOnError', checked)}
          >
            <Switch.Control>
              <Switch.Thumb />
            </Switch.Control>
            <Label>Continuar fila em caso de erro</Label>
          </Switch.Root>
        </div>
      </div>
    </Card>
  );
}
