import { beforeEach, expect, test, mock } from 'bun:test';
import type { QueueFile, SubtitleFile, SubtitleTranslationResult } from '../types';
import { DEFAULT_SETTINGS } from '../types';

const cancelTranslationCalls: string[] = [];
const cancelAllCalls: number[] = [];
const deleteFilesCalls: string[][] = [];
const loadSubtitleCalls: string[] = [];

const extractSubtitleTrackCalls: Array<{ videoPath: string; trackIndex: number; outputPath: string }> = [];
const muxSubtitleCalls: Array<{ videoPath: string; subtitlePath: string; outputPath: string }> = [];

let mockSubtitleFile: SubtitleFile;
let mockTranslationResult: SubtitleTranslationResult;

const cancelTranslation = async (fileId: string) => {
  cancelTranslationCalls.push(fileId);
};

const cancelAllTranslations = async () => {
  cancelAllCalls.push(1);
};

const loadSubtitle = async (path: string) => {
  loadSubtitleCalls.push(path);
  return mockSubtitleFile;
};

const translateSubtitleFull = async () => {
  return mockTranslationResult;
};

const saveSubtitle = async () => {
  return;
};

const extractSubtitleTrack = async (videoPath: string, trackIndex: number, outputPath: string) => {
  extractSubtitleTrackCalls.push({ videoPath, trackIndex, outputPath });
};

const muxSubtitleToVideo = async (videoPath: string, subtitlePath: string, outputPath: string) => {
  muxSubtitleCalls.push({ videoPath, subtitlePath, outputPath });
};

const deleteFiles = async (paths: string[]) => {
  deleteFilesCalls.push(paths);
  return paths;
};

mock.module('../utils/tauri', () => ({
  cancelTranslation,
  cancelAllTranslations,
  loadSubtitle,
  translateSubtitleFull,
  saveSubtitle,
  extractSubtitleTrack,
  muxSubtitleToVideo,
  deleteFiles,
}));

const { clearAllTranslatedIndices, useTranslationStore } = await import('./translationStore');
const { useSettingsStore } = await import('./settingsStore');

const makeFile = (overrides: Partial<QueueFile>): QueueFile => ({
  id: overrides.id ?? 'file-1',
  name: overrides.name ?? 'file.srt',
  path: overrides.path ?? '/tmp/file.srt',
  type: overrides.type ?? 'subtitle',
  status: overrides.status ?? 'pending',
  progress: overrides.progress ?? 0,
  totalLines: overrides.totalLines ?? 0,
  translatedLines: overrides.translatedLines ?? 0,
  ...overrides,
});

beforeEach(() => {
  cancelTranslationCalls.length = 0;
  cancelAllCalls.length = 0;
  deleteFilesCalls.length = 0;
  loadSubtitleCalls.length = 0;
  extractSubtitleTrackCalls.length = 0;
  muxSubtitleCalls.length = 0;
  clearAllTranslatedIndices();
  mockSubtitleFile = {
    format: 'ass',
    entries: [
      { index: 1, start_time: '00:00:01,000', end_time: '00:00:02,000', text: 'Hello' },
    ],
  };
  mockTranslationResult = {
    file: mockSubtitleFile,
    progress: {
      totalEntries: 1,
      translatedEntries: 1,
      lastTranslatedIndex: 0,
      isPartial: false,
      canContinue: false,
    },
  };
  useTranslationStore.setState({
    queue: [],
    currentFileId: null,
    isTranslating: false,
    isPaused: false,
  });
  useSettingsStore.setState({
    settings: { ...DEFAULT_SETTINGS },
  });
});

test('cancelFileTranslation marks a pending file as cancelled', async () => {
  const file = makeFile({ id: 'file-1', status: 'pending' });
  useTranslationStore.setState({
    queue: [file],
    currentFileId: file.id,
    isTranslating: true,
    isPaused: false,
  });

  await useTranslationStore.getState().cancelFileTranslation(file.id);

  const [updated] = useTranslationStore.getState().queue;
  expect(updated.status).toBe('cancelled');
  expect(cancelTranslationCalls.length).toBe(0);
});

test('cancelFileTranslation calls the native cancel for a processing file', async () => {
  const file = makeFile({ id: 'file-2', status: 'translating' });
  useTranslationStore.setState({
    queue: [file],
    currentFileId: file.id,
    isTranslating: true,
    isPaused: false,
  });

  await useTranslationStore.getState().cancelFileTranslation(file.id);

  const [updated] = useTranslationStore.getState().queue;
  expect(updated.status).toBe('cancelled');
  expect(cancelTranslationCalls).toEqual([file.id]);
});

test('cancelAllTranslations cancels pending and processing files', async () => {
  const pending = makeFile({ id: 'file-3', status: 'pending' });
  const translating = makeFile({ id: 'file-4', status: 'translating' });
  const completed = makeFile({ id: 'file-5', status: 'completed' });

  useTranslationStore.setState({
    queue: [pending, translating, completed],
    currentFileId: translating.id,
    isTranslating: true,
    isPaused: false,
  });

  await useTranslationStore.getState().cancelAllTranslations();

  const nextQueue = useTranslationStore.getState().queue;
  expect(nextQueue.find((file) => file.id === pending.id)?.status).toBe('cancelled');
  expect(nextQueue.find((file) => file.id === translating.id)?.status).toBe('cancelled');
  expect(nextQueue.find((file) => file.id === completed.id)?.status).toBe('completed');
  expect(useTranslationStore.getState().isTranslating).toBe(false);
  expect(useTranslationStore.getState().isPaused).toBe(false);
  expect(useTranslationStore.getState().currentFileId).toBeNull();
  expect(cancelAllCalls.length).toBe(1);
});

test('cleanupExtractedSubtitles deletes extracted subtitles for videos', async () => {
  useSettingsStore.setState({
    settings: {
      ...DEFAULT_SETTINGS,
      outputMode: 'separate',
      cleanupExtractedSubtitles: true,
      cleanupMuxArtifacts: false,
    },
  });

  const file = makeFile({
    id: 'file-6',
    name: 'video.mkv',
    path: '/tmp/video.mkv',
    type: 'video',
    selectedTrackIndex: 0,
  });

  useTranslationStore.setState({
    queue: [file],
    currentFileId: null,
    isTranslating: false,
    isPaused: false,
  });

  await useTranslationStore.getState().translateFile(file);

  expect(deleteFilesCalls.length).toBe(1);
  expect(deleteFilesCalls[0]).toEqual(['/tmp/video.extracted.ass']);
  expect(extractSubtitleTrackCalls.length).toBe(1);
  expect(loadSubtitleCalls).toEqual(['/tmp/video.extracted.ass']);
});

test('cleanupMuxArtifacts deletes translated and extracted files after mux', async () => {
  useSettingsStore.setState({
    settings: {
      ...DEFAULT_SETTINGS,
      outputMode: 'mux',
      cleanupExtractedSubtitles: false,
      cleanupMuxArtifacts: true,
    },
  });

  const file = makeFile({
    id: 'file-7',
    name: 'video.mkv',
    path: '/tmp/video.mkv',
    type: 'video',
    selectedTrackIndex: 0,
  });

  useTranslationStore.setState({
    queue: [file],
    currentFileId: null,
    isTranslating: false,
    isPaused: false,
  });

  await useTranslationStore.getState().translateFile(file);

  expect(deleteFilesCalls.length).toBe(1);
  const cleanupSet = new Set(deleteFilesCalls[0]);
  expect(cleanupSet).toEqual(new Set(['/tmp/video.translated.ass', '/tmp/video.extracted.ass']));
  expect(muxSubtitleCalls.length).toBe(1);
});
