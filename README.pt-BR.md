# Translator (GPUI)

[English](README.md) | Português (Brasil)

Translator é um app desktop para traduzir arquivos de legenda (e legendas embutidas em vídeos) usando APIs de LLM. É um app nativo em Rust feito com [GPUI](https://gpui.rs) e [gpui-component](https://longbridge.github.io/gpui-component/), e foca em fluxos rápidos de tradução em lote com edição final e opções flexíveis de saída.

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
- Vídeos: MKV, MP4, AVI, MOV, WEBM, M4V, TS (extração de legendas via FFmpeg).

## Download

Binários pré-compilados para todas as plataformas estão disponíveis na página de [Releases](https://github.com/hhs0001/translator/releases).

### Plataformas suportadas

| Plataforma | Arquitetura           | Arquivo     |
| ---------- | --------------------- | ----------- |
| Windows    | x64                   | `.exe`      |
| macOS      | ARM64 (Apple Silicon) | binário     |
| macOS      | x64 (Intel)           | binário     |
| Linux      | AMD64                 | binário     |

## Requisitos (para desenvolvimento)

- **Rust** 1.85+ (stable).
- **FFmpeg** no `PATH` para extrair/muxar legendas de vídeo.
- Um endpoint de API LLM (compatível com OpenAI ou Anthropic).

No Linux também são necessárias as bibliotecas de sistema do GPUI, em geral:

```bash
sudo apt-get install -y \
  libxkbcommon-dev libwayland-dev libvulkan-dev \
  libx11-dev libxrandr-dev libxi-dev libxcursor-dev libxinerama-dev \
  libssl-dev cmake pkg-config libfontconfig-dev
```

## Desenvolvimento

```bash
cargo run
```

## Build

```bash
cargo build --release
```

O binário fica em `target/release/translator` (ou `translator.exe` no Windows).

## Testes

```bash
cargo test
```

## Estrutura do projeto

```
src/main.rs      # entrada GPUI
src/app.rs       # RootView (navbar + páginas)
src/core/        # lógica (legenda, FFmpeg, LLM, settings)
src/state/       # entities GPUI (fila, settings, logs)
src/views/       # páginas da UI
assets/i18n/     # strings en / pt-BR
```

## CI/CD

Este projeto usa GitHub Actions para builds e releases automatizados:

- **Lint**: `cargo fmt` e `cargo clippy` no Ubuntu e no Windows
- **Test Build**: `cargo test` e `cargo build --release` no macOS, Ubuntu e Windows
- **Release**: binários cargo para todas as plataformas

Para criar uma nova release, atualize a versão no `Cargo.toml` e faça merge para a branch `release`.
