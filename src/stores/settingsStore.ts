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
