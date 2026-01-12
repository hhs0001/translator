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
