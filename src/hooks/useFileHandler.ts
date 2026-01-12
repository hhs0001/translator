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
    const validFiles = paths
      .map((path) => {
        const type = getFileType(path);
        if (!type) return null;
        const name = path.split(/[/\\]/).pop() || path;
        return { name, path, type };
      })
      .filter((f): f is NonNullable<typeof f> => f !== null);

    if (validFiles.length > 0) {
      addFiles(validFiles);
    }
  }, [addFiles]);

  const openFilePicker = useCallback(async () => {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      multiple: true,
      filters: [
        { name: 'Legendas', extensions: SUBTITLE_EXTENSIONS },
        { name: 'Vídeos', extensions: VIDEO_EXTENSIONS },
        { name: 'Todos suportados', extensions: [...SUBTITLE_EXTENSIONS, ...VIDEO_EXTENSIONS] },
      ],
    });

    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      handleFileDrop(paths);
    }
  }, [handleFileDrop]);

  return { handleFileDrop, openFilePicker };
}
