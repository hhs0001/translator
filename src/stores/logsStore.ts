import { create } from 'zustand';
import { LogEntry, LogLevel } from '../types';

interface LogsState {
  logs: LogEntry[];
  isOpen: boolean;
  filter: LogLevel | 'all';
  errorCount: number;

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
  errorCount: 0,

  addLog: (level, message, file, details) => {
    const timestamp = new Date();
    const entry: LogEntry = {
      id: crypto.randomUUID(),
      timestamp,
      timestampLabel: timestamp.toLocaleTimeString('pt-BR', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      }),
      level,
      message,
      file,
      details,
    };
    set((state) => {
      const nextLogs = [entry, ...state.logs];
      let nextErrorCount = state.errorCount + (level === 'error' ? 1 : 0);
      if (nextLogs.length > 500) {
        const removed = nextLogs.pop();
        if (removed?.level === 'error') {
          nextErrorCount -= 1;
        }
      }
      return { logs: nextLogs, errorCount: nextErrorCount };
    });
  },

  clearLogs: () => set({ logs: [], errorCount: 0 }),

  setFilter: (filter) => set({ filter }),

  toggleDrawer: () => set((state) => ({ isOpen: !state.isOpen })),
  openDrawer: () => set({ isOpen: true }),
  closeDrawer: () => set({ isOpen: false }),
}));
