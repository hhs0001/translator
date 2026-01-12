import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LLMModel } from '../types';
import { useSettingsStore } from '../stores/settingsStore';

export function useModels() {
  const [models, setModels] = useState<LLMModel[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { settings } = useSettingsStore();

  const fetchModels = useCallback(async () => {
    if (!settings.baseUrl) {
      setModels([]);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const result = await invoke<LLMModel[]>('list_llm_models', {
        config: {
          endpoint: settings.baseUrl,
          apiKey: settings.apiKey || '',
          model: settings.model || '',
          headers: settings.headers || [],
        }
      });
      setModels(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      setModels([]);
    } finally {
      setIsLoading(false);
    }
  }, [settings.baseUrl, settings.apiKey, settings.model, settings.headers]);

  // Auto-fetch when baseUrl changes
  useEffect(() => {
    const timer = setTimeout(fetchModels, 500); // Debounce
    return () => clearTimeout(timer);
  }, [fetchModels]);

  return { models, isLoading, error, refetch: fetchModels };
}
