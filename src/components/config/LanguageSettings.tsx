import { useTranslation } from 'react-i18next';
import { Card } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useSettingsStore } from '../../stores/settingsStore';
import { changeLanguage, type Language } from '../../i18n';
import { Language as LanguageType } from '../../types';

const LANGUAGE_OPTIONS = [
  { value: 'en', label: 'English' },
  { value: 'pt-BR', label: 'Portugues (Brasil)' },
] as const;

export function LanguageSettings() {
  const { t, i18n } = useTranslation();
  const { updateSetting } = useSettingsStore();

  const handleLanguageChange = async (value: string) => {
    await changeLanguage(value as Language);
    updateSetting('language', value as LanguageType);
  };

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">{t('settings.language.title')}</h3>

      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">{t('settings.language.label')}</label>
          <Select
            value={i18n.language}
            onValueChange={handleLanguageChange}
          >
            <Label className="sr-only">{t('settings.language.label')}</Label>
            <SelectTrigger className="w-full max-w-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {LANGUAGE_OPTIONS.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground mt-1">
            {t('settings.language.hint')}
          </p>
        </div>
      </div>
    </Card>
  );
}
