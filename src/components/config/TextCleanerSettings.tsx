import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Card } from '@/components/ui/card';
import { 
  AlertTriangle, 
  Info,
  Plus,
  X
} from 'lucide-react';
import { useSettingsStore } from '@/stores/settingsStore';

export function TextCleanerSettings() {
  const { t } = useTranslation();
  const { settings, saveSettings } = useSettingsStore();
  const [newTag, setNewTag] = useState('');
  const [newStyle, setNewStyle] = useState('');

  const config = {
    enabled: settings.textCleanerEnabled,
    preserveBasicFormatting: settings.textCleanerPreserveBasicFormatting,
    tagsToRemove: settings.textCleanerTagsToRemove,
    ignoredStyles: settings.textCleanerIgnoredStyles,
  };

  const handleToggleEnabled = (checked: boolean) => {
    saveSettings({ textCleanerEnabled: checked });
  };

  const handleTogglePreserveBasic = (checked: boolean) => {
    saveSettings({ textCleanerPreserveBasicFormatting: checked });
  };

  const handleAddTag = () => {
    if (newTag.trim() && !config.tagsToRemove.includes(newTag.trim())) {
      saveSettings({
        textCleanerTagsToRemove: [...config.tagsToRemove, newTag.trim()]
      });
      setNewTag('');
    }
  };

  const handleRemoveTag = (tag: string) => {
    saveSettings({
      textCleanerTagsToRemove: config.tagsToRemove.filter(t => t !== tag)
    });
  };

  const handleAddStyle = () => {
    if (newStyle.trim() && !config.ignoredStyles.includes(newStyle.trim())) {
      saveSettings({
        textCleanerIgnoredStyles: [...config.ignoredStyles, newStyle.trim()]
      });
      setNewStyle('');
    }
  };

  const handleRemoveStyle = (style: string) => {
    saveSettings({
      textCleanerIgnoredStyles: config.ignoredStyles.filter(s => s !== style)
    });
  };

  return (
    <div className="space-y-6">
      {/* Alert Info */}
      <Card className="p-4 bg-amber-500/5 border-amber-500/20">
        <div className="flex items-start gap-3">
          <AlertTriangle className="w-5 h-5 text-amber-500 shrink-0 mt-0.5" />
          <div className="text-sm">
            <p className="font-medium text-amber-500 mb-1">{t('settings.textCleaner.whatItDoes')}</p>
            <p className="text-muted-foreground">
              {t('settings.textCleaner.whatItDoesDescription')}
            </p>
            <p className="text-muted-foreground mt-2">
              {t('settings.textCleaner.tokenSavings')}
            </p>
          </div>
        </div>
      </Card>

      {/* Enable Toggle */}
      <div className="flex items-center justify-between py-3 border-b border-border/50">
        <div className="space-y-0.5">
          <Label htmlFor="text-cleaner-enabled" className="font-medium">
            {t('settings.textCleaner.enabled')}
          </Label>
          <p className="text-xs text-muted-foreground">
            {t('settings.textCleaner.enabledHint')}
          </p>
        </div>
        <Switch
          id="text-cleaner-enabled"
          checked={config.enabled}
          onCheckedChange={handleToggleEnabled}
        />
      </div>

      {config.enabled && (
        <>
          {/* Preserve Basic Formatting */}
          <div className="flex items-center justify-between py-3 border-b border-border/50">
            <div className="space-y-0.5">
              <Label htmlFor="preserve-basic" className="font-medium">
                {t('settings.textCleaner.preserveBasic')}
              </Label>
              <p className="text-xs text-muted-foreground">
                {t('settings.textCleaner.preserveBasicHint')}
              </p>
            </div>
            <Switch
              id="preserve-basic"
              checked={config.preserveBasicFormatting}
              onCheckedChange={handleTogglePreserveBasic}
            />
          </div>

          {/* Ignored Styles */}
          <div className="space-y-3">
            <div className="flex items-center gap-2">
              <Label className="font-medium">{t('settings.textCleaner.ignoredStyles')}</Label>
              <Info className="w-4 h-4 text-muted-foreground" />
            </div>
            <p className="text-xs text-muted-foreground">
              {t('settings.textCleaner.ignoredStylesHint')}
            </p>
            
            <div className="flex flex-wrap gap-2">
              {config.ignoredStyles.map(style => (
                <Badge key={style} variant="secondary" className="gap-1">
                  {style}
                  <button
                    onClick={() => handleRemoveStyle(style)}
                    className="ml-1 hover:text-destructive"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </Badge>
              ))}
            </div>

            <div className="flex gap-2">
              <Input
                placeholder="Nome do estilo (ex: Title, Signs)"
                value={newStyle}
                onChange={(e) => setNewStyle(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleAddStyle()}
              />
              <Button variant="outline" size="icon" onClick={handleAddStyle}>
                <Plus className="w-4 h-4" />
              </Button>
            </div>
          </div>

          {/* Custom Tags to Remove */}
          <div className="space-y-3">
            <div className="flex items-center gap-2">
              <Label className="font-medium">{t('settings.textCleaner.customTags')}</Label>
              <Info className="w-4 h-4 text-muted-foreground" />
            </div>
            <p className="text-xs text-muted-foreground">
              {t('settings.textCleaner.customTagsHint')}
            </p>
            
            <div className="flex flex-wrap gap-2">
              {config.tagsToRemove.map(tag => (
                <Badge key={tag} variant="outline" className="gap-1">
                  \\{tag}
                  <button
                    onClick={() => handleRemoveTag(tag)}
                    className="ml-1 hover:text-destructive"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </Badge>
              ))}
            </div>

            <div className="flex gap-2">
              <Input
                placeholder="Nome da tag (ex: fn, fs)"
                value={newTag}
                onChange={(e) => setNewTag(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleAddTag()}
              />
              <Button variant="outline" size="icon" onClick={handleAddTag}>
                <Plus className="w-4 h-4" />
              </Button>
            </div>
          </div>

          {/* Visual Effects List */}
          <Card className="p-4">
            <h4 className="text-sm font-medium mb-3">{t('settings.textCleaner.visualTagsTitle')}</h4>
            <div className="flex flex-wrap gap-1.5">
              {[
                'pos', 'move', 'org', 'fad', 'blur', 'be', 
                'c', '1c', '2c', '3c', '4c', 'alpha',
                'frx', 'fry', 'frz', 'fscx', 'fscy',
                'fax', 'fay', 'fsp', 'bord', 'shad',
                'clip', 'iclip', 'p', 'pbo'
              ].map(tag => (
                <Badge key={tag} variant="secondary" className="text-xs">
                  \\{tag}
                </Badge>
              ))}
            </div>
            <p className="text-xs text-muted-foreground mt-3">
              {t('settings.textCleaner.visualTagsHint')}
            </p>
          </Card>
        </>
      )}
    </div>
  );
}
