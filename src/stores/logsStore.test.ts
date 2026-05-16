import { beforeEach, expect, test } from 'bun:test';

const { useLogsStore } = await import('./logsStore');

beforeEach(() => {
  useLogsStore.setState({
    logs: [],
    isOpen: false,
    filter: 'all',
    errorCount: 0,
  });
});

test('has correct initial values', () => {
  const state = useLogsStore.getState();
  expect(state.logs).toEqual([]);
  expect(state.isOpen).toBe(false);
  expect(state.filter).toBe('all');
  expect(state.errorCount).toBe(0);
});

test('addLog adds entry to beginning of logs array', () => {
  useLogsStore.getState().addLog('info', 'Message 1');
  useLogsStore.getState().addLog('warning', 'Message 2');

  const state = useLogsStore.getState();
  expect(state.logs.length).toBe(2);
  expect(state.logs[0].message).toBe('Message 2');
  expect(state.logs[1].message).toBe('Message 1');
});

test('addLog increments errorCount for error level', () => {
  useLogsStore.getState().addLog('info', 'Info message');
  useLogsStore.getState().addLog('error', 'Error message');
  useLogsStore.getState().addLog('success', 'Success message');

  const state = useLogsStore.getState();
  expect(state.errorCount).toBe(1);
});

test('addLog includes optional file and details', () => {
  useLogsStore.getState().addLog('error', 'Error message', 'file.srt', 'Detailed error info');

  const state = useLogsStore.getState();
  expect(state.logs[0].file).toBe('file.srt');
  expect(state.logs[0].details).toBe('Detailed error info');
});

test('addLog generates valid log entry structure', () => {
  useLogsStore.getState().addLog('info', 'Test message');

  const state = useLogsStore.getState();
  const entry = state.logs[0];
  
  expect(entry.id).toBeTruthy();
  expect(entry.timestamp).toBeInstanceOf(Date);
  expect(entry.timestampLabel).toBeTruthy();
  expect(entry.level).toBe('info');
  expect(entry.message).toBe('Test message');
});

test('clearLogs removes all logs and resets errorCount', () => {
  useLogsStore.getState().addLog('error', 'Error 1');
  useLogsStore.getState().addLog('error', 'Error 2');
  useLogsStore.getState().addLog('info', 'Info');

  useLogsStore.getState().clearLogs();

  const state = useLogsStore.getState();
  expect(state.logs).toEqual([]);
  expect(state.errorCount).toBe(0);
});

test('setFilter updates filter state', () => {
  useLogsStore.getState().setFilter('error');

  const state = useLogsStore.getState();
  expect(state.filter).toBe('error');
});

test('toggleDrawer flips isOpen state', () => {
  expect(useLogsStore.getState().isOpen).toBe(false);

  useLogsStore.getState().toggleDrawer();
  expect(useLogsStore.getState().isOpen).toBe(true);

  useLogsStore.getState().toggleDrawer();
  expect(useLogsStore.getState().isOpen).toBe(false);
});

test('openDrawer sets isOpen to true', () => {
  useLogsStore.getState().openDrawer();
  expect(useLogsStore.getState().isOpen).toBe(true);
});

test('closeDrawer sets isOpen to false', () => {
  useLogsStore.setState({ isOpen: true });
  useLogsStore.getState().closeDrawer();
  expect(useLogsStore.getState().isOpen).toBe(false);
});

test('logs are limited to 500 entries', () => {
  for (let i = 0; i < 505; i++) {
    useLogsStore.getState().addLog('info', `Message ${i}`);
  }

  const state = useLogsStore.getState();
  expect(state.logs.length).toBe(500);
});

test('errorCount decreases when error logs are removed due to limit', () => {
  useLogsStore.getState().addLog('error', 'Error 1');
  
  for (let i = 0; i < 505; i++) {
    useLogsStore.getState().addLog('info', `Message ${i}`);
  }

  const state = useLogsStore.getState();
  expect(state.logs.length).toBe(500);
  expect(state.errorCount).toBe(0);
});

test('timestampLabel uses Brazilian Portuguese format', () => {
  useLogsStore.getState().addLog('info', 'Test');

  const state = useLogsStore.getState();
  const label = state.logs[0].timestampLabel;
  
  expect(label).toMatch(/^\d{2}:\d{2}:\d{2}$/);
});