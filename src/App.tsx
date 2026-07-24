import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";

interface DriveInfo {
  name: string;
  path: string;
  total_size: number;
  free_space: number;
  file_system: string;
  drive_type: string;
}

interface RecoveredFile {
  id: string;
  original_name: string;
  file_type: string;
  category: string;
  size: number;
  path: string;
  recovered_path: string;
  status: string;
  is_damaged: boolean;
  is_thumbnail: boolean;
  confidence: number;
  found_at: string;
}

interface ScanConfig {
  drive_path: string;
  scan_type: string;
  categories: string[];
  destination: string;
  filter_thumbnails: boolean;
  repair_damaged: boolean;
  max_file_size: number;
  min_file_size: number;
  skip_duplicates: boolean;
  preserve_structure: boolean;
  auto_recover: boolean;
  recover_metadata: boolean;
  verbose: boolean;
}

interface FileCategory {
  id: string;
  name: string;
  icon: string;
  extensions: string[];
}

type View = "home" | "scan" | "results" | "settings";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function App() {
  const [view, setView] = useState<View>("home");
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [selectedDrive, setSelectedDrive] = useState<DriveInfo | null>(null);
  const [categories, setCategories] = useState<FileCategory[]>([]);
  const [selectedCategories, setSelectedCategories] = useState<string[]>([]);
  const [destination, setDestination] = useState("");
  const [scanType, setScanType] = useState("fast");
  const [isScanning, setIsScanning] = useState(false);
  const [scanProgress, setScanProgress] = useState(0);
  const [results, setResults] = useState<RecoveredFile[]>([]);
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [toast, setToast] = useState<{ message: string; type: string } | null>(null);

  const [filterThumbnails, setFilterThumbnails] = useState(true);
  const [repairDamaged, setRepairDamaged] = useState(true);
  const [skipDuplicates, setSkipDuplicates] = useState(true);
  const [preserveStructure, setPreserveStructure] = useState(false);
  const [autoRecover, setAutoRecover] = useState(false);
  const [recoverMetadata, setRecoverMetadata] = useState(true);
  const [minFileSize, setMinFileSize] = useState(0);
  const [maxFileSize, setMaxFileSize] = useState(5000000000);
  const [showAdvanced, setShowAdvanced] = useState(false);

  useEffect(() => {
    loadDrives();
    loadCategories();
  }, []);

  const loadDrives = async () => {
    try {
      const drivesList = await invoke<DriveInfo[]>("get_available_drives");
      setDrives(drivesList);
    } catch (e) {
      console.error("Failed to load drives:", e);
      setDrives([
        { name: "Disco Local (C:)", path: "C:\\", total_size: 500000000000, free_space: 200000000000, file_system: "NTFS", drive_type: "local" },
        { name: "Disco Local (D:)", path: "D:\\", total_size: 1000000000000, free_space: 800000000000, file_system: "NTFS", drive_type: "local" },
        { name: "USB Drive (E:)", path: "E:\\", total_size: 32000000000, free_space: 16000000000, file_system: "FAT32", drive_type: "usb" },
      ]);
    }
  };

  const loadCategories = async () => {
    try {
      const cats = await invoke<FileCategory[]>("get_file_categories");
      setCategories(cats);
    } catch (e) {
      setCategories([
        { id: "all", name: "Todos os Arquivos", icon: "📁", extensions: ["*"] },
        { id: "images", name: "Imagens", icon: "🖼️", extensions: ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "cr2", "heic"] },
        { id: "videos", name: "Vídeos", icon: "🎬", extensions: ["mp4", "avi", "mkv", "mov", "flv", "wmv", "webm"] },
        { id: "audio", name: "Áudios", icon: "🎵", extensions: ["mp3", "wav", "flac", "ogg", "aac", "m4a", "wma"] },
        { id: "documents", name: "Documentos", icon: "", extensions: ["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "rtf"] },
        { id: "archives", name: "Arquivos Compactados", icon: "📦", extensions: ["zip", "rar", "7z", "tar", "gz"] },
      ]);
    }
  };

  const selectDestination = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Selecionar pasta de destino para arquivos recuperados",
      });
      if (selected) setDestination(selected as string);
    } catch (e) {
      console.error("Failed to select folder:", e);
    }
  };

  const toggleCategory = (id: string) => {
    if (id === "all") {
      setSelectedCategories(
        selectedCategories.length === categories.length - 1
          ? []
          : categories.filter(c => c.id !== "all").map(c => c.id)
      );
      return;
    }
    setSelectedCategories(prev =>
      prev.includes(id) ? prev.filter(c => c !== id) : [...prev, id]
    );
  };

  const startScan = async () => {
    if (!selectedDrive) { showToast("⚠️ Selecione um disco para escanear", "error"); return; }
    if (!destination) { showToast("⚠️ Selecione uma pasta de destino", "error"); return; }

    const config: ScanConfig = {
      drive_path: selectedDrive.path,
      scan_type: scanType,
      categories: selectedCategories,
      destination,
      filter_thumbnails: filterThumbnails,
      repair_damaged: repairDamaged,
      max_file_size: maxFileSize,
      min_file_size: minFileSize,
      skip_duplicates: skipDuplicates,
      preserve_structure: preserveStructure,
      auto_recover: autoRecover,
      recover_metadata: recoverMetadata,
      verbose: false,
    };

    setIsScanning(true);
    setView("scan");
    setResults([]);

    let progress = 0;
    const progressInterval = setInterval(() => {
      progress += Math.random() * 3;
      if (progress > 95) progress = 95;
      setScanProgress(progress);
    }, 400);

    try {
      const scanResults = await invoke<RecoveredFile[]>("start_scan", { config });
      clearInterval(progressInterval);
      setScanProgress(100);
      setResults(scanResults);
      setIsScanning(false);
      setTimeout(() => {
        setView("results");
        showToast(`✅ ${scanResults.length} arquivos encontrados!`, "success");
      }, 1000);
    } catch (e) {
      clearInterval(progressInterval);
      setIsScanning(false);
      showToast(`❌ Erro no scan: ${e}`, "error");
    }
  };

  const stopScan = async () => {
    try {
      await invoke("stop_scan");
      setIsScanning(false);
      setView("results");
    } catch (e) {
      console.error("Failed to stop scan:", e);
    }
  };

  const recoverSelected = async () => {
    if (selectedFiles.size === 0) { showToast("⚠️ Selecione arquivos para recuperar", "error"); return; }
    try {
      const recovered = await invoke<RecoveredFile[]>("recover_selected", { fileIds: Array.from(selectedFiles) });
      setResults(prev => prev.map(f => {
        const rec = recovered.find(r => r.id === f.id);
        return rec || f;
      }));
      showToast(`✅ ${recovered.length} arquivos recuperados!`, "success");
    } catch (e) {
      showToast(` Erro na recuperação: ${e}`, "error");
    }
  };

  const recoverAll = async () => {
    try {
      const recovered = await invoke<RecoveredFile[]>("recover_all");
      setResults(recovered);
      showToast(`✅ ${recovered.length} arquivos recuperados!`, "success");
    } catch (e) {
      showToast(`❌ Erro na recuperação: ${e}`, "error");
    }
  };

  const toggleFileSelection = (id: string) => {
    setSelectedFiles(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  };

  const selectAllFiles = () => {
    setSelectedFiles(selectedFiles.size === results.length ? new Set() : new Set(results.map(f => f.id)));
  };

  const showToast = (message: string, type: string) => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 4000);
  };

  const getDriveIcon = (type: string) => {
    switch (type) {
      case "local": return "";
      case "usb": return "";
      case "network": return "";
      case "removable": return "";
      case "home": return "";
      case "partition": return "";
      case "disk": return "";
      default: return "";
    }
  };

  const getFileIcon = (category: string) => {
    const icons: Record<string, string> = {
      images: "🖼️", videos: "", audio: "🎵",
      documents: "📄", archives: "", emails: "📧", databases: "🗄️",
    };
    return icons[category] || "📁";
  };

  const ToggleOption = ({ label, desc, value, onChange }: {
    label: string; desc: string; value: boolean; onChange: (v: boolean) => void;
  }) => (
    <div className="toggle-group">
      <div className="toggle-info">
        <span className="toggle-label">{label}</span>
        <span className="toggle-desc">{desc}</span>
      </div>
      <div className={`toggle-switch ${value ? "active" : ""}`} onClick={() => onChange(!value)} />
    </div>
  );

  const renderHome = () => (
    <div className="content-area">
      {/* DISK SELECTION */}
      <div className="section">
        <div className="section-header">
          <h2 className="section-title">
            <span className="section-icon"></span>
            Selecione o Disco
          </h2>
          <span className="section-badge">{drives.length} discos encontrados</span>
        </div>

        {drives.length === 0 ? (
          <div className="empty-state">
            <div className="icon"></div>
            <h3>Nenhum disco encontrado</h3>
            <p>Conecte um disco ou unidade e tente novamente.</p>
            <button className="btn btn-secondary" onClick={loadDrives} style={{ marginTop: 16 }}>
               Tentar novamente
            </button>
          </div>
        ) : (
          <div className="drives-grid">
            {drives.map((drive, idx) => {
              const isSelected = selectedDrive?.path === drive.path;
              const usedPercent = drive.total_size > 0
                ? Math.round(((drive.total_size - drive.free_space) / drive.total_size) * 100)
                : 0;

              return (
                <div
                  key={idx}
                  className={`drive-card ${isSelected ? "selected" : ""}`}
                  onClick={() => setSelectedDrive(drive)}
                >
                  <div className="drive-header">
                    <div className="drive-icon-wrapper">
                      <span className="drive-icon">{getDriveIcon(drive.drive_type)}</span>
                    </div>
                    {isSelected && <div className="drive-check">✓</div>}
                  </div>
                  <div className="drive-name">{drive.name}</div>
                  <div className="drive-path">{drive.path}</div>
                  <div className="drive-stats">
                    <div className="drive-stat">
                      <span className="drive-stat-label">Sistema</span>
                      <span className="drive-stat-value">{drive.file_system}</span>
                    </div>
                    <div className="drive-stat">
                      <span className="drive-stat-label">Tipo</span>
                      <span className="drive-stat-value">{drive.drive_type}</span>
                    </div>
                  </div>
                  <div className="drive-storage">
                    <div className="storage-info">
                      <span>{formatBytes(drive.total_size - drive.free_space)} usados</span>
                      <span>{100 - usedPercent}% livre</span>
                    </div>
                    <div className="drive-progress">
                      <div className={`drive-progress-bar ${isSelected ? "active" : ""}`} style={{ width: `${usedPercent}%` }} />
                    </div>
                    <div className="storage-detail">
                      <span style={{ color: "var(--text-muted)", fontSize: 11 }}>
                        Livre: {formatBytes(drive.free_space)} de {formatBytes(drive.total_size)}
                      </span>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* FILE TYPE */}
      <div className="section">
        <div className="section-header">
          <h2 className="section-title">
            <span className="section-icon"></span>
            Tipo de Arquivo
          </h2>
          {selectedCategories.length > 0 && (
            <span className="section-badge">{selectedCategories.length} selecionados</span>
          )}
        </div>
        <div className="category-grid">
          {categories.map(cat => {
            const isAll = cat.id === "all";
            const isActive = isAll
              ? selectedCategories.length === categories.length - 1
              : selectedCategories.includes(cat.id);
            return (
              <div
                key={cat.id}
                className={`category-item ${isActive ? "selected" : ""}`}
                onClick={() => toggleCategory(cat.id)}
              >
                <span className="cat-icon">{cat.icon}</span>
                <span className="cat-name">{cat.name}</span>
                {isActive && <span className="cat-check">✓</span>}
              </div>
            );
          })}
        </div>
      </div>

      {/* SCAN TYPE */}
      <div className="section">
        <h2 className="section-title">
          <span className="section-icon"></span>
          Modo de Varredura
        </h2>
        <div className="scan-type-grid">
          <div className={`scan-type-card ${scanType === "fast" ? "selected" : ""}`} onClick={() => setScanType("fast")}>
            <div className="scan-type-icon"></div>
            <div className="scan-type-name">Rápido</div>
            <div className="scan-type-desc">Analisa a estrutura de arquivos existente. Rápido e eficiente.</div>
            <div className="scan-type-time">~2-5 minutos</div>
          </div>
          <div className={`scan-type-card ${scanType === "deep" ? "selected" : ""}`} onClick={() => setScanType("deep")}>
            <div className="scan-type-icon"></div>
            <div className="scan-type-name">Profundo</div>
            <div className="scan-type-desc">Lê setor por setor buscando assinaturas de arquivos.</div>
            <div className="scan-type-time">~15-60 minutos</div>
          </div>
        </div>
      </div>

      {/* DESTINATION */}
      <div className="section">
        <h2 className="section-title">
          <span className="section-icon">📁</span>
          Pasta de Destino
        </h2>
        <div className="folder-input">
          <div className={`folder-input-display ${destination ? "filled" : ""}`}>
            {destination ? (
              <>
                <span className="folder-icon">📂</span>
                <span className="folder-path">{destination}</span>
              </>
            ) : (
              <span className="folder-placeholder">Nenhuma pasta selecionada...</span>
            )}
          </div>
          <button className="btn btn-secondary" onClick={selectDestination}>📂 Escolher</button>
        </div>
        {destination && (
          <div className="destination-info">
            <span>📍 Arquivos recuperados serão salvos em: <strong>{destination}</strong></span>
          </div>
        )}
      </div>

      {/* ADVANCED */}
      <div className="section">
        <button className="btn btn-secondary advanced-toggle" onClick={() => setShowAdvanced(!showAdvanced)}>
          <span className={`advanced-arrow ${showAdvanced ? "open" : ""}`}>▶</span>
          Opções Avançadas
        </button>
        {showAdvanced && (
          <div className="card advanced-card">
            <ToggleOption label=" Filtrar Thumbnails" desc="Não salvar imagens de preview e thumbnails pequenas" value={filterThumbnails} onChange={setFilterThumbnails} />
            <ToggleOption label="🔧 Reparar Arquivos Danificados" desc="Tentar reparar automaticamente arquivos corrompidos" value={repairDamaged} onChange={setRepairDamaged} />
            <ToggleOption label="📋 Pular Duplicatas" desc="Não recuperar arquivos já existentes no destino" value={skipDuplicates} onChange={setSkipDuplicates} />
            <ToggleOption label="📂 Preservar Estrutura de Pastas" desc="Manter a estrutura de diretórios original" value={preserveStructure} onChange={setPreserveStructure} />
            <ToggleOption label="🚀 Auto-Recuperar" desc="Recuperar automaticamente ao encontrar arquivos" value={autoRecover} onChange={setAutoRecover} />
            <ToggleOption label="📊 Recuperar Metadados (EXIF, datas)" desc="Preservar data, hora e informações EXIF dos arquivos" value={recoverMetadata} onChange={setRecoverMetadata} />
            <div className="scan-config">
              <div className="form-group">
                <label className="form-label">Tamanho Mínimo (bytes)</label>
                <input type="number" className="form-input" value={minFileSize} onChange={(e) => setMinFileSize(Number(e.target.value))} placeholder="0" />
              </div>
              <div className="form-group">
                <label className="form-label">Tamanho Máximo (bytes)</label>
                <input type="number" className="form-input" value={maxFileSize} onChange={(e) => setMaxFileSize(Number(e.target.value))} placeholder="5000000000" />
              </div>
            </div>
          </div>
        )}
      </div>

      {/* START SCAN */}
      <div className="start-scan-section">
        <button
          className="btn btn-primary btn-lg start-scan-btn"
          onClick={startScan}
          disabled={!selectedDrive || !destination || isScanning}
        >
          {isScanning ? (<><div className="spinner" /> Escaneando...</>) : (<>🔍 Iniciar Recuperação</>)}
        </button>
        {!selectedDrive && <p className="hint-text">️ Selecione um disco acima para começar</p>}
        {selectedDrive && !destination && <p className="hint-text">📂 Selecione uma pasta de destino para começar</p>}
      </div>
    </div>
  );

  const renderScan = () => (
    <div className="content-area">
      <div className="scan-progress-card">
        <div className="scan-progress-header">
          <div>
            <h3>{isScanning ? "🔍 Escaneando..." : "✅ Varredura Completa"}</h3>
            <p className="scan-drive-name">Disco: {selectedDrive?.name} ({selectedDrive?.path})</p>
          </div>
          <div className="scan-progress-percent">{Math.round(scanProgress)}%</div>
        </div>
        <div className="progress-bar-container">
          <div className="progress-bar" style={{ width: `${scanProgress}%` }} />
        </div>
        <div className="progress-stats">
          <div className="stat-card">
            <div className="stat-value">{results.length}</div>
            <div className="stat-label">Arquivos</div>
          </div>
          <div className="stat-card">
            <div className="stat-value">{formatBytes(results.reduce((a, f) => a + f.size, 0))}</div>
            <div className="stat-label">Total</div>
          </div>
          <div className="stat-card">
            <div className="stat-value">{results.filter(f => f.is_damaged).length}</div>
            <div className="stat-label">Danificados</div>
          </div>
          <div className="stat-card">
            <div className="stat-value">{scanType === "deep" ? "Profundo" : "Rápido"}</div>
            <div className="stat-label">Modo</div>
          </div>
        </div>
      </div>

      {isScanning && (
        <div className="scanning-indicator">
          <div className="spinner" />
          <p>Analisando {selectedDrive?.path}...</p>
          <button className="btn btn-danger" onClick={stopScan}>⏹ Parar Scan</button>
        </div>
      )}

      {!isScanning && results.length > 0 && (
        <div className="scan-complete-actions">
          <button className="btn btn-primary btn-lg" onClick={() => setView("results")}>📋 Ver Resultados →</button>
        </div>
      )}
    </div>
  );

  const renderResults = () => (
    <div className="content-area">
      <div className="results-toolbar">
        <div>
          <h2 className="section-title" style={{ margin: 0 }}>
             Arquivos Encontrados
            <span className="badge">{results.length}</span>
          </h2>
        </div>
        <div className="results-actions">
          <button className="btn btn-sm btn-secondary" onClick={selectAllFiles}>
            {selectedFiles.size === results.length ? "☐ Desmarcar" : "☑ Selecionar Tudo"}
          </button>
          <button className="btn btn-sm btn-primary" onClick={recoverSelected} disabled={selectedFiles.size === 0}>
             Recuperar ({selectedFiles.size})
          </button>
          <button className="btn btn-sm btn-primary" onClick={recoverAll}> Recuperar Tudo</button>
        </div>
      </div>

      {results.length === 0 ? (
        <div className="empty-state">
          <div className="icon"></div>
          <h3>Nenhum arquivo encontrado</h3>
          <p>Tente um scan profundo para encontrar mais arquivos.</p>
          <button className="btn btn-primary" onClick={() => setView("home")} style={{ marginTop: 16 }}>Voltar ao Início</button>
        </div>
      ) : (
        <div className="results-grid">
          {results.map(file => (
            <div key={file.id} className={`file-card ${selectedFiles.has(file.id) ? "selected" : ""}`} onClick={() => toggleFileSelection(file.id)}>
              <div className="file-checkbox">
                <div className={`checkbox ${selectedFiles.has(file.id) ? "checked" : ""}`} />
              </div>
              <div className="file-thumbnail">{getFileIcon(file.category)}</div>
              <div className="file-info">
                <div className="file-name" title={file.original_name}>{file.original_name}</div>
                <div className="file-meta">
                  <span className="file-size">{formatBytes(file.size)}</span>
                  <span className={`file-status ${file.status}`}>
                    {file.status === "found" && "Encontrado"}
                    {file.status === "damaged" && "Danificado"}
                    {file.status === "recovered" && "Recuperado"}
                    {file.status === "repaired" && "Reparado"}
                    {file.status === "recovered_damaged" && "Rec. Danificado"}
                  </span>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );

  const renderSettings = () => (
    <div className="content-area">
      <div className="section">
        <h2 className="section-title">
          <span className="section-icon">⚙️</span>
          Configurações
        </h2>
        <div className="card">
          <ToggleOption label="🎨 Tema Escuro" desc="Usar tema escuro na interface" value={true} onChange={() => {}} />
          <ToggleOption label="🔔 Notificações" desc="Receber notificações ao completar operações" value={true} onChange={() => {}} />
          <ToggleOption label="💾 Salvar Configurações" desc="Lembrar configurações entre sessões" value={true} onChange={() => {}} />
          <ToggleOption label="🔍 Preview de Arquivos" desc="Mostrar miniatura dos arquivos encontrados" value={true} onChange={() => {}} />
          <ToggleOption label="⚡ Modo Leve" desc="Reduzir uso de memória (ideal para PCs antigos)" value={true} onChange={() => {}} />
          <ToggleOption label="🔄 Verificar Integridade" desc="Verificar hash dos arquivos após recuperação" value={true} onChange={() => {}} />
        </div>
      </div>
      <div className="section">
        <h2 className="section-title">
          <span className="section-icon">ℹ️</span>
          Sobre o Restora
        </h2>
        <div className="card about-card">
          <div className="about-logo">
            <img src="/logo.png" alt="Restora" className="about-logo-img" />
          </div>
          <div className="about-info">
            <h3 className="about-title">Restora</h3>
            <p className="about-version">Versão 1.0.0</p>
            <p className="about-desc">
              Ferramenta de recuperação de arquivos leve e minimalista.
              Encontra e restaura arquivos deletados de qualquer disco ou unidade de armazenamento.
            </p>
            <div className="about-features">
              <span> Rápido</span>
              <span>🖥️ 2GB RAM</span>
              <span>🔧 Reparo</span>
              <span>📱 32-bit</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );

  return (
    <div className="app-container">
      <div className="sidebar">
        <div className="sidebar-logo">
          <img src="/logo.png" alt="Restora" className="logo-img" />
          <div className="logo-text">
            <h1>Restora</h1>
            <span>File Recovery v1.0</span>
          </div>
        </div>
        <div className="sidebar-nav">
          <button className={`nav-item ${view === "home" ? "active" : ""}`} onClick={() => setView("home")}>
            <span className="icon">🏠</span> Início
          </button>
          <button className={`nav-item ${view === "scan" ? "active" : ""}`} onClick={() => setView("scan")}>
            <span className="icon">🔍</span> Escanear
          </button>
          <button className={`nav-item ${view === "results" ? "active" : ""}`} onClick={() => setView("results")}>
            <span className="icon">📋</span> Resultados
            {results.length > 0 && <span className="nav-badge">{results.length}</span>}
          </button>
          <button className={`nav-item ${view === "settings" ? "active" : ""}`} onClick={() => setView("settings")}>
            <span className="icon">⚙️</span> Configurações
          </button>
        </div>
        <div className="sidebar-footer">
          {selectedDrive && (
            <div className="selected-drive-info">
              <span className="drive-dot" />
              <span>{selectedDrive.name}</span>
            </div>
          )}
          <p>Restora © 2026</p>
          <p>Leve & Rápido • 2GB RAM </p>
        </div>
      </div>

      <div className="main-content">
        <div className="header">
          <span className="header-title">
            {view === "home" && "🏠 Recuperação de Arquivos"}
            {view === "scan" && "🔍 Escaneamento em Andamento"}
            {view === "results" && "📋 Resultados da Varredura"}
            {view === "settings" && "⚙️ Configurações"}
          </span>
          <div className="header-actions">
            {view === "home" && selectedDrive && (
              <span className="header-drive">💽 {selectedDrive.name}</span>
            )}
            {view === "results" && (
              <button className="btn btn-sm btn-secondary" onClick={() => setView("home")}>← Novo Scan</button>
            )}
          </div>
        </div>
        {view === "home" && renderHome()}
        {view === "scan" && renderScan()}
        {view === "results" && renderResults()}
        {view === "settings" && renderSettings()}
      </div>

      {toast && (
        <div className={`toast ${toast.type}`}>
          <span>{toast.type === "success" ? "✅" : "❌"}</span>
          <span>{toast.message}</span>
        </div>
      )}
    </div>
  );
}

export default App;
