# Restora - Bug Tracker

## Bugs Conhecidos

### [BUG] Thumbnails de imagens não aparecem nos resultados
**Status:** ✅ Corrigido na v1.0.2  
**Severidade:** Média  
**Componente:** UI/Frontend

#### Descrição
Após recuperar arquivos de imagem, os cards na tela de resultados mostram apenas ícones genéricos (emoji ️) em vez de previews reais das imagens recuperadas.

#### Passos para Reproduzir
1. Executar scan em disco com imagens
2. Recuperar arquivos de imagem (JPG, PNG, etc.)
3. Navegar até a tela de resultados
4. Observar que os cards mostram apenas ícones, não previews

#### Causa Raiz
O componente `renderResults()` estava usando `getFileIcon(file.category)` que retornava apenas um emoji baseado na categoria, sem carregar a imagem real do arquivo.

#### Solução Implementada (v1.0.2)
- Criado componente `FileThumbnail` dedicado
- Usa `convertFileSrc()` do Tauri para gerar URLs válidas para arquivos locais
- Mostra preview real quando:
  - Arquivo é imagem (jpg, jpeg, png, gif, bmp, webp, tiff, heic)
  - Arquivo foi recuperado (tem `recovered_path` válido)
  - Imagem carrega sem erros
- Fallback para ícones SVG coloridos por categoria quando preview não está disponível
- Loading spinner durante carregamento da imagem
- Tratamento de erros com fallback graceful

#### Arquivos Modificados
- `src/components/FileThumbnail.tsx` (novo)
- `src/App.tsx` (integração do componente)
- `src/icons.tsx` (novos ícones SVG)
- `src/styles/global.css` (estilos do thumbnail)

---

### [BUG] Setup de 32-bit não incluído na release
**Status:** ✅ Corrigido na v1.0.2  
**Severidade:** Alta  
**Componente:** CI/CD (GitHub Actions)

#### Descrição
A release v1.0.0 e v1.0.1 não incluíam o instalador Windows 32-bit (`Restora-Setup-*.exe`), essencial para PCs antigos com menos de 4GB RAM.

#### Causa Raiz
O workflow usava `tauri-action` em paralelo, onde cada job criava releases separadas. O job de 32-bit criava uma release isolada que era sobrescrita ou não consolidada com os outros builds.

#### Solução Implementada (v1.0.2)
- Cada job agora faz build independente e upload de artifacts
- Job final `create-release` coleta todos os artifacts
- Usa globstar patterns (`artifacts/**/*.exe`) para encontrar todos os instaladores
- Single release com todos os arquivos de todas as plataformas

#### Arquivos Modificados
- `.github/workflows/build.yml` (refatorado completamente)

---

## Histórico de Versões

### v1.0.2 (2026-08-02)
- ✅ Corrigido: Preview de imagens nos resultados
- ✅ Corrigido: Setup 32-bit incluído na release (8 arquivos totais)
- ✅ Adicionado: Componente FileThumbnail com carregamento real de imagens
- ✅ Adicionado: Ícones SVG coloridos por categoria (fallback)
- ✅ Melhorado: Workflow de CI/CD com artifacts separados

### v1.0.1 (2026-08-01)
- ✅ Adicionado: Tema claro/escuro
- ✅ Adicionado: Splash screen com logo animado
- ✅ Corrigido: Scroll da interface

### v1.0.0 (2026-07-24)
- ✅ Release inicial
- ✅ Recuperação de arquivos rápida e profunda
- ✅ Suporte a múltiplos tipos de arquivo
- ✅ Reparo automático de arquivos danificados
- ✅ Filtro de thumbnails
