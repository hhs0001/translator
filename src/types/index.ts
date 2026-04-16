// ============================================
// SETTINGS
// ============================================

export type ApiFormat = 'openai' | 'anthropic' | 'auto';
export type Language = 'en' | 'pt-BR';
export type ReasoningEffort = 'default' | 'none' | 'minimal' | 'low' | 'medium' | 'high' | 'xhigh';

export interface AppSettings {
  // API
  baseUrl: string;
  apiKey: string;
  apiFormat: ApiFormat;
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
  streaming: boolean;  // Streaming de traduções conforme chegam da API
  reasoningEffort: ReasoningEffort;
  anthropicThinkingEnabled: boolean;
  anthropicThinkingBudgetTokens: number;

  // Saída
  outputMode: 'mux' | 'separate';
  muxLanguage: string;
  muxTitle: string;
  separateOutputDir: string;
  cleanupExtractedSubtitles: boolean;
  cleanupMuxArtifacts: boolean;

  // Interface language
  language: Language;

  // Text Cleaner (remoção de ruído visual de legendas ASS)
  textCleanerEnabled: boolean;
  textCleanerPreserveBasicFormatting: boolean;
  textCleanerTagsToRemove: string[];
  textCleanerIgnoredStyles: string[];
}

export interface Header {
  id: string;
  key: string;
  value: string;
}

export const DEFAULT_SETTINGS: AppSettings = {
  baseUrl: 'http://localhost:8045/v1',
  apiKey: '',
  apiFormat: 'auto',
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
  streaming: false,
  reasoningEffort: 'default',
  anthropicThinkingEnabled: false,
  anthropicThinkingBudgetTokens: 1024,
  outputMode: 'separate',
  muxLanguage: 'por',
  muxTitle: 'Portuguese',
  separateOutputDir: '',
  cleanupExtractedSubtitles: false,
  cleanupMuxArtifacts: false,
  language: 'en',
  textCleanerEnabled: false,
  textCleanerPreserveBasicFormatting: true,
  textCleanerTagsToRemove: [],
  textCleanerIgnoredStyles: ['draw'],
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
  // Campos adicionais do OpenRouter
  name?: string;
  description?: string;
  context_length?: number;
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

export type FileStatus = 'pending' | 'extracting' | 'translating' | 'detecting_language' | 'saving' | 'muxing' | 'paused' | 'cancelled' | 'completed' | 'error';

export interface BatchProgress {
  batchIndex: number;
  totalInBatch: number;
  completedInBatch: number;
  status: 'pending' | 'active' | 'completed' | 'error';
}

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
  
  // Progresso paralelo (para visualização de batches)
  parallelProgress?: {
    totalBatches: number;
    activeBatches: number;
    completedBatches: number;
    batchProgresses: BatchProgress[];
  };
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
  timestampLabel: string;
  level: LogLevel;
  message: string;
  file?: string;
  details?: string;
}

// ============================================
// TEXT CLEANER
// ============================================

export interface TextCleanerConfig {
  enabled: boolean;
  preserveBasicFormatting: boolean;
  tagsToRemove: string[];
  ignoredStyles: string[];
  preserveKaraokeTiming: boolean;
  preservePositioning: boolean;
}

export const DEFAULT_TEXT_CLEANER_CONFIG: TextCleanerConfig = {
  enabled: false,
  preserveBasicFormatting: true,
  tagsToRemove: [],
  ignoredStyles: ['draw'],
  preserveKaraokeTiming: false,
  preservePositioning: false,
};

export interface AssClutterAnalysis {
  totalLines: number;
  linesWithEffects: number;
  linesWithKaraoke: number;
  linesWithPositioning: number;
  styleCounts: Record<string, number>;
  estimatedTokensSaved: number;
}

export interface CleanedTextPreview {
  index: number;
  original: string;
  cleaned: string;
  shouldSkip: boolean;
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
