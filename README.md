# Restora 🔄

<div align="center">
  <img src="logo.png" alt="Restora Logo" width="120" />
  <h3>File Recovery Tool - Minimalist & Lightweight</h3>
  <p>Recupere arquivos deletados de forma rápida e eficiente</p>
</div>

---

## ✨ Features

- **🔍 Scan Rápido e Profundo** - Escolha entre scan rápido ou profundo
- **📁 Múltiplos Tipos de Arquivo** - Imagens, Vídeos, Áudios, Documentos, e mais
- **🔧 Reparo Automático** - Recupera e repara arquivos danificados
- **🖼️ Filtro de Thumbnails** - Evita salvar imagens de preview
- **💻 Ultra Leve** - Funciona com apenas 2GB de RAM
- **🖥️ Compatível com PCs Antigos** - Suporte a 32-bit Windows
- **🎨 Interface Minimalista** - UI moderna e intuitiva
- **📂 Pasta de Destino** - Escolha onde salvar os arquivos recuperados
- **🗄️ Suporte a Múltiplos Discos** - Escaneie qualquer disco ou partição

## 🚀 Instalação

### Windows
- Baixe o `.exe` (64-bit) ou `.exe` (32-bit) da [página de releases](../../releases)
- Execute o instalador

### Linux
- Baixe o `.deb` para distribuições Debian/Ubuntu
- Baixe o `.AppImage` para outras distribuições
- Torne o AppImage executável: `chmod +x Restora*.AppImage`

### macOS
- Baixe o `.dmg` da [página de releases](../../releases)

## 🛠️ Development

### Requisitos
- Node.js 18+
- Rust 1.60+
- System dependencies para Tauri

### Setup
```bash
# Instalar dependências
npm install

# Rodar em desenvolvimento
npm run tauri dev

# Build para produção
npm run tauri build
```

## 📦 Build

### GitHub Actions
O projeto usa GitHub Actions para build automático em todas as plataformas:
- Windows: `.exe` (32-bit e 64-bit)
- Linux: `.deb` e `.AppImage`
- macOS: `.dmg`

Para criar uma release:
```bash
git tag v1.0.0
git push origin v1.0.0
```

## 🎯 Tipos de Arquivo Suportados

| Categoria | Formatos |
|-----------|----------|
| 🖼️ Imagens | JPG, PNG, GIF, BMP, WEBP, TIFF, CR2, HEIC |
| 🎬 Vídeos | MP4, AVI, MKV, MOV, FLV, WMV |
| 🎵 Áudio | MP3, WAV, FLAC, OGG, AAC, M4A |
| 📄 Documentos | PDF, DOC, DOCX, XLSX, PPTX, RTF, TXT |
| 📦 Arquivos | ZIP, RAR, 7Z, TAR, GZ |
| 📧 Emails | EML, MSG, PST |
| 🗄️ Banco de Dados | DB, SQL, SQLite, MDB |

## 📋 Opções Avançadas

- **Filtrar Thumbnails** - Não salva imagens de preview
- **Reparar Danificados** - Repara automaticamente arquivos corrompidos
- **Pular Duplicatas** - Não recupera arquivos repetidos
- **Preservar Estrutura** - Mantém estrutura de pastas original
- **Auto-Recuperar** - Recupera automaticamente ao encontrar
- **Recuperar Metadados** - Preserva data/hora e EXIF
- **Filtro de Tamanho** - Define tamanho mínimo e máximo

## 🏗️ Tecnologias

- **Frontend**: React + TypeScript + Vite
- **Backend**: Rust + Tauri
- **Build**: GitHub Actions (multi-platform)

## 📄 License

MIT © 2026 Restora

---

<div align="center">
  <p>Feito com ❤️ para recuperação de arquivos</p>
</div>
