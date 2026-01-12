# Draft: UI Tauri Tradutor de Legendas

## Requirements (confirmed)
- App para traduzir legendas de anime/series com IA.
- Usuário define rota da API (ex: localhost:8045/v1) e buscar `/models` para listar modelos.
- Usuário pode selecionar modelo da lista ou inserir modelo custom.
- UI deve checar disponibilidade do `ffmpeg` via backend já existente.
- Usuário configura prompt de tradução e pode usar templates (CRUD completo em aba de configs).
- Usuário ajusta threshold de texto enviado (batch size) e auto-continue.
- Opção para continuar tradução automaticamente ou manual.
- Tela de tradução com modos single e batch (mesma lógica, muda quantidade/fila).
- Fluxo de tradução: extrair legenda via backend, ler arquivo, mostrar lado a lado original e tradução.
- Botão de traduzir na navbar; se resposta truncar, continuar (auto ou manual) até concluir.
- UI deve permitir API key e headers avançados (em dropdown/aba avançada).
- Configurações devem ser persistidas em arquivo no app data.
- Modelos devem atualizar automaticamente ao mudar a URL.
- Saída: usuário escolhe mux no vídeo ou arquivo separado; backup sempre.
- Manter estilo original do arquivo (ASS->ASS, etc) e destacar que ASS é melhor.
- VTT deve aparecer como "soon".
- UI deve ser completa e transparente (usuário entende o que está acontecendo e pode editar se necessário).
- Seleção de arquivos via drag-and-drop com fila visível (single/batch).
- Navbar deve ter área de logs (erros/warnings/status).
- Configs devem ter toggle de continuar/parar em erro + max retries.
- Configs devem ter ajuste de concorrência (n arquivos por vez).
- Plano deve ser separado: frontend e backend.
- Não incluir testes.
- UI deve usar HeroUI (Hero UI v3).
- Entregar plano em arquivo `.md` no repositório; aceitar sugestões.

## Resumo do Plano UI
- **Navegação**: navbar com tabs `Tradução` e `Configurações` e acesso aos logs.
- **Entrada de arquivos**: modos **single**/**batch** com drag-and-drop.
- **Fluxo de tradução**: fila com status e split view original/tradução.
- **Configurações**: API, modelo, prompt, templates e persistência de settings.
- **Resiliência**: auto-continue, retries e backup obrigatório.
- **Saída**: mux no vídeo ou arquivo separado, mantendo formato original.
- **Monitoramento**: drawer de logs com níveis e filtros.

## Technical Decisions
- Stack frontend atual: React + Vite + HeroUI + Tailwind (baseado em `src/main.tsx` e `src/App.tsx`).
- Persistência de templates já existe no backend via Tauri (commands `load_templates`, `add_template`, `update_template`, `delete_template`).
- Configuração deve ser persistida em arquivo no app data (mecanismo a definir na UI/plan).

## Research Findings
- Frontend inicial mínimo: `src/App.tsx` só renderiza um botão HeroUI.
- Backend Tauri expõe comandos para legendas, FFmpeg, tradução e templates em `src-tauri/src/lib.rs`.
- Tradução suporta batch/auto-continue via `translate_subtitle_full` e `continue_translation`.
- Não encontrei infra de testes JS/TS (sem scripts e configs em `package.json`).

## Open Questions
- Como lidar com limites de tamanho/token? (batch size e mensagens de erro)
- Qual comportamento esperado quando a API falhar ou `ffmpeg` não estiver disponível?
- Preferência sobre UX do batch: fila com status, reordenação, pausa/retomar?

## Scope Boundaries
- INCLUDE: Planejamento da UI (configuração, tradução single/batch, estados, layout, fluxos).
- EXCLUDE: Implementação do backend no plano de UI (backend terá plano separado).

## Test Strategy Decision
- **Infrastructure exists**: NÃO (frontend JS/TS). Existem testes Rust pontuais, mas sem runner JS.
- **User wants tests**: NÃO
- **Framework**: N/A
- **QA approach**: Manual (checklists detalhados)
