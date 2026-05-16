import { expect, test } from 'bun:test';
import type { AppSettings, SubtitleFile, QueueFile, SubtitleEntry, Template, LogEntry, TranslationProgress, SubtitleTranslationResult, DetectedLanguage, TextCleanerConfig, SubtitleTrack } from '../types';
import { DEFAULT_SETTINGS, DEFAULT_TEXT_CLEANER_CONFIG } from '../types';

test('DEFAULT_SETTINGS has all required fields', () => {
  const settings = DEFAULT_SETTINGS;
  
  expect(settings.baseUrl).toBe('http://localhost:8045/v1');
  expect(settings.apiKey).toBe('');
  expect(settings.apiFormat).toBe('auto');
  expect(settings.headers).toEqual([]);
  expect(settings.model).toBe('');
  expect(settings.customModel).toBe('');
  expect(settings.languageDetectionModel).toBe('');
  expect(settings.prompt).toBeTruthy();
  expect(settings.selectedTemplateId).toBeNull();
  expect(settings.batchSize).toBe(50);
  expect(settings.parallelRequests).toBe(1);
  expect(settings.autoContinue).toBe(true);
  expect(settings.continueOnError).toBe(true);
  expect(settings.maxRetries).toBe(3);
  expect(settings.concurrency).toBe(1);
  expect(settings.streaming).toBe(false);
  expect(settings.reasoningEffort).toBe('default');
  expect(settings.anthropicThinkingEnabled).toBe(false);
  expect(settings.anthropicThinkingBudgetTokens).toBe(1024);
  expect(settings.outputMode).toBe('separate');
  expect(settings.muxLanguage).toBe('por');
  expect(settings.muxTitle).toBe('Portuguese');
  expect(settings.separateOutputDir).toBe('');
  expect(settings.cleanupExtractedSubtitles).toBe(false);
  expect(settings.cleanupMuxArtifacts).toBe(false);
  expect(settings.language).toBe('en');
  expect(settings.textCleanerEnabled).toBe(false);
  expect(settings.textCleanerPreserveBasicFormatting).toBe(true);
  expect(settings.textCleanerTagsToRemove).toEqual([]);
  expect(settings.textCleanerIgnoredStyles).toEqual(['draw']);
});

test('DEFAULT_SETTINGS has correct types', () => {
  const settings = DEFAULT_SETTINGS;
  
  expect(typeof settings.baseUrl).toBe('string');
  expect(typeof settings.apiKey).toBe('string');
  expect(typeof settings.batchSize).toBe('number');
  expect(typeof settings.autoContinue).toBe('boolean');
  expect(typeof settings.maxRetries).toBe('number');
});

test('DEFAULT_TEXT_CLEANER_CONFIG has correct values', () => {
  const config = DEFAULT_TEXT_CLEANER_CONFIG;
  
  expect(config.enabled).toBe(false);
  expect(config.preserveBasicFormatting).toBe(true);
  expect(config.tagsToRemove).toEqual([]);
  expect(config.ignoredStyles).toEqual(['draw']);
  expect(config.preserveKaraokeTiming).toBe(false);
  expect(config.preservePositioning).toBe(false);
});

test('SubtitleEntry structure is valid', () => {
  const entry: SubtitleEntry = {
    index: 1,
    start_time: '00:00:01,000',
    end_time: '00:00:02,000',
    text: 'Hello',
    style: 'Default',
    actor: 'Character',
    margin_l: 10,
    margin_r: 10,
    margin_v: 10,
    effect: 'Fade',
  };
  
  expect(entry.index).toBe(1);
  expect(entry.start_time).toBe('00:00:01,000');
  expect(entry.end_time).toBe('00:00:02,000');
  expect(entry.text).toBe('Hello');
  expect(entry.style).toBe('Default');
  expect(entry.actor).toBe('Character');
  expect(entry.margin_l).toBe(10);
  expect(entry.margin_r).toBe(10);
  expect(entry.margin_v).toBe(10);
  expect(entry.effect).toBe('Fade');
});

test('SubtitleFile structure is valid', () => {
  const file: SubtitleFile = {
    format: 'srt',
    entries: [
      { index: 1, start_time: '00:00:01,000', end_time: '00:00:02,000', text: 'Hello' },
      { index: 2, start_time: '00:00:03,000', end_time: '00:00:04,000', text: 'World' },
    ],
  };
  
  expect(file.format).toBe('srt');
  expect(file.entries.length).toBe(2);
  expect(file.entries[0].text).toBe('Hello');
  expect(file.entries[1].text).toBe('World');
});

test('SubtitleFile with ASS format includes optional fields', () => {
  const file: SubtitleFile = {
    format: 'ass',
    header: '[Script Info]',
    styles: '[V4+ Styles]',
    entries: [
      { index: 1, start_time: '00:00:01,000', end_time: '00:00:02,000', text: 'Hello', style: 'Default' },
    ],
  };
  
  expect(file.format).toBe('ass');
  expect(file.header).toBe('[Script Info]');
  expect(file.styles).toBe('[V4+ Styles]');
  expect(file.entries[0].style).toBe('Default');
});

test('QueueFile structure is valid', () => {
  const file: QueueFile = {
    id: 'file-1',
    name: 'video.srt',
    path: '/tmp/video.srt',
    type: 'subtitle',
    status: 'pending',
    progress: 0,
    totalLines: 100,
    translatedLines: 0,
  };
  
  expect(file.id).toBe('file-1');
  expect(file.name).toBe('video.srt');
  expect(file.type).toBe('subtitle');
  expect(file.status).toBe('pending');
  expect(file.progress).toBe(0);
  expect(file.totalLines).toBe(100);
});

test('QueueFile with video type includes track info', () => {
  const file: QueueFile = {
    id: 'file-1',
    name: 'video.mkv',
    path: '/tmp/video.mkv',
    type: 'video',
    status: 'pending',
    progress: 0,
    totalLines: 0,
    translatedLines: 0,
    selectedTrackIndex: 0,
    subtitleTracks: [
      { index: 0, codec: 'ass', language: 'eng', title: 'English' },
      { index: 1, codec: 'ass', language: 'por', title: 'Portuguese' },
    ],
  };
  
  expect(file.type).toBe('video');
  expect(file.selectedTrackIndex).toBe(0);
  expect(file.subtitleTracks?.length).toBe(2);
  expect(file.subtitleTracks?.[0].language).toBe('eng');
});

test('QueueFile with detected language', () => {
  const file: QueueFile = {
    id: 'file-1',
    name: 'video.srt',
    path: '/tmp/video.srt',
    type: 'subtitle',
    status: 'detecting_language',
    progress: 0,
    totalLines: 0,
    translatedLines: 0,
    detectedLanguage: {
      code: 'eng',
      name: 'English',
      displayName: 'English (en)',
    },
  };
  
  expect(file.detectedLanguage?.code).toBe('eng');
  expect(file.detectedLanguage?.name).toBe('English');
  expect(file.detectedLanguage?.displayName).toBe('English (en)');
});

test('QueueFile with parallel progress', () => {
  const file: QueueFile = {
    id: 'file-1',
    name: 'video.srt',
    path: '/tmp/video.srt',
    type: 'subtitle',
    status: 'translating',
    progress: 50,
    totalLines: 100,
    translatedLines: 50,
    parallelProgress: {
      totalBatches: 4,
      activeBatches: 2,
      completedBatches: 1,
      batchProgresses: [
        { batchIndex: 0, totalInBatch: 25, completedInBatch: 25, status: 'completed' },
        { batchIndex: 1, totalInBatch: 25, completedInBatch: 25, status: 'active' },
        { batchIndex: 2, totalInBatch: 25, completedInBatch: 0, status: 'pending' },
        { batchIndex: 3, totalInBatch: 25, completedInBatch: 0, status: 'pending' },
      ],
    },
  };
  
  expect(file.parallelProgress?.totalBatches).toBe(4);
  expect(file.parallelProgress?.activeBatches).toBe(2);
  expect(file.parallelProgress?.completedBatches).toBe(1);
});

test('Template structure is valid', () => {
  const template: Template = {
    id: 'tpl-1',
    name: 'My Template',
    content: 'Translate this text',
    createdAt: Date.now(),
    updatedAt: Date.now(),
  };
  
  expect(template.id).toBe('tpl-1');
  expect(template.name).toBe('My Template');
  expect(template.content).toBe('Translate this text');
  expect(template.createdAt).toBeTruthy();
  expect(template.updatedAt).toBeTruthy();
});

test('LogEntry structure is valid', () => {
  const entry: LogEntry = {
    id: 'log-1',
    timestamp: new Date(),
    timestampLabel: '12:00:00',
    level: 'info',
    message: 'Test message',
    file: 'file.srt',
    details: 'Detailed info',
  };
  
  expect(entry.id).toBe('log-1');
  expect(entry.timestamp).toBeInstanceOf(Date);
  expect(entry.level).toBe('info');
  expect(entry.message).toBe('Test message');
  expect(entry.file).toBe('file.srt');
  expect(entry.details).toBe('Detailed info');
});

test('TranslationProgress structure is valid', () => {
  const progress: TranslationProgress = {
    totalEntries: 100,
    translatedEntries: 50,
    lastTranslatedIndex: 49,
    isPartial: true,
    canContinue: true,
  };
  
  expect(progress.totalEntries).toBe(100);
  expect(progress.translatedEntries).toBe(50);
  expect(progress.lastTranslatedIndex).toBe(49);
  expect(progress.isPartial).toBe(true);
  expect(progress.canContinue).toBe(true);
});

test('SubtitleTranslationResult structure is valid', () => {
  const result: SubtitleTranslationResult = {
    file: {
      format: 'srt',
      entries: [
        { index: 1, start_time: '00:00:01,000', end_time: '00:00:02,000', text: 'Translated' },
      ],
    },
    progress: {
      totalEntries: 1,
      translatedEntries: 1,
      lastTranslatedIndex: 0,
      isPartial: false,
      canContinue: false,
    },
  };
  
  expect(result.file.entries[0].text).toBe('Translated');
  expect(result.progress.translatedEntries).toBe(1);
});

test('SubtitleTranslationResult with error', () => {
  const result: SubtitleTranslationResult = {
    file: { format: 'srt', entries: [] },
    progress: {
      totalEntries: 0,
      translatedEntries: 0,
      lastTranslatedIndex: -1,
      isPartial: false,
      canContinue: false,
    },
    errorMessage: 'API Error',
  };
  
  expect(result.errorMessage).toBe('API Error');
});

test('DetectedLanguage structure is valid', () => {
  const lang: DetectedLanguage = {
    code: 'eng',
    name: 'English',
    displayName: 'English (en)',
  };
  
  expect(lang.code).toBe('eng');
  expect(lang.name).toBe('English');
  expect(lang.displayName).toBe('English (en)');
});

test('TextCleanerConfig structure is valid', () => {
  const config: TextCleanerConfig = {
    enabled: true,
    preserveBasicFormatting: true,
    tagsToRemove: ['pos', 'move'],
    ignoredStyles: ['draw', 'sign'],
    preserveKaraokeTiming: true,
    preservePositioning: false,
  };
  
  expect(config.enabled).toBe(true);
  expect(config.tagsToRemove).toContain('pos');
  expect(config.ignoredStyles).toContain('draw');
  expect(config.preserveKaraokeTiming).toBe(true);
});

test('AppSettings can be partially applied', () => {
  const partial: Partial<AppSettings> = {
    model: 'gpt-4o',
    batchSize: 100,
  };
  
  const merged: AppSettings = { ...DEFAULT_SETTINGS, ...partial };
  
  expect(merged.model).toBe('gpt-4o');
  expect(merged.batchSize).toBe(100);
  expect(merged.apiKey).toBe('');
  expect(merged.baseUrl).toBe('http://localhost:8045/v1');
});

test('all LogLevel values are valid', () => {
  const levels: Array<'info' | 'warning' | 'error' | 'success'> = ['info', 'warning', 'error', 'success'];
  
  for (const level of levels) {
    const entry: LogEntry = {
      id: 'log-1',
      timestamp: new Date(),
      timestampLabel: '12:00:00',
      level,
      message: 'Test',
    };
    expect(entry.level).toBe(level);
  }
});

test('all FileStatus values are valid', () => {
  const statuses = ['pending', 'extracting', 'translating', 'detecting_language', 'saving', 'muxing', 'paused', 'cancelled', 'completed', 'error'];
  
  for (const status of statuses) {
    const file: QueueFile = {
      id: 'test',
      name: 'test.srt',
      path: '/test.srt',
      type: 'subtitle',
      status: status as QueueFile['status'],
      progress: 0,
      totalLines: 0,
      translatedLines: 0,
    };
    expect(file.status).toBe(status);
  }
});

test('all SubtitleFormat values are valid', () => {
  const formats: Array<'srt' | 'ass' | 'ssa' | 'vtt' | 'unknown'> = ['srt', 'ass', 'ssa', 'vtt', 'unknown'];
  
  for (const format of formats) {
    const file: SubtitleFile = {
      format,
      entries: [],
    };
    expect(file.format).toBe(format);
  }
});

test('SubtitleTrack structure is valid', () => {
  const track: SubtitleTrack = {
    index: 0,
    codec: 'ass',
    language: 'eng',
    title: 'English',
  };
  
  expect(track.index).toBe(0);
  expect(track.codec).toBe('ass');
  expect(track.language).toBe('eng');
  expect(track.title).toBe('English');
});