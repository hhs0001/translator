import { useTranslation } from 'react-i18next';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { FileDropZone } from './FileDropZone';
import { FileQueue } from './FileQueue';
import { SubtitleEditor } from './SubtitleEditor';
import { useTranslationStore } from '../../stores/translationStore';
import { useShallow } from 'zustand/shallow';

export function TranslationPage() {
  const { t } = useTranslation();
  const { queue, currentFileId } = useTranslationStore(
    useShallow((s) => ({ queue: s.queue, currentFileId: s.currentFileId }))
  );
  const currentFile = queue.find((f) => f.id === currentFileId);

  return (
    <div className="space-y-6">
      <Tabs defaultValue="single" className="w-full">
        <TabsList aria-label={t('translation.mode.single')}>
          <TabsTrigger value="single">{t('translation.mode.single')}</TabsTrigger>
          <TabsTrigger value="batch">{t('translation.mode.batch')}</TabsTrigger>
        </TabsList>

        <TabsContent value="single" className="pt-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <div className="space-y-4">
              <FileDropZone />
              {queue.length > 0 && <FileQueue maxVisible={1} />}
            </div>
            {currentFile?.originalSubtitle && (
              <SubtitleEditor file={currentFile} />
            )}
          </div>
        </TabsContent>

        <TabsContent value="batch" className="pt-4">
          <div className="space-y-6">
            <FileDropZone />
            <FileQueue />
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
