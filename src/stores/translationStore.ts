import { create } from 'zustand';
import { QueueFile, FileStatus } from '../types';
import { useLogsStore } from './logsStore';
import { useSettingsStore } from './settingsStore';
import * as TauriUtils from '../utils/tauri';
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';

interface TranslationState {
  queue: QueueFile[];
  currentFileId: string | null;
  isTranslating: boolean;
  isPaused: boolean;

  addFiles: (files: { name: string; path: string; type: 'subtitle' | 'video' }[]) => void;
  removeFile: (id: string) => void;
  clearQueue: () => void;
  reorderQueue: (fromIndex: number, toIndex: number) => void;

  updateFile: (id: string, updates: Partial<QueueFile>) => void;
  setFileStatus: (id: string, status: FileStatus, error?: string) => void;
  setSelectedTrack: (id: string, trackIndex: number) => void;
  setAllVideoTracks: (trackIndex: number) => void;
  loadVideoTracks: (id: string) => Promise<void>;

  startTranslation: () => Promise<void>;
  pauseTranslation: () => void;
  resumeTranslation: () => void;
  stopTranslation: () => void;

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
      isLoadingTracks: f.type === 'video',
    }));
    set((state) => ({ queue: [...state.queue, ...newFiles] }));
    useLogsStore.getState().addLog('info', `${files.length} arquivo(s) adicionado(s) à fila`);
    
    // Carregar faixas de legenda para vídeos
    newFiles.filter(f => f.type === 'video').forEach(f => {
      get().loadVideoTracks(f.id);
    });
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

  setSelectedTrack: (id, trackIndex) => {
    set((state) => ({
      queue: state.queue.map((f) =>
        f.id === id ? { ...f, selectedTrackIndex: trackIndex } : f
      ),
    }));
  },

  setAllVideoTracks: (trackIndex) => {
    set((state) => ({
      queue: state.queue.map((f) =>
        f.type === 'video' ? { ...f, selectedTrackIndex: trackIndex } : f
      ),
    }));
    useLogsStore.getState().addLog('info', `Faixa ${trackIndex + 1} selecionada para todos os vídeos`);
  },

  loadVideoTracks: async (id) => {
    const { queue, updateFile } = get();
    const file = queue.find((f) => f.id === id);
    if (!file || file.type !== 'video') return;

    updateFile(id, { isLoadingTracks: true });
    try {
      const tracks = await TauriUtils.listVideoSubtitleTracks(file.path);
      updateFile(id, { 
        subtitleTracks: tracks, 
        isLoadingTracks: false,
        selectedTrackIndex: tracks.length > 0 ? 0 : undefined
      });
    } catch (error) {
      console.error('Failed to load tracks:', error);
      updateFile(id, { subtitleTracks: [], isLoadingTracks: false });
    }
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

    const filesToProcess = pendingFiles.slice(0, settings.concurrency);

    await Promise.all(
      filesToProcess.map((file) => get().translateFile(file))
    );

    if (get().isTranslating && !get().isPaused) {
      await get().processNextFile();
    }
  },

  translateFile: async (file) => {
    const { updateFile, setFileStatus } = get();
    const settings = useSettingsStore.getState().settings;
    const logs = useLogsStore.getState();
    const headersObj = settings.headers.reduce((acc, h) => ({ ...acc, [h.key]: h.value }), {} as Record<string, string>);

    try {
      set({ currentFileId: file.id });

      let subtitlePath = file.path;
      if (file.type === 'video') {
        setFileStatus(file.id, 'extracting');
        logs.addLog('info', `Extraindo legenda de ${file.name}...`, file.name);

        const trackIndex = file.selectedTrackIndex ?? 0;
        // Gera o path de saída baseado no vídeo original
        const outputSubtitlePath = file.path.replace(/\.[^/.]+$/, '.extracted.ass');
        await TauriUtils.extractSubtitleTrack(file.path, trackIndex, outputSubtitlePath);
        subtitlePath = outputSubtitlePath;
        updateFile(file.id, { extractedSubtitlePath: subtitlePath });
      }

      const subtitle = await TauriUtils.loadSubtitle(subtitlePath);
      updateFile(file.id, {
        originalSubtitle: subtitle,
        totalLines: subtitle.entries.length,
      });

      // Detectar idioma alvo antes de traduzir (se modelo de detecção configurado)
      let muxLanguage = settings.muxLanguage;
      let muxTitle = settings.muxTitle;

      if (settings.languageDetectionModel) {
        setFileStatus(file.id, 'detecting_language');
        logs.addLog('info', `Detectando idioma alvo do prompt...`, file.name);

        try {
          const detected = await TauriUtils.detectLanguage(
            settings.baseUrl,
            settings.apiKey,
            settings.languageDetectionModel,
            settings.prompt,
            headersObj
          );

          updateFile(file.id, { detectedLanguage: detected });
          muxLanguage = detected.code;
          muxTitle = detected.displayName;
          logs.addLog('info', `Idioma alvo detectado: ${detected.displayName}`, file.name);
        } catch (langError) {
          logs.addLog('warning', `Não foi possível detectar idioma: ${langError}. Usando configuração padrão.`, file.name);
        }
      }

      setFileStatus(file.id, 'translating');
      logs.addLog('info', `Traduzindo ${file.name} (${subtitle.entries.length} linhas)...`, file.name);

      const model = settings.customModel || settings.model;
      const result = await TauriUtils.translateSubtitleFull(
        subtitle,
        settings.prompt,
        settings.baseUrl,
        settings.apiKey,
        model,
        headersObj,
        file.id,
        {
          batchSize: settings.batchSize,
          parallelRequests: settings.parallelRequests,
          autoContinue: settings.autoContinue,
          continueOnError: settings.continueOnError,
          maxRetries: settings.maxRetries,
        }
      );

      updateFile(file.id, {
        translatedEntries: result.file.entries,
        translatedLines: result.progress.translatedEntries,
        progress: 100,
      });

      if (result.progress.isPartial && settings.autoContinue) {
        logs.addLog('warning', `Tradução parcial, continuando...`, file.name);
      }

      // Salvar arquivo de legenda traduzido
      setFileStatus(file.id, 'saving');
      
      // Determina o path de saída
      let outputSubtitlePath: string;
      if (settings.separateOutputDir) {
        const fileName = file.name.replace(/\.[^/.]+$/, '.translated.ass');
        outputSubtitlePath = `${settings.separateOutputDir}/${fileName}`;
      } else {
        outputSubtitlePath = subtitlePath.replace(/\.[^/.]+$/, '.translated.ass');
      }
      
      await TauriUtils.saveSubtitle(outputSubtitlePath, result.file);
      updateFile(file.id, { outputSubtitlePath });
      logs.addLog('info', `Legenda salva em: ${outputSubtitlePath}`, file.name);

      // Fazer mux se configurado e for um vídeo
      if (settings.outputMode === 'mux' && file.type === 'video') {
        setFileStatus(file.id, 'muxing');
        logs.addLog('info', `Fazendo mux de ${file.name}...`, file.name);
        
        const outputVideoPath = file.path.replace(/\.[^/.]+$/, '.muxed.mkv');
        
        await TauriUtils.muxSubtitleToVideo(
          file.path,
          outputSubtitlePath,
          outputVideoPath,
          muxLanguage,
          muxTitle
        );
        
        updateFile(file.id, { outputVideoPath });
        logs.addLog('success', `Vídeo muxado salvo em: ${outputVideoPath}`, file.name);
      }

      setFileStatus(file.id, 'completed');
      logs.addLog('success', `${file.name} processado com sucesso!`, file.name);

    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      setFileStatus(file.id, 'error', errorMsg);
      logs.addLog('error', `Erro ao processar ${file.name}: ${errorMsg}`, file.name);

      if (!settings.continueOnError) {
        get().pauseTranslation();
      }
    }
  },
}));

export function useTranslationEvents() {
  useEffect(() => {
    let unlistenProgress: (() => void) | null = null;
    let unlistenError: (() => void) | null = null;

    const setupListeners = async () => {
      try {
        unlistenProgress = await listen<{ fileId: string; progress: number; translated: number; total: number }>(
          'translation:progress',
          (event) => {
            const { fileId, progress, translated, total } = event.payload;
            const store = useTranslationStore.getState();
            store.updateFile(fileId, { progress, translatedLines: translated, totalLines: total });
          }
        );

        unlistenError = await listen<{ fileId: string; error: string; retryCount: number }>(
          'translation:error',
          (event) => {
            const { fileId, error, retryCount } = event.payload;
            const file = useTranslationStore.getState().queue.find(f => f.id === fileId);
            const fileName = file?.name || 'arquivo';
            useLogsStore.getState().addLog('warning', `Erro em ${fileName} (tentativa ${retryCount}): ${error}`, fileName);
          }
        );
      } catch (error) {
        console.error('Failed to setup translation event listeners:', error);
      }
    };

    setupListeners();

    return () => {
      unlistenProgress?.();
      unlistenError?.();
    };
  }, []);
}
