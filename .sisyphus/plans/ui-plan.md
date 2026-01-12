# Plano de Implementação UI - Tradutor de Legendas (Tauri)

> **Objetivo**: Este documento é um guia completo para implementar a UI do tradutor de legendas. Um agente de IA deve conseguir seguir este plano do início ao fim sem precisar de esclarecimentos adicionais.

---

## Índice

1. [Visão Geral](#1-visão-geral)
2. [Estrutura de Arquivos](#2-estrutura-de-arquivos)
3. [Tipos TypeScript](#3-tipos-typescript)
4. [Estado Global (Zustand)](#4-estado-global-zustand)
5. [Hooks Customizados](#5-hooks-customizados)
6. [Componentes - Implementação Detalhada](#6-componentes---implementação-detalhada)
7. [Integração com Tauri](#7-integração-com-tauri)
8. [Fluxos de Usuário](#8-fluxos-de-usuário)
9. [Checklist de Implementação](#9-checklist-de-implementação)

---

## 1. Visão Geral

### 1.1 Tecnologias
- **React 19** + **Vite** + **TypeScript**
- **HeroUI v3** (componentes)
- **Tailwind CSS v4** (estilos)
- **Zustand** (estado global)
- **Tauri v2** (backend)

### 1.2 Telas Principais
1. **Tradução** (default) - drag-and-drop, fila, editor split-view
2. **Configurações** - API, FFmpeg, prompt, templates, saída

### 1.3 Layout Base
```
┌─────────────────────────────────────────────────────────┐
│ Navbar: [Logo] [Tradução] [Configs]  [Status] [Logs]    │
├─────────────────────────────────────────────────────────┤
│                                                         │
│                    Conteúdo da Tela                     │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## 2. Estrutura de Arquivos

Criar a seguinte estrutura dentro de `src/`:

```
src/
├── App.tsx                          # Layout principal com Tabs
├── App.css                          # Já configurado (não alterar)
├── main.tsx                         # Entry point (não alterar)
│
├── types/
│   └── index.ts                     # Todos os tipos TypeScript
│
├── stores/
│   ├── settingsStore.ts             # Estado de configurações
│   ├── translationStore.ts          # Estado de tradução/fila
│   └── logsStore.ts                 # Estado de logs
│
├── hooks/
│   ├── useModels.ts                 # Buscar modelos da API
│   ├── useTauri.ts                  # Wrapper dos comandos Tauri
│   └── useFileHandler.ts            # Processar arquivos dropped
│
├── components/
│   ├── layout/
│   │   ├── Navbar.tsx               # Barra superior
│   │   ├── LogsDrawer.tsx           # Drawer lateral de logs
│   │   └── MainLayout.tsx           # Wrapper com navbar + content
│   │
│   ├── translation/
│   │   ├── TranslationPage.tsx      # Página principal de tradução
│   │   ├── FileDropZone.tsx         # Área de drag-and-drop
│   │   ├── FileQueue.tsx            # Lista de arquivos na fila
│   │   ├── FileQueueItem.tsx        # Item individual da fila
│   │   ├── SubtitleEditor.tsx       # Editor split-view
│   │   ├── SubtitleLine.tsx         # Linha individual (original + tradução)
│   │   └── TrackSelector.tsx        # Modal para selecionar faixa de vídeo
│   │
│   ├── config/
│   │   ├── ConfigPage.tsx           # Página de configurações
│   │   ├── ApiSettings.tsx          # Card: conexão API
│   │   ├── FfmpegStatus.tsx         # Card: status FFmpeg
│   │   ├── PromptEditor.tsx         # Card: prompt de tradução
│   │   ├── TemplateManager.tsx      # Card: CRUD de templates
│   │   ├── TranslationSettings.tsx  # Card: batch size, retries, etc
│   │   └── OutputSettings.tsx       # Card: modo de saída
│   │
│   └── common/
│       ├── StatusChip.tsx           # Chip de status reutilizável
│       └── ConfirmModal.tsx         # Modal de confirmação genérico
│
└── utils/
    ├── tauri.ts                     # Helpers para invocar comandos
    └── format.ts                    # Formatação de texto/tempo
```

---

## 3. Tipos TypeScript

**Arquivo**: `src/types/index.ts`

```typescript
// ============================================
// SETTINGS
// ============================================

export interface AppSettings {
  // API
  baseUrl: string;
  apiKey: string;
  headers: Header[];
  model: string;
  customModel: string;

  // Prompt
  prompt: string;
  selectedTemplateId: string | null;

  // Tradução
  batchSize: number;
  autoContinue: boolean;
  continueOnError: boolean;
  maxRetries: number;
  concurrency: number;

  // Saída
  outputMode: 'mux' | 'separate';
  muxLanguage: string;
  muxTitle: string;
  separateOutputDir: string;
}

export interface Header {
  id: string;
  key: string;
  value: string;
}

export const DEFAULT_SETTINGS: AppSettings = {
  baseUrl: 'http://localhost:8045/v1',
  apiKey: '',
  headers: [],
  model: '',
  customModel: '',
  prompt: 'Translate the following subtitle lines to Brazilian Portuguese. Keep the same tone and style. Return only the translations, one per line, in the same order.',
  selectedTemplateId: null,
  batchSize: 50,
  autoContinue: true,
  continueOnError: true,
  maxRetries: 3,
  concurrency: 1,
  outputMode: 'separate',
  muxLanguage: 'por',
  muxTitle: 'Portuguese',
  separateOutputDir: '',
};

// ============================================
// TEMPLATES
// ============================================

export interface Template {
  id: string;
  name: string;
  content: string;
}

// ============================================
// MODELS
// ============================================

export interface LLMModel {
  id: string;
  name: string;
}

// ============================================
// SUBTITLES
// ============================================

export type SubtitleFormat = 'srt' | 'ass' | 'ssa' | 'vtt' | 'unknown';

export interface SubtitleEntry {
  index: number;
  start_time: string;
  end_time: string;
  text: string;
  style?: string;        // Para ASS/SSA
  actor?: string;        // Para ASS/SSA
  margin_l?: number;
  margin_r?: number;
  margin_v?: number;
  effect?: string;
}

export interface SubtitleFile {
  format: SubtitleFormat;
  entries: SubtitleEntry[];
  header?: string;       // Header do ASS/SSA
  styles?: string;       // Estilos do ASS/SSA
}

// ============================================
// TRANSLATION QUEUE
// ============================================

export type FileStatus = 'pending' | 'extracting' | 'translating' | 'paused' | 'completed' | 'error';

export interface QueueFile {
  id: string;
  name: string;
  path: string;
  type: 'subtitle' | 'video';
  status: FileStatus;
  progress: number;          // 0-100
  totalLines: number;
  translatedLines: number;
  error?: string;

  // Dados carregados
  originalSubtitle?: SubtitleFile;
  translatedEntries?: SubtitleEntry[];

  // Para vídeos
  selectedTrackIndex?: number;
  extractedSubtitlePath?: string;
}

export interface TranslationState {
  queue: QueueFile[];
  currentFileId: string | null;
  isTranslating: boolean;
  isPaused: boolean;
}

// ============================================
// LOGS
// ============================================

export type LogLevel = 'info' | 'warning' | 'error' | 'success';

export interface LogEntry {
  id: string;
  timestamp: Date;
  level: LogLevel;
  message: string;
  file?: string;
  details?: string;
}

// ============================================
// VIDEO TRACKS
// ============================================

export interface SubtitleTrack {
  index: number;
  codec: string;
  language?: string;
  title?: string;
}
```

---

## 4. Estado Global (Zustand)

### 4.1 Settings Store

**Arquivo**: `src/stores/settingsStore.ts`

```typescript
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { AppSettings, DEFAULT_SETTINGS, Template } from '../types';

interface SettingsState {
  settings: AppSettings;
  templates: Template[];
  isLoading: boolean;
  ffmpegInstalled: boolean | null;

  // Actions
  loadSettings: () => Promise<void>;
  saveSettings: (settings: Partial<AppSettings>) => Promise<void>;
  updateSetting: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => void;

  // Templates
  loadTemplates: () => Promise<void>;
  addTemplate: (name: string, content: string) => Promise<void>;
  updateTemplate: (id: string, name: string, content: string) => Promise<void>;
  deleteTemplate: (id: string) => Promise<void>;

  // FFmpeg
  checkFfmpeg: () => Promise<boolean>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  templates: [],
  isLoading: true,
  ffmpegInstalled: null,

  loadSettings: async () => {
    try {
      const settings = await invoke<AppSettings>('load_settings');
      set({ settings: { ...DEFAULT_SETTINGS, ...settings }, isLoading: false });
    } catch (error) {
      console.error('Failed to load settings:', error);
      set({ isLoading: false });
    }
  },

  saveSettings: async (newSettings) => {
    const merged = { ...get().settings, ...newSettings };
    set({ settings: merged });
    try {
      await invoke('save_settings', { settings: merged });
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  },

  updateSetting: (key, value) => {
    const newSettings = { ...get().settings, [key]: value };
    set({ settings: newSettings });
    // Debounce save
    invoke('save_settings', { settings: newSettings }).catch(console.error);
  },

  loadTemplates: async () => {
    try {
      const templates = await invoke<Template[]>('load_templates');
      set({ templates });
    } catch (error) {
      console.error('Failed to load templates:', error);
    }
  },

  addTemplate: async (name, content) => {
    try {
      await invoke('add_template', { name, content });
      await get().loadTemplates();
    } catch (error) {
      console.error('Failed to add template:', error);
      throw error;
    }
  },

  updateTemplate: async (id, name, content) => {
    try {
      await invoke('update_template', { id, name, content });
      await get().loadTemplates();
    } catch (error) {
      console.error('Failed to update template:', error);
      throw error;
    }
  },

  deleteTemplate: async (id) => {
    try {
      await invoke('delete_template', { id });
      await get().loadTemplates();
    } catch (error) {
      console.error('Failed to delete template:', error);
      throw error;
    }
  },

  checkFfmpeg: async () => {
    try {
      const installed = await invoke<boolean>('check_ffmpeg_installed');
      set({ ffmpegInstalled: installed });
      return installed;
    } catch (error) {
      set({ ffmpegInstalled: false });
      return false;
    }
  },
}));
```

### 4.2 Translation Store

**Arquivo**: `src/stores/translationStore.ts`

```typescript
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { QueueFile, FileStatus, SubtitleFile, SubtitleEntry } from '../types';
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
        translated_entries: SubtitleEntry[];
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
```

### 4.3 Logs Store

**Arquivo**: `src/stores/logsStore.ts`

```typescript
import { create } from 'zustand';
import { LogEntry, LogLevel } from '../types';

interface LogsState {
  logs: LogEntry[];
  isOpen: boolean;
  filter: LogLevel | 'all';

  addLog: (level: LogLevel, message: string, file?: string, details?: string) => void;
  clearLogs: () => void;
  setFilter: (filter: LogLevel | 'all') => void;
  toggleDrawer: () => void;
  openDrawer: () => void;
  closeDrawer: () => void;
}

export const useLogsStore = create<LogsState>((set) => ({
  logs: [],
  isOpen: false,
  filter: 'all',

  addLog: (level, message, file, details) => {
    const entry: LogEntry = {
      id: crypto.randomUUID(),
      timestamp: new Date(),
      level,
      message,
      file,
      details,
    };
    set((state) => ({ logs: [entry, ...state.logs].slice(0, 500) })); // Keep last 500
  },

  clearLogs: () => set({ logs: [] }),

  setFilter: (filter) => set({ filter }),

  toggleDrawer: () => set((state) => ({ isOpen: !state.isOpen })),
  openDrawer: () => set({ isOpen: true }),
  closeDrawer: () => set({ isOpen: false }),
}));
```

---

## 5. Hooks Customizados

### 5.1 useModels

**Arquivo**: `src/hooks/useModels.ts`

```typescript
import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LLMModel } from '../types';
import { useSettingsStore } from '../stores/settingsStore';

export function useModels() {
  const [models, setModels] = useState<LLMModel[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { settings } = useSettingsStore();

  const fetchModels = useCallback(async () => {
    if (!settings.baseUrl) {
      setModels([]);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const result = await invoke<LLMModel[]>('list_llm_models', {
        baseUrl: settings.baseUrl,
        apiKey: settings.apiKey,
      });
      setModels(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch models');
      setModels([]);
    } finally {
      setIsLoading(false);
    }
  }, [settings.baseUrl, settings.apiKey]);

  // Auto-fetch when baseUrl changes
  useEffect(() => {
    const timer = setTimeout(fetchModels, 500); // Debounce
    return () => clearTimeout(timer);
  }, [fetchModels]);

  return { models, isLoading, error, refetch: fetchModels };
}
```

### 5.2 useFileHandler

**Arquivo**: `src/hooks/useFileHandler.ts`

```typescript
import { useCallback } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { useTranslationStore } from '../stores/translationStore';

const SUBTITLE_EXTENSIONS = ['srt', 'ass', 'ssa'];
const VIDEO_EXTENSIONS = ['mkv', 'mp4', 'avi', 'webm', 'mov'];

function getFileType(path: string): 'subtitle' | 'video' | null {
  const ext = path.split('.').pop()?.toLowerCase();
  if (!ext) return null;
  if (SUBTITLE_EXTENSIONS.includes(ext)) return 'subtitle';
  if (VIDEO_EXTENSIONS.includes(ext)) return 'video';
  return null;
}

export function useFileHandler() {
  const addFiles = useTranslationStore((s) => s.addFiles);

  const handleFileDrop = useCallback((paths: string[]) => {
    const validFiles = paths
      .map((path) => {
        const type = getFileType(path);
        if (!type) return null;
        const name = path.split(/[/\\]/).pop() || path;
        return { name, path, type };
      })
      .filter((f): f is NonNullable<typeof f> => f !== null);

    if (validFiles.length > 0) {
      addFiles(validFiles);
    }
  }, [addFiles]);

  const openFilePicker = useCallback(async () => {
    const selected = await open({
      multiple: true,
      filters: [
        { name: 'Legendas', extensions: SUBTITLE_EXTENSIONS },
        { name: 'Vídeos', extensions: VIDEO_EXTENSIONS },
        { name: 'Todos suportados', extensions: [...SUBTITLE_EXTENSIONS, ...VIDEO_EXTENSIONS] },
      ],
    });

    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      handleFileDrop(paths);
    }
  }, [handleFileDrop]);

  return { handleFileDrop, openFilePicker };
}
```

---

## 6. Componentes - Implementação Detalhada

### 6.1 App.tsx (Layout Principal)

**Arquivo**: `src/App.tsx`

```tsx
import { useEffect, useState } from 'react';
import { Tabs } from '@heroui/react';
import { MainLayout } from './components/layout/MainLayout';
import { TranslationPage } from './components/translation/TranslationPage';
import { ConfigPage } from './components/config/ConfigPage';
import { useSettingsStore } from './stores/settingsStore';
import './App.css';

function App() {
  const [activeTab, setActiveTab] = useState('translation');
  const { loadSettings, loadTemplates, checkFfmpeg } = useSettingsStore();

  useEffect(() => {
    // Load all data on startup
    loadSettings();
    loadTemplates();
    checkFfmpeg();
  }, []);

  return (
    <MainLayout activeTab={activeTab} onTabChange={setActiveTab}>
      {activeTab === 'translation' ? <TranslationPage /> : <ConfigPage />}
    </MainLayout>
  );
}

export default App;
```

### 6.2 MainLayout

**Arquivo**: `src/components/layout/MainLayout.tsx`

```tsx
import { ReactNode } from 'react';
import { Navbar } from './Navbar';
import { LogsDrawer } from './LogsDrawer';

interface Props {
  children: ReactNode;
  activeTab: string;
  onTabChange: (tab: string) => void;
}

export function MainLayout({ children, activeTab, onTabChange }: Props) {
  return (
    <div className="min-h-screen bg-background">
      <Navbar activeTab={activeTab} onTabChange={onTabChange} />
      <main className="container mx-auto p-4 pt-20">
        {children}
      </main>
      <LogsDrawer />
    </div>
  );
}
```

### 6.3 Navbar

**Arquivo**: `src/components/layout/Navbar.tsx`

```tsx
import { Button, Chip, Tabs } from '@heroui/react';
import { useTranslationStore } from '../../stores/translationStore';
import { useLogsStore } from '../../stores/logsStore';
import { useSettingsStore } from '../../stores/settingsStore';

interface Props {
  activeTab: string;
  onTabChange: (tab: string) => void;
}

export function Navbar({ activeTab, onTabChange }: Props) {
  const { isTranslating, isPaused, startTranslation, pauseTranslation, resumeTranslation, queue } = useTranslationStore();
  const { toggleDrawer, logs } = useLogsStore();
  const { ffmpegInstalled } = useSettingsStore();

  const pendingCount = queue.filter((f) => f.status === 'pending').length;
  const errorCount = logs.filter((l) => l.level === 'error').length;

  const handleTranslateClick = () => {
    if (isTranslating) {
      if (isPaused) {
        resumeTranslation();
      } else {
        pauseTranslation();
      }
    } else {
      startTranslation();
    }
  };

  const getTranslateButtonText = () => {
    if (!isTranslating) return 'Traduzir';
    if (isPaused) return 'Retomar';
    return 'Pausar';
  };

  return (
    <nav className="fixed top-0 left-0 right-0 z-50 bg-surface border-b border-divider">
      <div className="container mx-auto px-4 h-16 flex items-center justify-between">
        {/* Left: Logo + Tabs */}
        <div className="flex items-center gap-6">
          <h1 className="text-xl font-bold">SubTranslator</h1>

          <Tabs
            selectedKey={activeTab}
            onSelectionChange={(key) => onTabChange(key as string)}
            className="hidden sm:flex"
          >
            <Tabs.ListContainer>
              <Tabs.List aria-label="Navegação">
                <Tabs.Tab id="translation">
                  Tradução
                  <Tabs.Indicator />
                </Tabs.Tab>
                <Tabs.Tab id="config">
                  Configurações
                  <Tabs.Indicator />
                </Tabs.Tab>
              </Tabs.List>
            </Tabs.ListContainer>
          </Tabs>
        </div>

        {/* Right: Actions */}
        <div className="flex items-center gap-3">
          {/* FFmpeg status */}
          {ffmpegInstalled !== null && (
            <Chip
              color={ffmpegInstalled ? 'success' : 'danger'}
              size="sm"
              variant="flat"
            >
              FFmpeg {ffmpegInstalled ? 'OK' : 'N/A'}
            </Chip>
          )}

          {/* Queue count */}
          {pendingCount > 0 && (
            <Chip color="primary" size="sm" variant="flat">
              {pendingCount} na fila
            </Chip>
          )}

          {/* Translate button */}
          <Button
            color={isTranslating && !isPaused ? 'warning' : 'primary'}
            onPress={handleTranslateClick}
            isDisabled={queue.length === 0 && !isTranslating}
          >
            {getTranslateButtonText()}
          </Button>

          {/* Logs button */}
          <Button
            variant="ghost"
            onPress={toggleDrawer}
            className="relative"
          >
            Logs
            {errorCount > 0 && (
              <span className="absolute -top-1 -right-1 bg-danger text-white text-xs rounded-full w-5 h-5 flex items-center justify-center">
                {errorCount}
              </span>
            )}
          </Button>
        </div>
      </div>
    </nav>
  );
}
```

### 6.4 LogsDrawer

**Arquivo**: `src/components/layout/LogsDrawer.tsx`

```tsx
import { Drawer, Button, Chip, Select } from '@heroui/react';
import { useLogsStore } from '../../stores/logsStore';
import { LogLevel } from '../../types';

const LEVEL_COLORS: Record<LogLevel, 'default' | 'success' | 'warning' | 'danger'> = {
  info: 'default',
  success: 'success',
  warning: 'warning',
  error: 'danger',
};

export function LogsDrawer() {
  const { logs, isOpen, closeDrawer, clearLogs, filter, setFilter } = useLogsStore();

  const filteredLogs = filter === 'all'
    ? logs
    : logs.filter((l) => l.level === filter);

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('pt-BR', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  };

  return (
    <Drawer isOpen={isOpen} onOpenChange={(open) => !open && closeDrawer()} placement="right">
      <Drawer.Content className="w-96 max-w-full">
        <Drawer.Header className="flex items-center justify-between">
          <span className="text-lg font-semibold">Logs</span>
          <div className="flex gap-2">
            <Select
              aria-label="Filtrar logs"
              selectedKey={filter}
              onSelectionChange={(key) => setFilter(key as LogLevel | 'all')}
              className="w-28"
            >
              <Select.Trigger>
                <Select.Value />
              </Select.Trigger>
              <Select.Popover>
                <Select.ListBox>
                  <Select.Item key="all">Todos</Select.Item>
                  <Select.Item key="info">Info</Select.Item>
                  <Select.Item key="success">Sucesso</Select.Item>
                  <Select.Item key="warning">Warning</Select.Item>
                  <Select.Item key="error">Erro</Select.Item>
                </Select.ListBox>
              </Select.Popover>
            </Select>
            <Button size="sm" variant="ghost" onPress={clearLogs}>
              Limpar
            </Button>
          </div>
        </Drawer.Header>

        <Drawer.Body className="p-0">
          <div className="divide-y divide-divider">
            {filteredLogs.length === 0 ? (
              <p className="p-4 text-center text-foreground-500">
                Nenhum log encontrado
              </p>
            ) : (
              filteredLogs.map((log) => (
                <div key={log.id} className="p-3 hover:bg-content2">
                  <div className="flex items-start gap-2">
                    <Chip size="sm" color={LEVEL_COLORS[log.level]} variant="flat">
                      {log.level}
                    </Chip>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm break-words">{log.message}</p>
                      {log.file && (
                        <p className="text-xs text-foreground-500 mt-1">
                          Arquivo: {log.file}
                        </p>
                      )}
                      <p className="text-xs text-foreground-400 mt-1">
                        {formatTime(log.timestamp)}
                      </p>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </Drawer.Body>
      </Drawer.Content>
    </Drawer>
  );
}
```

### 6.5 TranslationPage

**Arquivo**: `src/components/translation/TranslationPage.tsx`

```tsx
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
      <Tabs className="w-full">
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
      </Tabs>
    </div>
  );
}
```

### 6.6 FileDropZone

**Arquivo**: `src/components/translation/FileDropZone.tsx`

```tsx
import { useState, useCallback } from 'react';
import { Button } from '@heroui/react';
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { useFileHandler } from '../../hooks/useFileHandler';

export function FileDropZone() {
  const [isDragging, setIsDragging] = useState(false);
  const { handleFileDrop, openFilePicker } = useFileHandler();

  // Listen for Tauri file drop events
  useEffect(() => {
    const unlisten = listen<{ paths: string[] }>('tauri://drop', (event) => {
      handleFileDrop(event.payload.paths);
    });

    const unlistenHover = listen('tauri://drop-hover', () => {
      setIsDragging(true);
    });

    const unlistenCancel = listen('tauri://drop-cancelled', () => {
      setIsDragging(false);
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenHover.then((fn) => fn());
      unlistenCancel.then((fn) => fn());
    };
  }, [handleFileDrop]);

  // Also handle HTML5 drag events for visual feedback
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback(() => {
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    // Tauri handles the actual file paths
  }, []);

  return (
    <div
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      className={`
        border-2 border-dashed rounded-xl p-8 text-center transition-all duration-200
        ${isDragging
          ? 'border-primary bg-primary/10 scale-[1.02]'
          : 'border-divider hover:border-primary/50'
        }
      `}
    >
      <div className="space-y-4">
        <div className="text-4xl">📁</div>
        <div>
          <p className="text-lg font-medium">
            Arraste arquivos aqui
          </p>
          <p className="text-sm text-foreground-500 mt-1">
            Suporta: SRT, ASS, SSA, MKV, MP4, AVI
          </p>
        </div>
        <div className="flex items-center gap-4 justify-center">
          <div className="h-px bg-divider flex-1 max-w-16" />
          <span className="text-foreground-500">ou</span>
          <div className="h-px bg-divider flex-1 max-w-16" />
        </div>
        <Button color="primary" variant="flat" onPress={openFilePicker}>
          Selecionar arquivos
        </Button>
      </div>
    </div>
  );
}
```

### 6.7 FileQueue

**Arquivo**: `src/components/translation/FileQueue.tsx`

```tsx
import { Card, Button } from '@heroui/react';
import { FileQueueItem } from './FileQueueItem';
import { useTranslationStore } from '../../stores/translationStore';

interface Props {
  maxVisible?: number;
}

export function FileQueue({ maxVisible }: Props) {
  const { queue, clearQueue } = useTranslationStore();

  const visibleQueue = maxVisible ? queue.slice(0, maxVisible) : queue;
  const hiddenCount = maxVisible ? Math.max(0, queue.length - maxVisible) : 0;

  if (queue.length === 0) {
    return null;
  }

  return (
    <Card className="p-4">
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold">
          Fila ({queue.length} {queue.length === 1 ? 'arquivo' : 'arquivos'})
        </h3>
        <Button size="sm" variant="ghost" color="danger" onPress={clearQueue}>
          Limpar
        </Button>
      </div>

      <div className="space-y-2">
        {visibleQueue.map((file, index) => (
          <FileQueueItem key={file.id} file={file} index={index} />
        ))}

        {hiddenCount > 0 && (
          <p className="text-center text-sm text-foreground-500 py-2">
            +{hiddenCount} arquivo(s) na fila
          </p>
        )}
      </div>
    </Card>
  );
}
```

### 6.8 FileQueueItem

**Arquivo**: `src/components/translation/FileQueueItem.tsx`

```tsx
import { Button, Chip, Progress } from '@heroui/react';
import { QueueFile, FileStatus } from '../../types';
import { useTranslationStore } from '../../stores/translationStore';

interface Props {
  file: QueueFile;
  index: number;
}

const STATUS_CONFIG: Record<FileStatus, { label: string; color: 'default' | 'primary' | 'success' | 'warning' | 'danger' }> = {
  pending: { label: 'Pendente', color: 'default' },
  extracting: { label: 'Extraindo', color: 'primary' },
  translating: { label: 'Traduzindo', color: 'primary' },
  paused: { label: 'Pausado', color: 'warning' },
  completed: { label: 'Concluído', color: 'success' },
  error: { label: 'Erro', color: 'danger' },
};

export function FileQueueItem({ file, index }: Props) {
  const { removeFile } = useTranslationStore();
  const config = STATUS_CONFIG[file.status];

  const isProcessing = file.status === 'extracting' || file.status === 'translating';

  return (
    <div className="flex items-center gap-3 p-3 bg-content2 rounded-lg">
      <div className="w-6 text-center text-foreground-500 text-sm">
        {index + 1}
      </div>

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium truncate">{file.name}</span>
          <Chip size="sm" color={config.color} variant="flat">
            {config.label}
          </Chip>
        </div>

        {isProcessing && (
          <Progress
            value={file.progress}
            className="mt-2"
            size="sm"
            color="primary"
          />
        )}

        {file.status === 'translating' && file.totalLines > 0 && (
          <p className="text-xs text-foreground-500 mt-1">
            {file.translatedLines}/{file.totalLines} linhas
          </p>
        )}

        {file.error && (
          <p className="text-xs text-danger mt-1 truncate">
            {file.error}
          </p>
        )}
      </div>

      <Button
        size="sm"
        variant="ghost"
        color="danger"
        isIconOnly
        onPress={() => removeFile(file.id)}
        isDisabled={isProcessing}
      >
        ✕
      </Button>
    </div>
  );
}
```

### 6.9 SubtitleEditor

**Arquivo**: `src/components/translation/SubtitleEditor.tsx`

```tsx
import { Card, Progress, Button } from '@heroui/react';
import { QueueFile } from '../../types';
import { SubtitleLine } from './SubtitleLine';
import { useTranslationStore } from '../../stores/translationStore';
import { useRef, useEffect } from 'react';

interface Props {
  file: QueueFile;
}

export function SubtitleEditor({ file }: Props) {
  const { updateFile } = useTranslationStore();
  const scrollRef = useRef<HTMLDivElement>(null);

  const entries = file.originalSubtitle?.entries || [];
  const translated = file.translatedEntries || [];

  const progress = entries.length > 0
    ? Math.round((translated.length / entries.length) * 100)
    : 0;

  const handleTranslationChange = (index: number, newText: string) => {
    const newTranslated = [...translated];
    if (newTranslated[index]) {
      newTranslated[index] = { ...newTranslated[index], text: newText };
      updateFile(file.id, { translatedEntries: newTranslated });
    }
  };

  // Auto-scroll to current translation
  useEffect(() => {
    if (scrollRef.current && translated.length > 0) {
      const lastItem = scrollRef.current.querySelector(`[data-index="${translated.length - 1}"]`);
      lastItem?.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }, [translated.length]);

  return (
    <Card className="p-4 h-[600px] flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div>
          <h3 className="font-semibold">{file.name}</h3>
          <p className="text-sm text-foreground-500">
            {file.originalSubtitle?.format.toUpperCase()} • {entries.length} linhas
          </p>
        </div>
        <div className="text-right">
          <p className="text-sm font-medium">{progress}%</p>
          <Progress value={progress} className="w-24" size="sm" />
        </div>
      </div>

      <div className="grid grid-cols-2 gap-2 text-xs font-medium text-foreground-500 px-2 mb-2">
        <div>Original</div>
        <div>Tradução</div>
      </div>

      <div ref={scrollRef} className="flex-1 overflow-y-auto space-y-1">
        {entries.map((entry, index) => (
          <SubtitleLine
            key={entry.index}
            index={index}
            original={entry}
            translated={translated[index]}
            onTranslationChange={(text) => handleTranslationChange(index, text)}
          />
        ))}
      </div>
    </Card>
  );
}
```

### 6.10 SubtitleLine

**Arquivo**: `src/components/translation/SubtitleLine.tsx`

```tsx
import { TextArea } from '@heroui/react';
import { SubtitleEntry } from '../../types';

interface Props {
  index: number;
  original: SubtitleEntry;
  translated?: SubtitleEntry;
  onTranslationChange: (text: string) => void;
}

export function SubtitleLine({ index, original, translated, onTranslationChange }: Props) {
  const isTranslated = !!translated;

  return (
    <div
      data-index={index}
      className={`
        grid grid-cols-2 gap-2 p-2 rounded-lg transition-colors
        ${isTranslated ? 'bg-success/10' : 'bg-content2'}
      `}
    >
      {/* Original */}
      <div className="text-sm">
        <p className="text-xs text-foreground-500 mb-1">
          {original.start_time} → {original.end_time}
        </p>
        <p className="whitespace-pre-wrap">{original.text}</p>
      </div>

      {/* Translation */}
      <div>
        <TextArea
          aria-label={`Tradução linha ${index + 1}`}
          value={translated?.text || ''}
          onChange={(e) => onTranslationChange(e.target.value)}
          placeholder={isTranslated ? '' : 'Aguardando tradução...'}
          className="w-full text-sm"
          rows={2}
          isDisabled={!isTranslated}
        />
      </div>
    </div>
  );
}
```

### 6.11 ConfigPage

**Arquivo**: `src/components/config/ConfigPage.tsx`

```tsx
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
```

### 6.12 ApiSettings

**Arquivo**: `src/components/config/ApiSettings.tsx`

```tsx
import { Card, Input, Select, Accordion, Button } from '@heroui/react';
import { useSettingsStore } from '../../stores/settingsStore';
import { useModels } from '../../hooks/useModels';
import { useState } from 'react';
import { Header } from '../../types';

export function ApiSettings() {
  const { settings, updateSetting, saveSettings } = useSettingsStore();
  const { models, isLoading, error, refetch } = useModels();

  const addHeader = () => {
    const newHeader: Header = {
      id: crypto.randomUUID(),
      key: '',
      value: '',
    };
    updateSetting('headers', [...settings.headers, newHeader]);
  };

  const updateHeader = (id: string, field: 'key' | 'value', value: string) => {
    const newHeaders = settings.headers.map((h) =>
      h.id === id ? { ...h, [field]: value } : h
    );
    updateSetting('headers', newHeaders);
  };

  const removeHeader = (id: string) => {
    updateSetting('headers', settings.headers.filter((h) => h.id !== id));
  };

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Conexão com API</h3>

      <div className="space-y-4">
        {/* Base URL */}
        <div>
          <label className="block text-sm font-medium mb-1">Base URL</label>
          <Input
            placeholder="http://localhost:8045/v1"
            value={settings.baseUrl}
            onChange={(e) => updateSetting('baseUrl', e.target.value)}
            className="w-full"
          />
          <p className="text-xs text-foreground-500 mt-1">
            O endpoint /chat/completions será adicionado automaticamente
          </p>
        </div>

        {/* API Key */}
        <div>
          <label className="block text-sm font-medium mb-1">API Key</label>
          <Input
            type="password"
            placeholder="sk-..."
            value={settings.apiKey}
            onChange={(e) => updateSetting('apiKey', e.target.value)}
            className="w-full"
          />
        </div>

        {/* Model Selection */}
        <div>
          <label className="block text-sm font-medium mb-1">Modelo</label>
          <div className="flex gap-2">
            <Select
              aria-label="Selecionar modelo"
              selectedKey={settings.model}
              onSelectionChange={(key) => updateSetting('model', key as string)}
              className="flex-1"
              isDisabled={isLoading || models.length === 0}
            >
              <Select.Trigger>
                <Select.Value placeholder={isLoading ? 'Carregando...' : 'Selecione um modelo'} />
              </Select.Trigger>
              <Select.Popover>
                <Select.ListBox>
                  {models.map((model) => (
                    <Select.Item key={model.id}>{model.name || model.id}</Select.Item>
                  ))}
                </Select.ListBox>
              </Select.Popover>
            </Select>
            <Button variant="ghost" onPress={refetch} isDisabled={isLoading}>
              ↻
            </Button>
          </div>
          {error && <p className="text-xs text-danger mt-1">{error}</p>}
        </div>

        {/* Custom Model */}
        <div>
          <label className="block text-sm font-medium mb-1">Modelo Customizado (opcional)</label>
          <Input
            placeholder="gpt-4-turbo"
            value={settings.customModel}
            onChange={(e) => updateSetting('customModel', e.target.value)}
            className="w-full"
          />
          <p className="text-xs text-foreground-500 mt-1">
            Se preenchido, será usado em vez do modelo selecionado acima
          </p>
        </div>

        {/* Advanced Headers */}
        <Accordion>
          <Accordion.Item key="headers" title="Headers Avançados">
            <div className="space-y-2">
              {settings.headers.map((header) => (
                <div key={header.id} className="flex gap-2">
                  <Input
                    placeholder="Header-Name"
                    value={header.key}
                    onChange={(e) => updateHeader(header.id, 'key', e.target.value)}
                    className="flex-1"
                  />
                  <Input
                    placeholder="Value"
                    value={header.value}
                    onChange={(e) => updateHeader(header.id, 'value', e.target.value)}
                    className="flex-1"
                  />
                  <Button
                    variant="ghost"
                    color="danger"
                    isIconOnly
                    onPress={() => removeHeader(header.id)}
                  >
                    ✕
                  </Button>
                </div>
              ))}
              <Button variant="flat" size="sm" onPress={addHeader}>
                + Adicionar Header
              </Button>
            </div>
          </Accordion.Item>
        </Accordion>
      </div>
    </Card>
  );
}
```

### 6.13 FfmpegStatus

**Arquivo**: `src/components/config/FfmpegStatus.tsx`

```tsx
import { Card, Button, Chip, Alert } from '@heroui/react';
import { useSettingsStore } from '../../stores/settingsStore';

export function FfmpegStatus() {
  const { ffmpegInstalled, checkFfmpeg } = useSettingsStore();

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">FFmpeg</h3>

      <div className="flex items-center gap-4">
        <Button onPress={checkFfmpeg}>
          Verificar FFmpeg
        </Button>

        {ffmpegInstalled !== null && (
          <Chip color={ffmpegInstalled ? 'success' : 'danger'}>
            {ffmpegInstalled ? 'Instalado' : 'Não encontrado'}
          </Chip>
        )}
      </div>

      {ffmpegInstalled === false && (
        <Alert className="mt-4">
          <Alert.Indicator />
          <Alert.Content>
            <Alert.Title>FFmpeg não encontrado</Alert.Title>
            <Alert.Description>
              O FFmpeg é necessário para extrair legendas de vídeos.
              Instale-o e adicione ao PATH do sistema.
            </Alert.Description>
          </Alert.Content>
        </Alert>
      )}

      <div className="mt-4 flex gap-2">
        <Chip color="success" variant="flat">SRT</Chip>
        <Chip color="success" variant="flat">ASS</Chip>
        <Chip color="success" variant="flat">SSA</Chip>
        <Chip color="warning" variant="flat">VTT (em breve)</Chip>
      </div>
      <p className="text-xs text-foreground-500 mt-2">
        ASS/SSA preservam melhor estilos e formatação
      </p>
    </Card>
  );
}
```

### 6.14 PromptEditor

**Arquivo**: `src/components/config/PromptEditor.tsx`

```tsx
import { Card, TextArea, Select, Button } from '@heroui/react';
import { useSettingsStore } from '../../stores/settingsStore';

export function PromptEditor() {
  const { settings, updateSetting, templates } = useSettingsStore();

  const applyTemplate = (templateId: string) => {
    const template = templates.find((t) => t.id === templateId);
    if (template) {
      updateSetting('prompt', template.content);
      updateSetting('selectedTemplateId', templateId);
    }
  };

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Prompt de Tradução</h3>

      <div className="space-y-4">
        {templates.length > 0 && (
          <div className="flex gap-2">
            <Select
              aria-label="Selecionar template"
              selectedKey={settings.selectedTemplateId || ''}
              onSelectionChange={(key) => applyTemplate(key as string)}
              className="flex-1"
            >
              <Select.Trigger>
                <Select.Value placeholder="Aplicar template..." />
              </Select.Trigger>
              <Select.Popover>
                <Select.ListBox>
                  {templates.map((template) => (
                    <Select.Item key={template.id}>{template.name}</Select.Item>
                  ))}
                </Select.ListBox>
              </Select.Popover>
            </Select>
          </div>
        )}

        <TextArea
          aria-label="Prompt de tradução"
          value={settings.prompt}
          onChange={(e) => updateSetting('prompt', e.target.value)}
          rows={6}
          className="w-full"
          placeholder="Digite o prompt de tradução..."
        />

        <p className="text-xs text-foreground-500">
          Use um prompt claro que instrua o modelo a traduzir as legendas mantendo formatação e contexto.
        </p>
      </div>
    </Card>
  );
}
```

### 6.15 TemplateManager

**Arquivo**: `src/components/config/TemplateManager.tsx`

```tsx
import { Card, Button, Modal, Input, TextArea } from '@heroui/react';
import { useState } from 'react';
import { useSettingsStore } from '../../stores/settingsStore';
import { Template } from '../../types';

export function TemplateManager() {
  const { templates, addTemplate, updateTemplate, deleteTemplate } = useSettingsStore();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingTemplate, setEditingTemplate] = useState<Template | null>(null);
  const [name, setName] = useState('');
  const [content, setContent] = useState('');

  const openNewModal = () => {
    setEditingTemplate(null);
    setName('');
    setContent('');
    setIsModalOpen(true);
  };

  const openEditModal = (template: Template) => {
    setEditingTemplate(template);
    setName(template.name);
    setContent(template.content);
    setIsModalOpen(true);
  };

  const handleSave = async () => {
    if (!name.trim() || !content.trim()) return;

    try {
      if (editingTemplate) {
        await updateTemplate(editingTemplate.id, name, content);
      } else {
        await addTemplate(name, content);
      }
      setIsModalOpen(false);
    } catch (error) {
      console.error('Failed to save template:', error);
    }
  };

  const handleDelete = async (id: string) => {
    if (confirm('Tem certeza que deseja excluir este template?')) {
      await deleteTemplate(id);
    }
  };

  return (
    <Card className="p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">Templates de Prompt</h3>
        <Button color="primary" size="sm" onPress={openNewModal}>
          + Novo Template
        </Button>
      </div>

      {templates.length === 0 ? (
        <p className="text-foreground-500 text-center py-4">
          Nenhum template criado ainda
        </p>
      ) : (
        <div className="space-y-2">
          {templates.map((template) => (
            <div
              key={template.id}
              className="flex items-center justify-between p-3 bg-content2 rounded-lg"
            >
              <div>
                <p className="font-medium">{template.name}</p>
                <p className="text-sm text-foreground-500 truncate max-w-md">
                  {template.content}
                </p>
              </div>
              <div className="flex gap-2">
                <Button size="sm" variant="flat" onPress={() => openEditModal(template)}>
                  Editar
                </Button>
                <Button
                  size="sm"
                  variant="flat"
                  color="danger"
                  onPress={() => handleDelete(template.id)}
                >
                  Excluir
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Modal */}
      <Modal isOpen={isModalOpen} onOpenChange={setIsModalOpen}>
        <Modal.Content>
          <Modal.Header>
            {editingTemplate ? 'Editar Template' : 'Novo Template'}
          </Modal.Header>
          <Modal.Body>
            <div className="space-y-4">
              <Input
                label="Nome"
                placeholder="Ex: Anime Informal"
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
              <TextArea
                label="Conteúdo do Prompt"
                placeholder="Digite o prompt..."
                value={content}
                onChange={(e) => setContent(e.target.value)}
                rows={6}
              />
            </div>
          </Modal.Body>
          <Modal.Footer>
            <Button variant="ghost" onPress={() => setIsModalOpen(false)}>
              Cancelar
            </Button>
            <Button color="primary" onPress={handleSave}>
              Salvar
            </Button>
          </Modal.Footer>
        </Modal.Content>
      </Modal>
    </Card>
  );
}
```

### 6.16 TranslationSettings

**Arquivo**: `src/components/config/TranslationSettings.tsx`

```tsx
import { Card, Input, Switch, Select } from '@heroui/react';
import { useSettingsStore } from '../../stores/settingsStore';

const BATCH_PRESETS = [
  { value: '25', label: '25 linhas' },
  { value: '50', label: '50 linhas' },
  { value: '100', label: '100 linhas' },
  { value: '150', label: '150 linhas' },
];

export function TranslationSettings() {
  const { settings, updateSetting } = useSettingsStore();

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Configurações de Tradução</h3>

      <div className="space-y-4">
        {/* Batch Size */}
        <div>
          <label className="block text-sm font-medium mb-1">Tamanho do Batch</label>
          <div className="flex gap-2">
            <Select
              aria-label="Tamanho do batch"
              selectedKey={String(settings.batchSize)}
              onSelectionChange={(key) => updateSetting('batchSize', Number(key))}
              className="flex-1"
            >
              <Select.Trigger>
                <Select.Value />
              </Select.Trigger>
              <Select.Popover>
                <Select.ListBox>
                  {BATCH_PRESETS.map((preset) => (
                    <Select.Item key={preset.value}>{preset.label}</Select.Item>
                  ))}
                </Select.ListBox>
              </Select.Popover>
            </Select>
            <Input
              type="number"
              value={String(settings.batchSize)}
              onChange={(e) => updateSetting('batchSize', Number(e.target.value))}
              className="w-24"
              min={1}
              max={500}
            />
          </div>
          <p className="text-xs text-foreground-500 mt-1">
            Quantidade de linhas enviadas por requisição
          </p>
        </div>

        {/* Concurrency */}
        <div>
          <label className="block text-sm font-medium mb-1">Concorrência</label>
          <Input
            type="number"
            value={String(settings.concurrency)}
            onChange={(e) => updateSetting('concurrency', Number(e.target.value))}
            className="w-24"
            min={1}
            max={10}
          />
          <p className="text-xs text-foreground-500 mt-1">
            Número de arquivos processados simultaneamente
          </p>
        </div>

        {/* Max Retries */}
        <div>
          <label className="block text-sm font-medium mb-1">Máximo de Retentativas</label>
          <Input
            type="number"
            value={String(settings.maxRetries)}
            onChange={(e) => updateSetting('maxRetries', Number(e.target.value))}
            className="w-24"
            min={0}
            max={10}
          />
        </div>

        {/* Toggles */}
        <div className="space-y-3 pt-2">
          <Switch
            isSelected={settings.autoContinue}
            onValueChange={(v) => updateSetting('autoContinue', v)}
          >
            Continuar automaticamente (respostas parciais)
          </Switch>

          <Switch
            isSelected={settings.continueOnError}
            onValueChange={(v) => updateSetting('continueOnError', v)}
          >
            Continuar fila em caso de erro
          </Switch>
        </div>
      </div>
    </Card>
  );
}
```

### 6.17 OutputSettings

**Arquivo**: `src/components/config/OutputSettings.tsx`

```tsx
import { Card, Input, Switch, Select, Button } from '@heroui/react';
import { useSettingsStore } from '../../stores/settingsStore';
import { open } from '@tauri-apps/plugin-dialog';

export function OutputSettings() {
  const { settings, updateSetting } = useSettingsStore();

  const selectOutputDir = async () => {
    const dir = await open({
      directory: true,
      multiple: false,
    });
    if (dir) {
      updateSetting('separateOutputDir', dir as string);
    }
  };

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Configurações de Saída</h3>

      <div className="space-y-4">
        {/* Output Mode */}
        <div>
          <label className="block text-sm font-medium mb-1">Modo de Saída</label>
          <Select
            aria-label="Modo de saída"
            selectedKey={settings.outputMode}
            onSelectionChange={(key) => updateSetting('outputMode', key as 'mux' | 'separate')}
          >
            <Select.Trigger>
              <Select.Value />
            </Select.Trigger>
            <Select.Popover>
              <Select.ListBox>
                <Select.Item key="separate">Arquivo Separado</Select.Item>
                <Select.Item key="mux">Mux no Vídeo (MKV)</Select.Item>
              </Select.ListBox>
            </Select.Popover>
          </Select>
        </div>

        {/* Separate mode options */}
        {settings.outputMode === 'separate' && (
          <div>
            <label className="block text-sm font-medium mb-1">Pasta de Saída</label>
            <div className="flex gap-2">
              <Input
                value={settings.separateOutputDir}
                onChange={(e) => updateSetting('separateOutputDir', e.target.value)}
                placeholder="Mesma pasta do original"
                className="flex-1"
              />
              <Button variant="flat" onPress={selectOutputDir}>
                Selecionar
              </Button>
            </div>
          </div>
        )}

        {/* Mux mode options */}
        {settings.outputMode === 'mux' && (
          <>
            <div>
              <label className="block text-sm font-medium mb-1">Código do Idioma</label>
              <Input
                value={settings.muxLanguage}
                onChange={(e) => updateSetting('muxLanguage', e.target.value)}
                placeholder="por"
                className="w-32"
              />
              <p className="text-xs text-foreground-500 mt-1">
                Código ISO 639-2 (ex: por, eng, spa)
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium mb-1">Título da Faixa</label>
              <Input
                value={settings.muxTitle}
                onChange={(e) => updateSetting('muxTitle', e.target.value)}
                placeholder="Portuguese"
                className="w-48"
              />
            </div>
          </>
        )}

        {/* Backup info */}
        <div className="p-3 bg-content2 rounded-lg">
          <p className="text-sm">
            <span className="font-medium">Backup automático:</span> Sempre ativado
          </p>
          <p className="text-xs text-foreground-500 mt-1">
            Um backup do arquivo original será criado antes de qualquer modificação
          </p>
        </div>
      </div>
    </Card>
  );
}
```

---

## 7. Integração com Tauri

### 7.1 Comandos Disponíveis

Os seguintes comandos Tauri já estão implementados no backend (`src-tauri/src/lib.rs`):

```typescript
// Legendas
invoke('load_subtitle', { path: string }) → SubtitleFile
invoke('save_subtitle', { subtitle: SubtitleFile, path: string, format: SubtitleFormat }) → void
invoke('detect_subtitle_format', { path: string }) → SubtitleFormat

// FFmpeg
invoke('check_ffmpeg_installed') → boolean
invoke('list_video_subtitle_tracks', { videoPath: string }) → SubtitleTrack[]
invoke('extract_subtitle_track', { videoPath: string, trackIndex: number }) → string // returns extracted path
invoke('mux_subtitle_to_video', { videoPath: string, subtitlePath: string, language?: string, title?: string }) → string

// Tradução
invoke('list_llm_models', { baseUrl: string, apiKey?: string }) → LLMModel[]
invoke('translate_subtitle_full', {
  subtitle: SubtitleFile,
  prompt: string,
  baseUrl: string,
  apiKey: string,
  model: string,
  batchSize: number,
  headers?: Record<string, string>
}) → { translated_entries: SubtitleEntry[], is_partial: boolean }
invoke('translate_subtitle_batch', { ... }) → ...
invoke('continue_translation', { ... }) → ...

// Templates
invoke('load_templates') → Template[]
invoke('add_template', { name: string, content: string }) → void
invoke('update_template', { id: string, name: string, content: string }) → void
invoke('delete_template', { id: string }) → void

// Settings (a implementar no backend se não existir)
invoke('load_settings') → AppSettings
invoke('save_settings', { settings: AppSettings }) → void

// Utilitários
invoke('backup_file', { path: string }) → string // returns backup path
invoke('replace_file', { sourcePath: string, destPath: string }) → void
```

### 7.2 Eventos Tauri

Escutar eventos do backend para progresso:

```typescript
import { listen } from '@tauri-apps/api/event';

// Progresso de tradução
listen<{ file_id: string, progress: number, translated: number, total: number }>('translation:progress', (event) => {
  // Atualizar UI
});

// Erros
listen<{ file_id: string, error: string, retry_count: number }>('translation:error', (event) => {
  // Adicionar log
});
```

---

## 8. Fluxos de Usuário

### 8.1 Primeiro Uso

1. Usuário abre o app
2. Sistema carrega settings (ou defaults)
3. Sistema verifica FFmpeg automaticamente
4. Usuário vai em Configurações
5. Configura Base URL e API Key
6. Sistema busca modelos automaticamente
7. Usuário seleciona modelo
8. Usuário configura prompt (ou aplica template)
9. Usuário volta para Tradução

### 8.2 Tradução Single

1. Usuário arrasta arquivo (legenda ou vídeo)
2. Se vídeo:
   a. Sistema lista faixas de legenda
   b. Usuário seleciona faixa
   c. Sistema extrai legenda
3. Sistema carrega e exibe legenda no editor
4. Usuário clica "Traduzir" na navbar
5. Sistema traduz em batches, atualizando UI
6. Usuário pode editar traduções manualmente
7. Usuário salva (mux ou arquivo separado)

### 8.3 Tradução Batch

1. Usuário arrasta múltiplos arquivos
2. Sistema monta fila com status "Pendente"
3. Usuário clica "Traduzir"
4. Sistema processa arquivos conforme concorrência
5. Cada arquivo passa por: Extraindo → Traduzindo → Concluído/Erro
6. Usuário pode pausar/retomar a qualquer momento
7. Logs mostram progresso detalhado

### 8.4 Tratamento de Erros

1. Se request falhar:
   a. Sistema tenta novamente (até maxRetries)
   b. Se continueOnError = true: marca como erro, continua fila
   c. Se continueOnError = false: pausa fila, mostra erro
2. Erros aparecem no drawer de logs
3. Usuário pode reprocessar arquivos com erro

---

## 9. Checklist de Implementação

### Fase 1: Estrutura Base
- [ ] Criar estrutura de pastas conforme seção 2
- [ ] Criar `src/types/index.ts` com todos os tipos
- [ ] Instalar Zustand: `npm install zustand`
- [ ] Criar os 3 stores (settings, translation, logs)
- [ ] Criar hooks customizados

### Fase 2: Layout e Navegação
- [ ] Implementar `MainLayout.tsx`
- [ ] Implementar `Navbar.tsx`
- [ ] Implementar `LogsDrawer.tsx`
- [ ] Atualizar `App.tsx` com layout e tabs

### Fase 3: Tela de Tradução
- [ ] Implementar `TranslationPage.tsx`
- [ ] Implementar `FileDropZone.tsx`
- [ ] Implementar `FileQueue.tsx` e `FileQueueItem.tsx`
- [ ] Implementar `SubtitleEditor.tsx` e `SubtitleLine.tsx`

### Fase 4: Tela de Configurações
- [ ] Implementar `ConfigPage.tsx`
- [ ] Implementar `ApiSettings.tsx`
- [ ] Implementar `FfmpegStatus.tsx`
- [ ] Implementar `PromptEditor.tsx`
- [ ] Implementar `TemplateManager.tsx`
- [ ] Implementar `TranslationSettings.tsx`
- [ ] Implementar `OutputSettings.tsx`

### Fase 5: Integração Tauri
- [ ] Testar todos os comandos Tauri
- [ ] Implementar load/save de settings no backend (se necessário)
- [ ] Configurar listeners de eventos
- [ ] Testar fluxo completo de tradução

### Fase 6: Polimento
- [ ] Adicionar loading states em todos os botões
- [ ] Adicionar toasts de feedback
- [ ] Testar responsividade
- [ ] Testar edge cases (arquivo vazio, API offline, etc)

---

## Notas Finais

- **Backup é obrigatório**: Sempre chamar `backup_file` antes de salvar
- **VTT é "soon"**: Mostrar chip de warning, não permitir tradução
- **ASS é preferido**: Mostrar dica sobre preservação de estilos
- **Logs são essenciais**: Todo erro deve ir para o drawer
- **Settings persistem**: Salvar toda alteração automaticamente (com debounce)
