import { Tabs } from '@heroui/react';
import { FileDropZone } from './FileDropZone';
import { FileQueue } from './FileQueue';
import { SubtitleEditor } from './SubtitleEditor';
import { useTranslationStore } from '../../stores/translationStore';

export function TranslationPage() {
  const { queue, currentFileId } = useTranslationStore();
  const currentFile = queue.find((f) => f.id === currentFileId);

  return (
    <div className="space-y-6">
      <Tabs.Root className="w-full">
        <Tabs.ListContainer>
          <Tabs.List aria-label="Modo de tradução">
            <Tabs.Tab id="single">
              Single
              <Tabs.Indicator />
            </Tabs.Tab>
            <Tabs.Tab id="batch">
              Batch
              <Tabs.Indicator />
            </Tabs.Tab>
          </Tabs.List>
        </Tabs.ListContainer>

        <Tabs.Panel id="single" className="pt-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <div className="space-y-4">
              <FileDropZone />
              {queue.length > 0 && <FileQueue maxVisible={1} />}
            </div>
            {currentFile?.originalSubtitle && (
              <SubtitleEditor file={currentFile} />
            )}
          </div>
        </Tabs.Panel>

        <Tabs.Panel id="batch" className="pt-4">
          <div className="space-y-6">
            <FileDropZone />
            <FileQueue />
          </div>
        </Tabs.Panel>
      </Tabs.Root>
    </div>
  );
}
