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
              onSelectionChange={(key) => updateSetting('batchSize', Number(key))}
              className="flex-1"
              placeholder="Selecione"
            >
              <Label>Tamanho</Label>
              <Select.Trigger>
                <Select.Value />
              </Select.Trigger>
              <Select.Popover>
                <ListBox>
                  {BATCH_PRESETS.map((preset) => (
                    <ListBox.Item key={String(preset.value)} textValue={preset.label}>
                      {preset.label}
                    </ListBox.Item>
                  ))}
                </ListBox>
              </Select.Popover>
            </Select>
            <Input
              type="number"
              value={String(settings.batchSize)}
              onChange={(e) => updateSetting('batchSize', Number(e.target.value))}
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
          <label className="block text-sm font-medium mb-1">Concorrência</label>
          <Input
            type="number"
            value={String(settings.concurrency)}
            onChange={(e) => updateSetting('concurrency', Number(e.target.value))}
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
            value={String(settings.maxRetries)}
            onChange={(e) => updateSetting('maxRetries', Number(e.target.value))}
            className="w-24"
            min={0}
            max={10}
          />
        </div>

        <div className="space-y-3 pt-2">
          <Switch
            isSelected={settings.autoContinue}
            onChange={(checked) => updateSetting('autoContinue', checked)}
          >
            Continuar automaticamente (respostas parciais)
          </Switch>

          <Switch
            isSelected={settings.continueOnError}
            onChange={(checked) => updateSetting('continueOnError', checked)}
          >
            Continuar fila em caso de erro
          </Switch>
        </div>
      </div>
    </Card>
  );
}
