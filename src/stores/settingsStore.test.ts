import { beforeEach, expect, test } from 'bun:test';
import { DEFAULT_SETTINGS } from '../types';

const { useSettingsStore } = await import('./settingsStore');

beforeEach(() => {
  useSettingsStore.setState({
    settings: { ...DEFAULT_SETTINGS },
    templates: [],
    isLoading: true,
    ffmpegInstalled: null,
  });
});

test('has correct default values', () => {
  const state = useSettingsStore.getState();
  expect(state.settings).toEqual(DEFAULT_SETTINGS);
  expect(state.templates).toEqual([]);
  expect(state.isLoading).toBe(true);
  expect(state.ffmpegInstalled).toBeNull();
});

test('settings can be updated directly via setState', () => {
  useSettingsStore.setState({
    settings: {
      ...DEFAULT_SETTINGS,
      model: 'gpt-4o',
      batchSize: 100,
    },
  });

  const state = useSettingsStore.getState();
  expect(state.settings.model).toBe('gpt-4o');
  expect(state.settings.batchSize).toBe(100);
});

test('templates can be set directly via setState', () => {
  const mockTemplates = [
    { id: 'tpl-1', name: 'Template 1', content: 'Content 1' },
    { id: 'tpl-2', name: 'Template 2', content: 'Content 2' },
  ];

  useSettingsStore.setState({ templates: mockTemplates });

  const state = useSettingsStore.getState();
  expect(state.templates.length).toBe(2);
  expect(state.templates[0].name).toBe('Template 1');
});

test('isLoading can be updated via setState', () => {
  expect(useSettingsStore.getState().isLoading).toBe(true);

  useSettingsStore.setState({ isLoading: false });

  expect(useSettingsStore.getState().isLoading).toBe(false);
});

test('ffmpegInstalled can be updated via setState', () => {
  expect(useSettingsStore.getState().ffmpegInstalled).toBeNull();

  useSettingsStore.setState({ ffmpegInstalled: true });

  expect(useSettingsStore.getState().ffmpegInstalled).toBe(true);

  useSettingsStore.setState({ ffmpegInstalled: false });

  expect(useSettingsStore.getState().ffmpegInstalled).toBe(false);
});

test('settings partial update preserves unchanged fields', () => {
  useSettingsStore.setState({
    settings: {
      ...DEFAULT_SETTINGS,
      model: 'gpt-4o',
      batchSize: 100,
    },
  });

  useSettingsStore.setState({
    settings: {
      ...useSettingsStore.getState().settings,
      model: 'claude-3',
    },
  });

  const state = useSettingsStore.getState();
  expect(state.settings.model).toBe('claude-3');
  expect(state.settings.batchSize).toBe(100);
  expect(state.settings.apiKey).toBe(DEFAULT_SETTINGS.apiKey);
});

test('multiple template operations', () => {
  const templates = [
    { id: 'tpl-1', name: 'Template 1', content: 'Content 1' },
  ];

  useSettingsStore.setState({ templates });

  useSettingsStore.setState({
    templates: [...useSettingsStore.getState().templates, { id: 'tpl-2', name: 'Template 2', content: 'Content 2' }],
  });

  expect(useSettingsStore.getState().templates.length).toBe(2);

  const updatedTemplates = useSettingsStore.getState().templates.map(t =>
    t.id === 'tpl-1' ? { ...t, name: 'Updated' } : t
  );
  useSettingsStore.setState({ templates: updatedTemplates });

  expect(useSettingsStore.getState().templates[0].name).toBe('Updated');
  expect(useSettingsStore.getState().templates[1].name).toBe('Template 2');
});

test('filter state updates correctly', () => {
  expect(useSettingsStore.getState().isLoading).toBe(true);
  expect(useSettingsStore.getState().ffmpegInstalled).toBeNull();

  useSettingsStore.setState({ isLoading: false, ffmpegInstalled: true });

  const state = useSettingsStore.getState();
  expect(state.isLoading).toBe(false);
  expect(state.ffmpegInstalled).toBe(true);
});