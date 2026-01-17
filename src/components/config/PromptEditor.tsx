import { Card } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { useSettingsStore } from '../../stores/settingsStore';

export function PromptEditor() {
  const { settings, updateSetting, templates } = useSettingsStore();

  const applyTemplate = (value: string) => {
    if (!value) return;
    const templateId = value;
    const template = templates.find((t) => t.id === templateId);
    if (template) {
      updateSetting('prompt', template.content);
      updateSetting('selectedTemplateId', templateId);
    }
  };

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Prompt de Tradução</h3>

      <div className="space-y-4">
        {templates.length > 0 && (
          <div className="flex gap-2">
            <Select
              value={settings.selectedTemplateId || ''}
              onValueChange={applyTemplate}
            >
              <Label className="sr-only">Template</Label>
              <SelectTrigger className="flex-1">
                <SelectValue placeholder="Aplicar template..." />
              </SelectTrigger>
              <SelectContent>
                {templates.map((template) => (
                  <SelectItem key={template.id} value={template.id}>
                    {template.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}

        <Textarea
          value={settings.prompt}
          onChange={(e) => updateSetting('prompt', e.target.value)}
          className="w-full min-h-[150px]"
          placeholder="Digite o prompt de tradução..."
        />

        <p className="text-xs text-muted-foreground">
          Use um prompt claro que instrua o modelo a traduzir as legendas mantendo formatação e contexto.
        </p>
      </div>
    </Card>
  );
}
