import { Card, Button, Chip, Alert } from '@heroui/react';
import { useSettingsStore } from '../../stores/settingsStore';

export function FfmpegStatus() {
  const { ffmpegInstalled, checkFfmpeg } = useSettingsStore();

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">FFmpeg</h3>

      <div className="flex items-center gap-4">
        <Button onPress={checkFfmpeg}>
          Verificar FFmpeg
        </Button>

        {ffmpegInstalled !== null && (
          <Chip color={ffmpegInstalled ? 'success' : 'danger'}>
            {ffmpegInstalled ? 'Instalado' : 'Não encontrado'}
          </Chip>
        )}
      </div>

      {ffmpegInstalled === false && (
        <Alert color="danger" className="mt-4">
          <Alert.Title>FFmpeg não encontrado</Alert.Title>
          <Alert.Description>
            O FFmpeg é necessário para extrair legendas de vídeos.
            Instale-o e adicione ao PATH do sistema.
          </Alert.Description>
        </Alert>
      )}

      <div className="mt-4 flex gap-2">
        <Chip color="success" variant="soft">SRT</Chip>
        <Chip color="success" variant="soft">ASS</Chip>
        <Chip color="success" variant="soft">SSA</Chip>
        <Chip color="warning" variant="soft">VTT (em breve)</Chip>
      </div>
      <p className="text-xs text-default-500 mt-2">
        ASS/SSA preservam melhor estilos e formatação
      </p>
    </Card>
  );
}
