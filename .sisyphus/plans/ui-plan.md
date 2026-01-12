# Plano UI - Tradutor de Legendas (Tauri)

## Contexto
- Frontend atual: React + Vite + HeroUI + Tailwind (`src/main.tsx`, `src/App.tsx`).
- Backend Tauri já expõe comandos para legendas, ffmpeg, tradução e templates em `src-tauri/src/lib.rs`.
- Objetivo é uma UI completa, transparente e orientada a fluxo (configuração → tradução → salvar).

## Objetivos
- Prover telas de configuração e tradução (single e batch).
- Permitir configuração de API/modelos/prompt/templates, batch size e auto-continue.
- Garantir visibilidade do processo (logs, progresso, status por arquivo).
- Suportar formatos mantendo o original (ASS → ASS), com destaque para ASS.
- Salvar output com opção de mux ou arquivo separado + backup sempre.

## Escopo
- **Inclui**: UI/UX, navegação, estados, integrações com comandos Tauri, persistência de settings via backend.
- **Exclui**: Mudanças no backend (ver `backend-plan.md`).

## Comandos Tauri já disponíveis (para UI)
- Legendas: `load_subtitle`, `save_subtitle`, `detect_subtitle_format`.
- FFmpeg: `check_ffmpeg_installed`, `list_video_subtitle_tracks`, `extract_subtitle_track`, `mux_subtitle_to_video`.
- Tradução: `list_llm_models`, `translate_subtitle_full`, `translate_subtitle_batch`, `continue_translation`, `translate_text`.
- Templates: `load_templates`, `add_template`, `update_template`, `delete_template`.
- Utilitários: `get_file_info`, `backup_file`, `replace_file`, `delete_files`.

## Arquitetura de UI
### Navegação
- **Navbar fixa** com: título, botão “Traduzir”, indicador de status global, botão “Logs”.
- **Rotas/Tabs principais**: `Tradução` e `Configurações`.

### Componentes HeroUI sugeridos
- `Tabs`, `Card`, `Input`, `Textarea`, `Select`, `Table`, `Switch`, `Button`, `Chip`, `Progress`, `Accordion`/`Collapse`, `Modal`, `Drawer`/`Sheet` (logs).
- Manter layout em cards por seção com ícones e descrições curtas.

## Tela de Configurações
### 1) Conexão com API
- **Base URL** (ex: `http://localhost:8045/v1`).
- **Auto-refresh** da lista de modelos ao mudar URL.
- **Modelo**: Select com lista + campo “custom model”.
- **API Key**: campo obrigatório/visível.
- **Headers avançados**: dropdown/accordion com lista key/value (add/remover).
- **Normalização do endpoint**: salvar base URL e derivar `.../chat/completions` para traduções.

### 2) Verificação FFmpeg
- Botão “Verificar FFmpeg” + Chip de status.
- Mensagem de erro clara quando não estiver instalado.

### 3) Prompt de Tradução
- Textarea de prompt.
- Seletor de template com botão “Aplicar no prompt”.
- Preview simples do prompt final (opcional).

### 4) Templates (CRUD completo)
- Tabela/lista com nome, ações (editar, duplicar, excluir).
- Modal para criar/editar (nome + conteúdo).
- Persistência via `load_templates`, `add_template`, `update_template`, `delete_template`.

### 5) Tradução e Retentativas
- **Batch size** com presets (50/100/150) + input numérico.
- **Auto-continue** toggle.
- **Continuar em erro** toggle (se desativado, parar no erro).
- **Max retries** input.
- **Concorrência**: número de arquivos processados simultaneamente (1..N).

### 6) Saída/Exportação
- Modo de saída:
  - **Mux no vídeo** (mantém legendas antigas).
  - **Arquivo separado** (em pasta escolhida).
- **Backup sempre ativo** (informar ao usuário).
- Campos adicionais para mux: idioma/título da faixa (opcional).

### 7) Formatos
- Mostrar badges de suporte: **SRT/ASS/SSA (OK)**, **VTT (Soon)**.
- Aviso: “ASS preserva melhor estilo e formatação”.

## Tela de Tradução
### Modo Single e Batch
- Tabs: **Single** e **Batch**.
- Área de **drag-and-drop**.
- **Fila visível** com status por arquivo: Pendente / Traduzindo / Erro / Concluído.
- Reordenação opcional + botão “Pausar/Retomar fila”.

### Fluxo Single
1. Drag‑and‑drop de **arquivo de legenda** ou **arquivo de vídeo**.
2. Se vídeo → listar faixas (`list_video_subtitle_tracks`) e extrair (`extract_subtitle_track`).
3. Carregar legenda (`load_subtitle`) e exibir resumo (formato, quantidade de linhas).
4. Botão “Traduzir” no navbar inicia a tradução.

### Fluxo Batch
1. Drag‑and‑drop múltiplos arquivos.
2. Montar fila com status e progress.
3. Processar com concorrência configurada.

### Visualização/edição
- **Split view**: original (esquerda) vs tradução (direita).
- Scroll sincronizado e highlights por linha.
- Tradução **editável** (campo à direita).
- Contagem de progresso: X/Y linhas traduzidas.

### Continuação de Tradução
- Se resposta parcial: mostrar status `is_partial`.
- Se auto-continue ligado → continuar automaticamente.
- Se desligado → botão “Continuar tradução”.
- Respeitar `max_retries` e registrar erros em logs.

### Salvar
- **Backup obrigatório** via `backup_file`.
- Se modo **mux** → chamar `mux_subtitle_to_video`.
- Se **arquivo separado** → `save_subtitle`.
- Se “substituir original” → usar `replace_file` (com backup prévio).

## Logs e Transparência
- **Drawer de Logs** na navbar com entradas: info, warning, error.
- Cada entrada inclui timestamp, arquivo, etapa e mensagem.
- Ações: copiar, limpar, filtrar por nível.

## Estado e Persistência (UI)
- Settings salvos em arquivo (via backend):
  - `baseUrl`, `apiKey`, `headers[]`
  - `model`, `customModel`
  - `prompt`, `selectedTemplateId`
  - `batchSize`, `autoContinue`, `continueOnError`, `maxRetries`
  - `concurrency`
  - `outputMode` (mux/separado), `muxLanguage`, `muxTitle`
- Carregar settings no boot e refletir imediatamente na UI.

## UX e Acessibilidade
- Desabilitar botões durante requests ativas.
- Exibir spinner/Progress para batch e single.
- Mostrar toasts para sucesso/erro.
- Mensagens claras ao detectar `ffmpeg` ausente.

## QA Manual (sem testes)
- Configurar API base + API key → modelos carregam automaticamente.
- Criar/editar/remover template → persistência correta.
- Check FFmpeg → feedback visível.
- Tradução single e batch → fila e progress atualizam.
- Interrupção/erro → logs preenchidos + respeita continue/stop.
- Salvar com mux e com arquivo separado → backup sempre criado.

## Sugestões (opcional)
- Wizard inicial de configuração (primeiro uso).
- Presets de templates (anime, shows, formal, informal).
- Atalhos de teclado para iniciar/pausar tradução.
- Pré-visualização rápida da primeira linha traduzida.
