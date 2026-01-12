import { invoke } from '@tauri-apps/api/core';
import { AppSettings } from '../types';
import { Template, LLMModel, SubtitleFile, SubtitleEntry, SubtitleTrack } from '../types';

export async function loadSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('load_settings');
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke('save_settings', { settings });
}

export async function loadTemplates(): Promise<Template[]> {
  return invoke<Template[]>('load_templates');
}

export async function addTemplate(name: string, content: string): Promise<Template> {
  return invoke('add_template', { name, content });
}

export async function updateTemplate(id: string, name: string, content: string): Promise<Template> {
  return invoke('update_template', { template_id: id, name, content });
}

export async function deleteTemplate(id: string): Promise<void> {
  return invoke('delete_template', { template_id: id });
}

export async function listLLMModels(baseUrl: string, apiKey: string): Promise<LLMModel[]> {
  return invoke<LLMModel[]>('list_llm_models', { config: { baseUrl, apiKey, headers: [], model: '', customModel: '' } });
}

export async function loadSubtitle(path: string): Promise<SubtitleFile> {
  return invoke<SubtitleFile>('load_subtitle', { path });
}

export async function saveSubtitle(path: string, file: SubtitleFile): Promise<void> {
  return invoke('save_subtitle', { path, file });
}

export async function checkFfmpegInstalled(): Promise<string> {
  return invoke<string>('check_ffmpeg_installed');
}

export async function listVideoSubtitleTracks(videoPath: string): Promise<SubtitleTrack[]> {
  return invoke<SubtitleTrack[]>('list_video_subtitle_tracks', { videoPath });
}

export async function extractSubtitleTrack(videoPath: string, trackIndex: number, outputPath?: string): Promise<void> {
  return invoke('extract_subtitle_track', { videoPath, trackIndex, outputPath: outputPath || '' });
}

export async function muxSubtitleToVideo(
  videoPath: string,
  subtitlePath: string,
  outputPath: string,
  language?: string,
  title?: string
): Promise<void> {
  return invoke('mux_subtitle_to_video', { videoPath, subtitlePath, outputPath, language, title });
}

export async function translateSubtitleFull(
  subtitle: SubtitleFile,
  prompt: string,
  baseUrl: string,
  apiKey: string,
  model: string,
  batchSize: number,
  headers: Record<string, string>
): Promise<{ translated_entries: SubtitleEntry[]; is_partial: boolean }> {
  return invoke('translate_subtitle_full', {
    config: {
      baseUrl,
      apiKey,
      headers: Object.entries(headers).map(([k, v]) => [k, v]),
      model,
      customModel: model,
    },
    system_prompt: prompt,
    file: subtitle,
    settings: {
      batch_size: batchSize,
      auto_continue: true,
      continue_on_error: true,
      max_retries: 3,
      concurrency: 1,
    },
  });
}

export async function backupFile(path: string): Promise<string> {
  return invoke('backup_file', { path });
}

export async function replaceFile(sourcePath: string, targetPath: string): Promise<void> {
  return invoke('replace_file', { sourcePath, targetPath });
}

export async function getFileInfo(path: string): Promise<{ path: string; filename: string; extension: string; size: number; is_video: boolean; is_subtitle: boolean }> {
  return invoke('get_file_info', { path });
}
