import { useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LLMModel, ApiFormat } from '../types';
import { useSettingsStore } from '../stores/settingsStore';

/**
 * Detecta o formato da API baseado na URL e configuração
 */
function detectApiFormat(baseUrl: string, configuredFormat: ApiFormat): ApiFormat {
  if (configuredFormat !== 'auto') {
    return configuredFormat;
  }

  const lower = baseUrl.toLowerCase();
  if (lower.includes('anthropic') || lower.endsWith('/messages') || lower.includes('/v1/messages')) {
    return 'anthropic';
  }
  return 'openai';
}

export function useModels() {
  const [models, setModels] = useState<LLMModel[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { settings } = useSettingsStore();

  // Detecta o formato baseado na URL e configuração
  const detectedFormat = useMemo(
    () => detectApiFormat(settings.baseUrl, settings.apiFormat),
    [settings.baseUrl, settings.apiFormat]
  );

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
          apiFormat: settings.apiFormat,
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
  }, [settings.baseUrl, settings.apiKey, settings.apiFormat, settings.model, settings.headers]);

  // Auto-fetch when baseUrl or apiFormat changes
  useEffect(() => {
    const timer = setTimeout(fetchModels, 500); // Debounce
    return () => clearTimeout(timer);
  }, [fetchModels]);

  return { models, isLoading, error, refetch: fetchModels, detectedFormat };
}
