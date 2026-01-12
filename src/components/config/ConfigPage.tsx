import { ApiSettings } from './ApiSettings';
import { FfmpegStatus } from './FfmpegStatus';
import { PromptEditor } from './PromptEditor';
import { TemplateManager } from './TemplateManager';
import { TranslationSettings } from './TranslationSettings';
import { OutputSettings } from './OutputSettings';

export function ConfigPage() {
  return (
    <div className="space-y-6 max-w-4xl mx-auto">
      <h2 className="text-2xl font-bold">Configurações</h2>

      <div className="grid gap-6">
        <ApiSettings />
        <FfmpegStatus />
        <PromptEditor />
        <TemplateManager />
        <TranslationSettings />
        <OutputSettings />
      </div>
    </div>
  );
}
