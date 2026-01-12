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
