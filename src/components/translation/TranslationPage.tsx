import { useTranslation } from 'react-i18next';
import { motion } from 'framer-motion';
import { FileText, Stack } from '@phosphor-icons/react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { FileDropZone } from './FileDropZone';
import { FileQueue } from './FileQueue';
import { SubtitleEditor } from './SubtitleEditor';
import { useTranslationStore } from '../../stores/translationStore';
import { useShallow } from 'zustand/shallow';

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.08,
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

export function TranslationPage() {
  const { t } = useTranslation();
  const { queue, currentFileId } = useTranslationStore(
    useShallow((s) => ({ queue: s.queue, currentFileId: s.currentFileId }))
  );
  const currentFile = queue.find((f) => f.id === currentFileId);

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="visible"
      className="space-y-6"
    >
      {/* Header */}
      <motion.div variants={itemVariants} className="space-y-1">
        <h1 className="text-headline">{t('translation.title')}</h1>
        <p className="text-caption">{t('translation.subtitle')}</p>
      </motion.div>

      <Tabs defaultValue="single" className="w-full">
        {/* Custom Tab Navigation */}
        <motion.div variants={itemVariants}>
          <TabsList className="bg-muted/50 p-1 rounded-xl h-auto gap-1">
            <TabsTrigger 
              value="single" 
              className="rounded-lg px-4 py-2.5 data-[state=active]:bg-background data-[state=active]:shadow-sm data-[state=active]:text-foreground text-muted-foreground transition-all"
            >
              <FileText className="w-4 h-4 mr-2" />
              {t('translation.mode.single')}
            </TabsTrigger>
            <TabsTrigger 
              value="batch"
              className="rounded-lg px-4 py-2.5 data-[state=active]:bg-background data-[state=active]:shadow-sm data-[state=active]:text-foreground text-muted-foreground transition-all"
            >
              <Stack className="w-4 h-4 mr-2" />
              {t('translation.mode.batch')}
            </TabsTrigger>
          </TabsList>
        </motion.div>

        <TabsContent value="single" className="mt-6">
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
            {/* Left column - Drop zone + Queue */}
            <motion.div 
              variants={itemVariants}
              className="lg:col-span-5 space-y-4"
            >
              <FileDropZone />
              {queue.length > 0 && (
                <motion.div
                  initial={{ opacity: 0, scale: 0.98 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ type: 'spring', stiffness: 100, damping: 20 }}
                >
                  <FileQueue maxVisible={1} />
                </motion.div>
              )}
            </motion.div>

            {/* Right column - Editor */}
            <div className="lg:col-span-7">
              <AnimatePresence mode="wait">
                {currentFile?.originalSubtitle ? (
                  <motion.div
                    key="editor"
                    variants={itemVariants}
                    initial="hidden"
                    animate="visible"
                    exit={{ opacity: 0, scale: 0.98 }}
                  >
                    <SubtitleEditor file={currentFile} />
                  </motion.div>
                ) : (
                  <motion.div
                    key="empty"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    className="h-full min-h-[400px] flex flex-col items-center justify-center rounded-2xl border border-dashed border-border/60 bg-muted/30"
                  >
                    <div className="w-16 h-16 rounded-2xl bg-muted flex items-center justify-center mb-4">
                      <FileText className="w-8 h-8 text-muted-foreground" />
                    </div>
                    <p className="text-muted-foreground text-sm">
                      {t('translation.editor.empty')}
                    </p>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          </div>
        </TabsContent>

        <TabsContent value="batch" className="mt-6">
          <motion.div 
            variants={containerVariants}
            initial="hidden"
            animate="visible"
            className="space-y-6"
          >
            <motion.div variants={itemVariants}>
              <FileDropZone />
            </motion.div>
            <motion.div variants={itemVariants}>
              <FileQueue showCancelAll />
            </motion.div>
          </motion.div>
        </TabsContent>
      </Tabs>
    </motion.div>
  );
}

import { AnimatePresence } from 'framer-motion';
