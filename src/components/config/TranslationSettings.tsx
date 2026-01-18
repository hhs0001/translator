import { Card } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { useSettingsStore } from '../../stores/settingsStore';

const BATCH_PRESETS = [
  { value: 25, label: '25 linhas' },
  { value: 50, label: '50 linhas' },
  { value: 100, label: '100 linhas' },
  { value: 150, label: '150 linhas' },
];

export function TranslationSettings() {
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

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Configurações de Tradução</h3>

      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">Tamanho do Batch</label>
          <div className="flex gap-2">
            <Select
              value={String(settings.batchSize)}
              onValueChange={handleBatchSelect}
            >
              <Label className="sr-only">Tamanho</Label>
              <SelectTrigger className="flex-1">
                <SelectValue placeholder="Selecione" />
              </SelectTrigger>
              <SelectContent>
                {BATCH_PRESETS.map((preset) => (
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
          <p className="text-xs text-muted-foreground mt-1">
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
          <p className="text-xs text-muted-foreground mt-1">
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
          <div className="flex items-center gap-2">
            <Switch
              id="streaming"
              checked={settings.streaming && !streamingDisabled}
              onCheckedChange={(checked) => updateSetting('streaming', checked)}
              disabled={streamingDisabled}
            />
            <Label htmlFor="streaming" className={streamingDisabled ? 'text-muted-foreground' : ''}>
              Streaming (exibir traduções conforme chegam)
            </Label>
          </div>
          <p className="text-xs text-muted-foreground ml-10">
            {streamingDisabled
              ? 'Streaming não é suportado com a API Anthropic direta. Use uma API compatível com OpenAI.'
              : 'Quando habilitado, as traduções aparecem em tempo real conforme a API responde.'}
          </p>

          <div className="flex items-center gap-2">
            <Switch
              id="auto-continue"
              checked={settings.autoContinue}
              onCheckedChange={(checked) => updateSetting('autoContinue', checked)}
            />
            <Label htmlFor="auto-continue">Continuar automaticamente (respostas parciais)</Label>
          </div>

          <div className="flex items-center gap-2">
            <Switch
              id="continue-on-error"
              checked={settings.continueOnError}
              onCheckedChange={(checked) => updateSetting('continueOnError', checked)}
            />
            <Label htmlFor="continue-on-error">Continuar fila em caso de erro</Label>
          </div>
        </div>
      </div>
    </Card>
  );
}
