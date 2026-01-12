import { Card, Input, Select, Label, ListBox, Button } from '@heroui/react';
import { useSettingsStore } from '../../stores/settingsStore';

export function OutputSettings() {
  const { settings, updateSetting } = useSettingsStore();

  const selectOutputDir = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const dir = await open({
        directory: true,
        multiple: false,
      });
      if (dir) {
        updateSetting('separateOutputDir', dir as string);
      }
    } catch (err) {
      console.error('Erro ao abrir diálogo:', err);
    }
  };

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Configurações de Saída</h3>

      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">Modo de Saída</label>
          <Select
            aria-label="Modo de saída"
            selectedKey={settings.outputMode}
            onSelectionChange={(key) => updateSetting('outputMode', key as 'mux' | 'separate')}
            placeholder="Selecione o modo"
          >
            <Label>Modo</Label>
            <Select.Trigger>
              <Select.Value />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                <ListBox.Item key="separate" textValue="Arquivo Separado">
                  Arquivo Separado
                </ListBox.Item>
                <ListBox.Item key="mux" textValue="Mux no Vídeo (MKV)">
                  Mux no Vídeo (MKV)
                </ListBox.Item>
              </ListBox>
            </Select.Popover>
          </Select>
        </div>

        {settings.outputMode === 'separate' && (
          <div>
            <label className="block text-sm font-medium mb-1">Pasta de Saída</label>
            <div className="flex gap-2">
              <Input
                value={settings.separateOutputDir}
                onChange={(e) => updateSetting('separateOutputDir', e.target.value)}
                placeholder="Mesma pasta do original"
                className="flex-1"
              />
              <Button variant="ghost" onPress={selectOutputDir}>
                Selecionar
              </Button>
            </div>
          </div>
        )}

        {settings.outputMode === 'mux' && (
          <>
            <div>
              <label className="block text-sm font-medium mb-1">Código do Idioma</label>
              <Input
                value={settings.muxLanguage}
                onChange={(e) => updateSetting('muxLanguage', e.target.value)}
                placeholder="por"
                className="w-32"
              />
              <p className="text-xs text-default-500 mt-1">
                Código ISO 639-2 (ex: por, eng, spa)
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium mb-1">Título da Faixa</label>
              <Input
                value={settings.muxTitle}
                onChange={(e) => updateSetting('muxTitle', e.target.value)}
                placeholder="Portuguese"
                className="w-48"
              />
            </div>
          </>
        )}

        <div className="p-3 bg-default-100 rounded-lg">
          <p className="text-sm">
            <span className="font-medium">Arquivos originais:</span> Preservados
          </p>
          <p className="text-xs text-default-500 mt-1">
            {settings.outputMode === 'mux' 
              ? 'Legendas salvas como .translated.ass e vídeo muxado como .muxed.mkv'
              : 'Legendas traduzidas salvas como .translated.ass'}
          </p>
        </div>
      </div>
    </Card>
  );
}
