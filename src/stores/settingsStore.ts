import { create } from 'zustand';
import { AppSettings, DEFAULT_SETTINGS, Template } from '../types';
import * as TauriUtils from '../utils/tauri';

interface SettingsState {
  settings: AppSettings;
  templates: Template[];
  isLoading: boolean;
  ffmpegInstalled: boolean | null;

  loadSettings: () => Promise<void>;
  saveSettings: (settings: Partial<AppSettings>) => Promise<void>;
  updateSetting: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => void;

  loadTemplates: () => Promise<void>;
  addTemplate: (name: string, content: string) => Promise<void>;
  updateTemplate: (id: string, name: string, content: string) => Promise<void>;
  deleteTemplate: (id: string) => Promise<void>;

  checkFfmpeg: () => Promise<boolean>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  templates: [],
  isLoading: true,
  ffmpegInstalled: null,

  loadSettings: async () => {
    try {
      const settings = await TauriUtils.loadSettings();
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
      await TauriUtils.saveSettings(merged);
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  },

  updateSetting: (key, value) => {
    const newSettings = { ...get().settings, [key]: value };
    set({ settings: newSettings });
    TauriUtils.saveSettings(newSettings).catch(console.error);
  },

  loadTemplates: async () => {
    try {
      const templates = await TauriUtils.loadTemplates();
      set({ templates });
    } catch (error) {
      console.error('Failed to load templates:', error);
    }
  },

  addTemplate: async (name, content) => {
    try {
      await TauriUtils.addTemplate(name, content);
      await get().loadTemplates();
    } catch (error) {
      console.error('Failed to add template:', error);
      throw error;
    }
  },

  updateTemplate: async (id, name, content) => {
    try {
      await TauriUtils.updateTemplate(id, name, content);
      await get().loadTemplates();
    } catch (error) {
      console.error('Failed to update template:', error);
      throw error;
    }
  },

  deleteTemplate: async (id) => {
    try {
      await TauriUtils.deleteTemplate(id);
      await get().loadTemplates();
    } catch (error) {
      console.error('Failed to delete template:', error);
      throw error;
    }
  },

  checkFfmpeg: async () => {
    try {
      const result = await TauriUtils.checkFfmpegInstalled();
      const installed = result !== null && result !== '';
      set({ ffmpegInstalled: installed });
      return installed;
    } catch (error) {
      set({ ffmpegInstalled: false });
      return false;
    }
  },
}));
