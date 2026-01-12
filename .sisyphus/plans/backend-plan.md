# Plano Backend - Suporte à UI (Tauri)

## Objetivos
- Persistir configurações da UI em arquivo no app data.
- Facilitar uso de base URL para LLM + normalização para `/chat/completions`.
- Melhorar feedback de progresso/erros para logs da UI.
- Manter compatibilidade com comandos já existentes.

## Escopo
- **Inclui**: novos comandos e estruturas para settings, melhorias de observabilidade.
- **Exclui**: mudanças de UI (ver `ui-plan.md`).

## 1) Persistência de Settings
### Estrutura sugerida
- `AppSettings` (serde):
  - `base_url`, `api_key`, `headers: Vec<(String, String)>`
  - `model`, `custom_model`, `selected_template_id`
  - `prompt`
  - `batch_size`, `auto_continue`, `continue_on_error`, `max_retries`
  - `concurrency`
  - `output_mode` (mux/separado), `mux_language`, `mux_title`

### Comandos
- `load_settings(app: AppHandle) -> Result<AppSettings, String>`
- `save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String>`
- (Opcional) `reset_settings(app: AppHandle)`

### Arquivo
- `settings.json` no `app_data_dir` (padrão de `templates.json`).

## 2) Normalização do Endpoint LLM
- Função helper para aceitar **base URL** e derivar endpoint completo:
  - Se o usuário informar `/v1` → usar `/v1/chat/completions` na tradução.
  - Se informar `.../chat/completions` → manter.
- Alternativa: adicionar campo `endpoint` derivado no backend.

## 3) Feedback de Progresso e Logs
- Emitir eventos Tauri durante `translate_all_batched`:
  - `translation:progress` com `TranslationProgress`.
  - `translation:retry` e `translation:error` com detalhes (retry count).
- Isso alimenta o painel de logs da UI.

## 4) Continuação em Caso de Erro
- Ajustar fluxo para retornar resultados parciais quando ocorrer erro após retries.
- Opções:
  - Retornar `SubtitleTranslationResult` + `error_message`.
  - Novo comando `translate_subtitle_full_with_report`.

## 5) Modo de Saída e Backup
- Manter comando `backup_file` e garantir uso obrigatório pela UI.
- (Opcional) criar helper `backup_and_replace` para simplificar fluxo.

## QA Manual (sem testes)
- Salvar e carregar settings; validar arquivo no app data.
- Verificar que base URL gera `/models` correto e endpoint correto para tradução.
- Confirmar que eventos de progresso são emitidos durante tradução.
- Forçar erro de API e confirmar retorno de parcial + logs.
