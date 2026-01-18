import { useTranslation } from 'react-i18next';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { useState } from 'react';
import { useSettingsStore } from '../../stores/settingsStore';

export function FfmpegStatus() {
  const { t } = useTranslation();
  const { ffmpegInstalled, checkFfmpeg } = useSettingsStore();
  const [isChecking, setIsChecking] = useState(false);

  const handleCheck = async () => {
    setIsChecking(true);
    await checkFfmpeg();
    setIsChecking(false);
  };

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">{t('settings.ffmpeg.title')}</h3>

      <div className="flex items-center gap-4">
        <Button onClick={handleCheck} disabled={isChecking}>
          {isChecking ? t('settings.ffmpeg.checking') : t('settings.ffmpeg.check')}
        </Button>

        {ffmpegInstalled !== null && (
          <Badge
            variant="outline"
            className={
              ffmpegInstalled
                ? 'border-transparent bg-emerald-500/15 text-emerald-700 dark:text-emerald-300'
                : 'border-transparent bg-destructive/15 text-destructive'
            }
          >
            {ffmpegInstalled ? t('settings.ffmpeg.installed') : t('settings.ffmpeg.notFound')}
          </Badge>
        )}
      </div>

      {ffmpegInstalled === false && (
        <Alert variant="destructive" className="mt-4">
          <AlertTitle>{t('settings.ffmpeg.notFoundTitle')}</AlertTitle>
          <AlertDescription>
            {t('settings.ffmpeg.notFoundDescription')}
          </AlertDescription>
        </Alert>
      )}

      <div className="mt-4 flex gap-2">
        <Badge
          variant="outline"
          className="border-transparent bg-emerald-500/15 text-emerald-700 dark:text-emerald-300"
        >
          SRT
        </Badge>
        <Badge
          variant="outline"
          className="border-transparent bg-emerald-500/15 text-emerald-700 dark:text-emerald-300"
        >
          ASS
        </Badge>
        <Badge
          variant="outline"
          className="border-transparent bg-emerald-500/15 text-emerald-700 dark:text-emerald-300"
        >
          SSA
        </Badge>
        <Badge
          variant="outline"
          className="border-transparent bg-amber-500/15 text-amber-700 dark:text-amber-300"
        >
          VTT ({t('settings.ffmpeg.comingSoon')})
        </Badge>
      </div>
      <p className="text-xs text-muted-foreground mt-2">
        {t('settings.ffmpeg.formatsHint')}
      </p>
    </Card>
  );
}
