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

  // Modelo para detecção de idioma
  languageDetectionModel: string;

  // Prompt
  prompt: string;
  selectedTemplateId: string | null;

  // Tradução
  batchSize: number;
  parallelRequests: number;  // Número de requisições paralelas por arquivo
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
  languageDetectionModel: '',
  prompt: 'Translate the following subtitle lines to Brazilian Portuguese. Keep the same tone and style. Return only the translations, one per line, in the same order.',
  selectedTemplateId: null,
  batchSize: 50,
  parallelRequests: 1,
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
  createdAt?: number;
  updatedAt?: number;
}

// ============================================
// MODELS
// ============================================

export interface LLMModel {
  id: string;
  object: string;
  owned_by?: string;
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
// TRANSLATION RESULT
// ============================================

export interface TranslationProgress {
  totalEntries: number;
  translatedEntries: number;
  lastTranslatedIndex: number;
  isPartial: boolean;
  canContinue: boolean;
}

export interface SubtitleTranslationResult {
  file: SubtitleFile;
  progress: TranslationProgress;
  errorMessage?: string;
}

// ============================================
// LANGUAGE DETECTION
// ============================================

export interface DetectedLanguage {
  code: string;        // ISO 639-2 (por, eng, spa, etc)
  name: string;        // Nome completo (Portuguese, English, Spanish)
  displayName: string; // Nome para exibição com código (Portuguese (pt-BR))
}

// ============================================
// TRANSLATION QUEUE
// ============================================

export type FileStatus = 'pending' | 'extracting' | 'translating' | 'detecting_language' | 'saving' | 'muxing' | 'paused' | 'completed' | 'error';

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
  subtitleTracks?: SubtitleTrack[];
  isLoadingTracks?: boolean;
  extractedSubtitlePath?: string;
  
  // Idioma detectado
  detectedLanguage?: DetectedLanguage;
  
  // Paths de saída
  outputSubtitlePath?: string;
  outputVideoPath?: string;
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
