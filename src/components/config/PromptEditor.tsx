import { Card, TextArea, Select, Label, ListBox } from '@heroui/react';
import { useSettingsStore } from '../../stores/settingsStore';

export function PromptEditor() {
  const { settings, updateSetting, templates } = useSettingsStore();

  const applyTemplate = (key: React.Key | null) => {
    if (!key) return;
    const templateId = String(key);
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
              aria-label="Selecionar template"
              selectedKey={settings.selectedTemplateId || undefined}
              onSelectionChange={applyTemplate}
              className="flex-1"
              placeholder="Aplicar template..."
            >
              <Label>Template</Label>
              <Select.Trigger>
                <Select.Value />
                <Select.Indicator />
              </Select.Trigger>
              <Select.Popover>
                <ListBox>
                  {templates.map((template) => (
                    <ListBox.Item id={template.id} key={template.id} textValue={template.name}>
                      {template.name}
                      <ListBox.ItemIndicator />
                    </ListBox.Item>
                  ))}
                </ListBox>
              </Select.Popover>
            </Select>
          </div>
        )}

        <TextArea
          value={settings.prompt}
          onChange={(e) => updateSetting('prompt', e.target.value)}
          className="w-full min-h-[150px]"
          placeholder="Digite o prompt de tradução..."
        />

        <p className="text-xs text-default-500">
          Use um prompt claro que instrua o modelo a traduzir as legendas mantendo formatação e contexto.
        </p>
      </div>
    </Card>
  );
}
