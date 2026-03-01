# GitHub Actions - Configuração de CI/CD

Este repositório utiliza GitHub Actions para automação de builds, testes e releases.

## Workflows Disponíveis

### 1. Lint & Type Check (`lint.yml`)
Executa validação de código em todos os PRs.

- **Frontend**: Verificação de tipos TypeScript
- **Backend**: Rust formatting, Clippy em múltiplas plataformas (Linux, macOS, Windows)
- **Disparado em**: Pull requests para `main`, `master`, `release`

### 2. Test Build (`test-build.yml`)
Compila a aplicação em todas as plataformas suportadas sem gerar release.

- **Plataformas**: macOS (ARM64 + x64), Ubuntu 22.04, Windows
- **Disparado em**: Pull requests para `main`, `master`, `release`
- **Propósito**: Validar que o código compila corretamente em todas as plataformas

### 3. Release (`release.yml`)
Cria releases automáticas com binários para todas as plataformas.

- **Plataformas**: macOS (ARM64 + x64), Ubuntu 22.04, Windows
- **Disparado em**: Push para branch `release` ou manualmente via workflow_dispatch
- **Gera**: 
  - Tag git automática (`vX.X.X`) baseada na versão do `package.json`
  - Changelog automático com contribuidores e histórico de commits

## Como Disparar uma Release

### Método 1: Push para branch `release`
```bash
git checkout release
git merge main
git push origin release
```

### Método 2: Manual (via GitHub)
1. Acesse a aba **Actions**
2. Selecione o workflow **Release**
3. Clique em **Run workflow**
4. Selecione a branch base e clique em **Run**

### Importante: Atualize a versão
Antes de fazer uma release, atualize a versão no `package.json`:
```json
{
  "version": "0.2.0"
}
```

O workflow会自动创建对应的 Git tag (`v0.2.0`) e usá-la para gerar o changelog desde a versão anterior.

## Formato da Release

A release incluirá:

- **Nome**: `Translator vX.X.X`
- **Tag Git**: `vX.X.X` (criada automaticamente)
- **Descrição**: Changelog automático contendo:
  - Lista de mudanças desde a versão anterior
  - Lista de contribuidores
  - Link para diff completo
- **Binários**:
  - `translator_X.X.X_macos_arm64.app.tar.gz` - macOS ARM64
  - `translator_X.X.X_macos_x64.app.tar.gz` - macOS x64
  - `translator_X.X.X_amd64.deb` - Linux
  - `translator_X.X.X_x64-setup.exe` - Windows

## Assinatura de Código (macOS)

### Modo Automático (sem certificado)
Por padrão, o build do macOS funciona **sem certificado**. O app será criado, mas não будет assinado. Isso é útil para:
- Testes internos
- Distribuição fora da App Store

### Modo Com Assinatura (opcional)
Para distribuir apps assinados, configure os seguintes secrets no repositório:

| Secret | Descrição |
|--------|-----------|
| `APPLE_CERTIFICATE` | Certificado de desenvolvedor codificado em Base64 (.p12) |
| `APPLE_CERTIFICATE_PASSWORD` | Senha do certificado |
| `APPLE_ID` | ID da Apple Developer |
| `APPLE_PASSWORD` | App Password específica para CI |
| `APPLE_TEAM_ID` | Team ID da sua conta Developer |
| `KEYCHAIN_PASSWORD` | Senha temporária para o keychain |

### Como configurar o certificado
1. Gere um certificado de "Developer ID Application" no site da Apple Developer
2. Exporte o certificado para `.p12`
3. Codifique em Base64:
   ```bash
   base64 -i certificado.p12 -o cert.txt
   ```
4. Adicione o conteúdo de `cert.txt` como secret `APPLE_CERTIFICATE`
5. Configure os outros secrets

## Configurações de Segurança

### Permissões
- `contents: write` - Necessário para criar e publicar releases e tags

### Segredos (Secrets)
O workflow utiliza:
- `GITHUB_TOKEN` - Disponível automaticamente
- Secrets opcionais para macOS signing (veja seção acima)

## Versionamento

A versão é lida automaticamente do arquivo `package.json`:
```json
{
  "version": "0.1.0"
}
```

**Importante**: A versão deve ser incrementada a cada release. Se a versão for igual à anterior, a tag não será criada novamente.

## Fluxo do Changelog

1. O workflow busca a tag mais recente
2. Gera lista de commits entre a tag anterior e a atual
3. Formata cada commit com: `* mensagem do commit (@autor)`
4. Lista contribuidores únicos
5. Gera link para diff completo

Exemplo de output:
```
## What's Changed

* Add Portuguese translation support (@johndoe)
* Fix subtitle parsing error (@janedoe)
* Improve batch processing performance (@bobsmith)

## Contributors

* @johndoe
* @janedoe
* @bobsmith

**Full Changelog**: https://github.com/user/repo/compare/v0.1.0...v0.2.0
```

## Troubleshooting

### Build falha no Ubuntu
Verifique se as dependências do sistema estão instaladas. O workflow inclui etapa automática para isso.

### Release não aparece
1. Verifique se o push foi para a branch `release`
2. Confirme que a versão no `package.json` foi incrementada
3. Verifique a aba **Actions** para logs de erro

### Tag não é criada
- Verifique se a versão no `package.json` é maior que a anterior
- Confirme que você tem permissões de escrita no repositório

### macOS build falha com erro de signing
- Se não deseja assinar, ignore os erros - o app ainda será gerado
- Para signing, confirme que todos os secrets estão configurados corretamente