import { useCallback } from 'react';
import { useTranslationStore } from '../stores/translationStore';

const SUBTITLE_EXTENSIONS = ['srt', 'ass', 'ssa'];
const VIDEO_EXTENSIONS = ['mkv', 'mp4', 'avi', 'webm', 'mov'];

function getFileType(path: string): 'subtitle' | 'video' | null {
  const ext = path.split('.').pop()?.toLowerCase();
  if (!ext) return null;
  if (SUBTITLE_EXTENSIONS.includes(ext)) return 'subtitle';
  if (VIDEO_EXTENSIONS.includes(ext)) return 'video';
  return null;
}

export function useFileHandler() {
  const addFiles = useTranslationStore((s) => s.addFiles);

  const handleFileDrop = useCallback((paths: string[]) => {
    console.log("[DEBUG] handleFileDrop called with paths:", paths);
    const validFiles = paths
      .map((path) => {
        const type = getFileType(path);
        console.log("[DEBUG] File:", path, "type:", type);
        if (!type) return null;
        const name = path.split(/[/\\]/).pop() || path;
        return { name, path, type };
      })
      .filter((f): f is NonNullable<typeof f> => f !== null);

    console.log("[DEBUG] Valid files:", validFiles);
    if (validFiles.length > 0) {
      console.log("[DEBUG] Calling addFiles with:", validFiles);
      addFiles(validFiles);
    } else {
      console.warn("[DEBUG] No valid files to add");
    }
  }, [addFiles]);

  const openFilePicker = useCallback(async () => {
    console.log("[DEBUG] openFilePicker called");
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      multiple: true,
      filters: [
        { name: 'Legendas', extensions: SUBTITLE_EXTENSIONS },
        { name: 'Vídeos', extensions: VIDEO_EXTENSIONS },
        { name: 'Todos suportados', extensions: [...SUBTITLE_EXTENSIONS, ...VIDEO_EXTENSIONS] },
      ],
    });

    console.log("[DEBUG] File picker returned:", selected);
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      console.log("[DEBUG] Paths to process:", paths);
      handleFileDrop(paths);
    } else {
      console.log("[DEBUG] No files selected");
    }
  }, [handleFileDrop]);

  return { handleFileDrop, openFilePicker };
}
