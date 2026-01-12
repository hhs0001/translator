import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { QueueFile, FileStatus, SubtitleFile } from '../types';
import { useLogsStore } from './logsStore';
import { useSettingsStore } from './settingsStore';

interface TranslationState {
  queue: QueueFile[];
  currentFileId: string | null;
  isTranslating: boolean;
  isPaused: boolean;

  // Queue management
  addFiles: (files: { name: string; path: string; type: 'subtitle' | 'video' }[]) => void;
  removeFile: (id: string) => void;
  clearQueue: () => void;
  reorderQueue: (fromIndex: number, toIndex: number) => void;

  // File state
  updateFile: (id: string, updates: Partial<QueueFile>) => void;
  setFileStatus: (id: string, status: FileStatus, error?: string) => void;

  // Translation control
  startTranslation: () => Promise<void>;
  pauseTranslation: () => void;
  resumeTranslation: () => void;
  stopTranslation: () => void;

  // Internal
  processNextFile: () => Promise<void>;
  translateFile: (file: QueueFile) => Promise<void>;
}

export const useTranslationStore = create<TranslationState>((set, get) => ({
  queue: [],
  currentFileId: null,
  isTranslating: false,
  isPaused: false,

  addFiles: (files) => {
    const newFiles: QueueFile[] = files.map((f) => ({
      id: crypto.randomUUID(),
      name: f.name,
      path: f.path,
      type: f.type,
      status: 'pending',
      progress: 0,
      totalLines: 0,
      translatedLines: 0,
    }));
    set((state) => ({ queue: [...state.queue, ...newFiles] }));
    useLogsStore.getState().addLog('info', `${files.length} arquivo(s) adicionado(s) à fila`);
  },

  removeFile: (id) => {
    set((state) => ({ queue: state.queue.filter((f) => f.id !== id) }));
  },

  clearQueue: () => {
    set({ queue: [], currentFileId: null });
  },

  reorderQueue: (fromIndex, toIndex) => {
    set((state) => {
      const newQueue = [...state.queue];
      const [removed] = newQueue.splice(fromIndex, 1);
      newQueue.splice(toIndex, 0, removed);
      return { queue: newQueue };
    });
  },

  updateFile: (id, updates) => {
    set((state) => ({
      queue: state.queue.map((f) => (f.id === id ? { ...f, ...updates } : f)),
    }));
  },

  setFileStatus: (id, status, error) => {
    set((state) => ({
      queue: state.queue.map((f) =>
        f.id === id ? { ...f, status, error } : f
      ),
    }));
  },

  startTranslation: async () => {
    const { queue } = get();
    if (queue.length === 0) return;

    set({ isTranslating: true, isPaused: false });
    useLogsStore.getState().addLog('info', 'Iniciando tradução...');
    await get().processNextFile();
  },

  pauseTranslation: () => {
    set({ isPaused: true });
    useLogsStore.getState().addLog('warning', 'Tradução pausada');
  },

  resumeTranslation: () => {
    set({ isPaused: false });
    useLogsStore.getState().addLog('info', 'Tradução retomada');
    get().processNextFile();
  },

  stopTranslation: () => {
    set({ isTranslating: false, isPaused: false, currentFileId: null });
    useLogsStore.getState().addLog('warning', 'Tradução interrompida');
  },

  processNextFile: async () => {
    const { queue, isPaused, isTranslating } = get();
    if (!isTranslating || isPaused) return;

    const settings = useSettingsStore.getState().settings;
    const pendingFiles = queue.filter((f) => f.status === 'pending');

    if (pendingFiles.length === 0) {
      set({ isTranslating: false, currentFileId: null });
      useLogsStore.getState().addLog('success', 'Todos os arquivos foram processados!');
      return;
    }

    // Process up to `concurrency` files
    const filesToProcess = pendingFiles.slice(0, settings.concurrency);

    await Promise.all(
      filesToProcess.map((file) => get().translateFile(file))
    );

    // Continue with next batch
    if (get().isTranslating && !get().isPaused) {
      await get().processNextFile();
    }
  },

  translateFile: async (file) => {
    const { updateFile, setFileStatus } = get();
    const settings = useSettingsStore.getState().settings;
    const logs = useLogsStore.getState();

    try {
      set({ currentFileId: file.id });

      // Step 1: If video, extract subtitle
      let subtitlePath = file.path;
      if (file.type === 'video') {
        setFileStatus(file.id, 'extracting');
        logs.addLog('info', `Extraindo legenda de ${file.name}...`, file.name);

        const trackIndex = file.selectedTrackIndex ?? 0;
        subtitlePath = await invoke<string>('extract_subtitle_track', {
          videoPath: file.path,
          trackIndex,
        });
        updateFile(file.id, { extractedSubtitlePath: subtitlePath });
      }

      // Step 2: Load subtitle
      const subtitle = await invoke<SubtitleFile>('load_subtitle', {
        path: subtitlePath,
      });
      updateFile(file.id, {
        originalSubtitle: subtitle,
        totalLines: subtitle.entries.length,
      });

      // Step 3: Translate
      setFileStatus(file.id, 'translating');
      logs.addLog('info', `Traduzindo ${file.name} (${subtitle.entries.length} linhas)...`, file.name);

      const model = settings.customModel || settings.model;
      const result = await invoke<{
        translated_entries: typeof subtitle.entries;
        is_partial: boolean;
      }>('translate_subtitle_full', {
        subtitle,
        prompt: settings.prompt,
        baseUrl: settings.baseUrl,
        apiKey: settings.apiKey,
        model,
        batchSize: settings.batchSize,
        headers: settings.headers.reduce((acc, h) => ({ ...acc, [h.key]: h.value }), {}),
      });

      updateFile(file.id, {
        translatedEntries: result.translated_entries,
        translatedLines: result.translated_entries.length,
        progress: 100,
      });

      // Step 4: Handle partial
      if (result.is_partial && settings.autoContinue) {
        logs.addLog('warning', `Tradução parcial, continuando...`, file.name);
        // Continue translation logic here
      }

      setFileStatus(file.id, 'completed');
      logs.addLog('success', `${file.name} traduzido com sucesso!`, file.name);

    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      setFileStatus(file.id, 'error', errorMsg);
      logs.addLog('error', `Erro ao traduzir ${file.name}: ${errorMsg}`, file.name);

      if (!settings.continueOnError) {
        get().pauseTranslation();
      }
    }
  },
}));
