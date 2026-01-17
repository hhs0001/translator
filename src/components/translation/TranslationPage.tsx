import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { FileDropZone } from './FileDropZone';
import { FileQueue } from './FileQueue';
import { SubtitleEditor } from './SubtitleEditor';
import { useTranslationStore } from '../../stores/translationStore';

export function TranslationPage() {
  const { queue, currentFileId } = useTranslationStore();
  const currentFile = queue.find((f) => f.id === currentFileId);

  return (
    <div className="space-y-6">
      <Tabs defaultValue="single" className="w-full">
        <TabsList aria-label="Modo de tradução">
          <TabsTrigger value="single">Single</TabsTrigger>
          <TabsTrigger value="batch">Batch</TabsTrigger>
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
