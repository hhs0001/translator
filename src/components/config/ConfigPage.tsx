import { useTranslation } from 'react-i18next';
import { motion } from 'framer-motion';
import { 
  Gear,
  Key,
  Terminal,
  FileCode,
  ListChecks,
  FolderOpen,
  Translate,
  ArrowSquareOut
} from '@phosphor-icons/react';
import { ApiSettings } from './ApiSettings';
import { FfmpegStatus } from './FfmpegStatus';
import { PromptEditor } from './PromptEditor';
import { TemplateManager } from './TemplateManager';
import { TranslationSettings } from './TranslationSettings';
import { OutputSettings } from './OutputSettings';
import { LanguageSettings } from './LanguageSettings';
import { Button } from '@/components/ui/button';
import { getAppDataDir, openFolder } from '@/utils/tauri';

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.06,
      delayChildren: 0.1,
    },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 12 },
  visible: {
    opacity: 1,
    y: 0,
    transition: {
      type: 'spring' as const,
      stiffness: 100,
      damping: 20,
    },
  },
};

export function ConfigPage() {
  const { t } = useTranslation();

  const handleOpenConfigFolder = async () => {
    try {
      const path = await getAppDataDir();
      await openFolder(path);
    } catch (error) {
      console.error('Failed to open config folder:', error);
    }
  };

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="visible"
      className="max-w-5xl mx-auto space-y-8"
    >
      {/* Header */}
      <motion.div variants={itemVariants} className="space-y-1">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center">
            <Gear className="w-5 h-5 text-primary" />
          </div>
          <div>
            <h1 className="text-headline">{t('settings.title')}</h1>
            <p className="text-caption">{t('settings.subtitle')}</p>
          </div>
        </div>
      </motion.div>

      {/* Bento Grid Layout */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left Column - Core Settings */}
        <div className="lg:col-span-7 space-y-6">
          {/* API Settings - Large Card */}
          <motion.div variants={itemVariants}>
            <div className="card-bento">
              <div className="flex items-center gap-3 mb-5">
                <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
                  <Key className="w-4 h-4 text-primary" />
                </div>
                <h2 className="text-title">{t('settings.api.title')}</h2>
              </div>
              <ApiSettings />
            </div>
          </motion.div>

          {/* Language Settings */}
          <motion.div variants={itemVariants}>
            <div className="card-bento">
              <div className="flex items-center gap-3 mb-5">
                <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
                  <Translate className="w-4 h-4 text-primary" />
                </div>
                <h2 className="text-title">{t('settings.language.title')}</h2>
              </div>
              <LanguageSettings />
            </div>
          </motion.div>

          {/* Prompt Editor - Large Card */}
          <motion.div variants={itemVariants}>
            <div className="card-bento">
              <div className="flex items-center gap-3 mb-5">
                <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
                  <FileCode className="w-4 h-4 text-primary" />
                </div>
                <h2 className="text-title">{t('settings.prompt.title')}</h2>
              </div>
              <PromptEditor />
            </div>
          </motion.div>
        </div>

        {/* Right Column - Secondary Settings */}
        <div className="lg:col-span-5 space-y-6">
          {/* FFmpeg Status */}
          <motion.div variants={itemVariants}>
            <div className="card-bento">
              <div className="flex items-center gap-3 mb-5">
                <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
                  <Terminal className="w-4 h-4 text-primary" />
                </div>
                <h2 className="text-title">{t('settings.ffmpeg.title')}</h2>
              </div>
              <FfmpegStatus />
            </div>
          </motion.div>

          {/* Template Manager */}
          <motion.div variants={itemVariants}>
            <div className="card-bento">
              <div className="flex items-center gap-3 mb-5">
                <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
                  <ListChecks className="w-4 h-4 text-primary" />
                </div>
                <h2 className="text-title">{t('settings.templates.title')}</h2>
              </div>
              <TemplateManager />
            </div>
          </motion.div>

          {/* Translation Settings */}
          <motion.div variants={itemVariants}>
            <div className="card-bento">
              <div className="flex items-center gap-3 mb-5">
                <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
                  <Translate className="w-4 h-4 text-primary" />
                </div>
                <h2 className="text-title">{t('settings.translation.title')}</h2>
              </div>
              <TranslationSettings />
            </div>
          </motion.div>

          {/* Output Settings */}
          <motion.div variants={itemVariants}>
            <div className="card-bento">
              <div className="flex items-center gap-3 mb-5">
                <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
                  <FolderOpen className="w-4 h-4 text-primary" />
                </div>
                <h2 className="text-title">{t('settings.output.title')}</h2>
              </div>
              <OutputSettings />
            </div>
          </motion.div>

          {/* App Data */}
          <motion.div variants={itemVariants}>
            <div className="card-bento bg-muted/30">
              <div className="flex items-center gap-3 mb-4">
                <div className="w-8 h-8 rounded-lg bg-foreground/5 flex items-center justify-center">
                  <FolderOpen className="w-4 h-4 text-muted-foreground" />
                </div>
                <h2 className="text-title text-muted-foreground">{t('settings.appData.title')}</h2>
              </div>
              <Button 
                variant="outline" 
                onClick={handleOpenConfigFolder}
                className="w-full gap-2"
              >
                <ArrowSquareOut className="w-4 h-4" />
                {t('settings.appData.openConfigFolder')}
              </Button>
              <p className="text-xs text-muted-foreground mt-3">
                {t('settings.appData.configFilesHint')}
              </p>
            </div>
          </motion.div>
        </div>
      </div>
    </motion.div>
  );
}
