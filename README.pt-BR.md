# Translator (Tauri + React)

[English](README.md) | Português (Brasil)

Translator é um app desktop para traduzir arquivos de legenda (e legendas embutidas em vídeos) usando APIs de LLM. Ele foi feito com Tauri, React e Vite, e foca em fluxos rápidos de tradução em lote com edição final e opções flexíveis de saída.

## Recursos

- Fila com drag-and-drop para arquivos de legenda e vídeos, em modos single e batch.
- Editor de legendas com progresso ao vivo e ajuste linha a linha.
- Configuração da API com detecção automática de formato (OpenAI/Anthropic) e listagem de modelos.
- Templates de prompt e editor de prompt para padronizar o estilo de tradução.
- Controle de batch, paralelismo de requisições, retentativas e concorrência por arquivo.
- Detecção opcional de idioma para preencher metadados de mux automaticamente.
- Extração e mux de legendas em vídeos via FFmpeg.
- Saída em arquivo separado ou muxado em MKV.

## Formatos suportados

- Legendas: SRT, ASS, SSA (VTT em breve).
- Vídeos: MKV, MP4, AVI (extração de legendas via FFmpeg).

## Requisitos

- **Bun** (recomendado) ou outro gerenciador Node.js.
- **Rust + Tauri CLI** para builds desktop.
- **FFmpeg** para extrair/muxar legendas de vídeo.
- Um endpoint de API LLM (compatível com OpenAI ou Anthropic).

## Desenvolvimento

```bash
bun install
bun run dev
```

## Rodar o app desktop (Tauri)

```bash
bun run tauri dev
```

## Build

```bash
bun run tauri build
```

## Estrutura do projeto

```text
src/           # UI em React (configurações, tradução, editor)
src-tauri/     # Backend Tauri (FFmpeg, parser de legenda, chamadas LLM)
```
