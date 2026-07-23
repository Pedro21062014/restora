import React, { useState, useEffect, useCallback } from "react";
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

  // Advanced options
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
      if (drivesList.length > 0) {
        setSelectedDrive(drivesList[0]);
      }
    } catch (e) {
      console.error("Failed to load drives:", e);
    }
  };

  const loadCategories = async () => {
    try {
      const cats = await invoke<FileCategory[]>("get_file_categories");
      setCategories(cats);
    } catch (e) {
      console.error("Failed to load categories:", e);
    }
  };

  const selectDestination = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Selecionar pasta de destino",
      });
      if (selected) {
        setDestination(selected as string);
      }
    } catch (e) {
      console.error("Failed to select folder:", e);
    }
  };

  const toggleCategory = (id: string) => {
    if (id === "all") {
      if (selectedCategories.length === categories.length - 1) {
        setSelectedCategories([]);
      } else {
        setSelectedCategories(categories.filter(c => c.id !== "all").map(c => c.id));
      }
      return;
    }

    setSelectedCategories(prev => {
      if (prev.includes(id)) {
        return prev.filter(c => c !== id);
      } else {
        return [...prev, id];
      }
    });
  };

  const startScan = async () => {
    if (!selectedDrive) {
      showToast("Selecione um disco para escanear", "error");
      return;
    }
    if (!destination) {
      showToast("Selecione uma pasta de destino", "error");
      return;
    }

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

    // Simulate progress
    let progress = 0;
    const progressInterval = setInterval(() => {
      progress += Math.random() * 5;
      if (progress > 95) progress = 95;
      setScanProgress(progress);
    }, 300);

    try {
      const scanResults = await invoke<RecoveredFile[]>("start_scan", { config });
      clearInterval(progressInterval);
      setScanProgress(100);
      setResults(scanResults);
      setIsScanning(false);

      setTimeout(() => {
        setView("results");
        showToast(`${scanResults.length} arquivos encontrados!`, "success");
      }, 1000);
    } catch (e) {
      clearInterval(progressInterval);
      setIsScanning(false);
      showToast(`Erro no scan: ${e}`, "error");
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
    if (selectedFiles.size === 0) {
      showToast("Selecione arquivos para recuperar", "error");
      return;
    }

    try {
      const recovered = await invoke<RecoveredFile[]>("recover_selected", {
        fileIds: Array.from(selectedFiles),
      });
      setResults(prev => prev.map(f => {
        const rec = recovered.find(r => r.id === f.id);
        return rec || f;
      }));
      showToast(`${recovered.length} arquivos recuperados com sucesso!`, "success");
    } catch (e) {
      showToast(`Erro na recuperação: ${e}`, "error");
    }
  };

  const recoverAll = async () => {
    try {
      const recovered = await invoke<RecoveredFile[]>("recover_all");
      setResults(recovered);
      showToast(`${recovered.length} arquivos recuperados com sucesso!`, "success");
    } catch (e) {
      showToast(`Erro na recuperação: ${e}`, "error");
    }
  };

  const toggleFileSelection = (id: string) => {
    setSelectedFiles(prev => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const selectAllFiles = () => {
    if (selectedFiles.size === results.length) {
      setSelectedFiles(new Set());
    } else {
      setSelectedFiles(new Set(results.map(f => f.id)));
    }
  };

  const showToast = (message: string, type: string) => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 4000);
  };

  const getFileIcon = (category: string) => {
    const icons: Record<string, string> = {
      images: "🖼️",
      videos: "🎬",
      audio: "🎵",
      documents: "📄",
      archives: "📦",
      emails: "📧",
      databases: "🗄️",
    };
    return icons[category] || "📁";
  };

  const renderHome = () => (
    <div className="content-area">
      <div className="section">
        <h2 className="section-title">
          💽 Selecione o Disco
        </h2>
        <div className="drives-grid">
          {drives.map((drive, idx) => (
            <div
              key={idx}
              className={`drive-card ${selectedDrive?.path === drive.path ? "selected" : ""}`}
              onClick={() => setSelectedDrive(drive)}
            >
              <div className="drive-icon">
                {drive.drive_type === "home" ? "🏠" : drive.drive_type === "usb" ? "🔌" : "💿"}
              </div>
              <div className="drive-name">{drive.name}</div>
              <div className="drive-path">{drive.path}</div>
              <div className="drive-info">
                <span>{drive.file_system}</span>
                <span>{drive.total_size > 0 ? formatBytes(drive.total_size) : "---"}</span>
              </div>
              {drive.total_size > 0 && (
                <div className="drive-progress">
                  <div
                    className="drive-progress-bar"
                    style={{ width: `${((drive.total_size - drive.free_space) / drive.total_size) * 100}%` }}
                  />
                </div>
              )}
            </div>
          ))}
        </div>
      </div>

      <div className="section">
        <h2 className="section-title">📂 Tipo de Arquivo</h2>
        <div className="category-grid">
          {categories.map(cat => (
            <div
              key={cat.id}
              className={`category-item ${
                cat.id === "all"
                  ? selectedCategories.length === categories.length - 1 ? "selected" : ""
                  : selectedCategories.includes(cat.id) ? "selected" : ""
              }`}
              onClick={() => toggleCategory(cat.id)}
            >
              <span className="cat-icon">{cat.icon}</span>
              <span>{cat.name}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="section">
        <h2 className="section-title">⚡ Tipo de Scan</h2>
        <div className="tabs">
          <button
            className={`tab ${scanType === "fast" ? "active" : ""}`}
            onClick={() => setScanType("fast")}
          >
            ⚡ Rápido
          </button>
          <button
            className={`tab ${scanType === "deep" ? "active" : ""}`}
            onClick={() => setScanType("deep")}
          >
            🔍 Profundo
          </button>
        </div>
        <p style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 8 }}>
          {scanType === "fast"
            ? "Escaneia rapidamente os arquivos conhecidos na estrutura do sistema."
            : "Escaneia setor por setor buscando assinaturas de arquivos. Mais lento, mas encontra mais arquivos."}
        </p>
      </div>

      <div className="section">
        <h2 className="section-title">📁 Pasta de Destino</h2>
        <div className="folder-input">
          <input
            type="text"
            className="form-input"
            placeholder="Selecione onde salvar os arquivos recuperados..."
            value={destination}
            readOnly
          />
          <button className="btn btn-secondary" onClick={selectDestination}>
            📂 Escolher
          </button>
        </div>
      </div>

      <div className="section">
        <button
          className="btn btn-secondary"
          onClick={() => setShowAdvanced(!showAdvanced)}
          style={{ marginBottom: 16 }}
        >
          {showAdvanced ? "▼" : "▶"} Opções Avançadas
        </button>

        {showAdvanced && (
          <div className="card">
            <div className="toggle-group">
              <div className="toggle-info">
                <span className="toggle-label">🔲 Filtrar Thumbnails</span>
                <span className="toggle-desc">Não salvar imagens de preview e thumbnails pequenas</span>
              </div>
              <div
                className={`toggle-switch ${filterThumbnails ? "active" : ""}`}
                onClick={() => setFilterThumbnails(!filterThumbnails)}
              />
            </div>

            <div className="toggle-group">
              <div className="toggle-info">
                <span className="toggle-label">🔧 Reparar Arquivos Danificados</span>
                <span className="toggle-desc">Tentar reparar arquivos corrompidos automaticamente</span>
              </div>
              <div
                className={`toggle-switch ${repairDamaged ? "active" : ""}`}
                onClick={() => setRepairDamaged(!repairDamaged)}
              />
            </div>

            <div className="toggle-group">
              <div className="toggle-info">
                <span className="toggle-label">📋 Pular Duplicatas</span>
                <span className="toggle-desc">Não recuperar arquivos duplicados</span>
              </div>
              <div
                className={`toggle-switch ${skipDuplicates ? "active" : ""}`}
                onClick={() => setSkipDuplicates(!skipDuplicates)}
              />
            </div>

            <div className="toggle-group">
              <div className="toggle-info">
                <span className="toggle-label">📂 Preservar Estrutura</span>
                <span className="toggle-desc">Manter a estrutura de pastas original</span>
              </div>
              <div
                className={`toggle-switch ${preserveStructure ? "active" : ""}`}
                onClick={() => setPreserveStructure(!preserveStructure)}
              />
            </div>

            <div className="toggle-group">
              <div className="toggle-info">
                <span className="toggle-label">🚀 Auto-Recuperar</span>
                <span className="toggle-desc">Recuperar automaticamente ao encontrar arquivos</span>
              </div>
              <div
                className={`toggle-switch ${autoRecover ? "active" : ""}`}
                onClick={() => setAutoRecover(!autoRecover)}
              />
            </div>

            <div className="toggle-group">
              <div className="toggle-info">
                <span className="toggle-label">📊 Recuperar Metadados</span>
                <span className="toggle-desc">Preservar data, hora e informações EXIF</span>
              </div>
              <div
                className={`toggle-switch ${recoverMetadata ? "active" : ""}`}
                onClick={() => setRecoverMetadata(!recoverMetadata)}
              />
            </div>

            <div className="scan-config" style={{ marginTop: 20 }}>
              <div className="form-group">
                <label className="form-label">Tamanho Mínimo</label>
                <input
                  type="number"
                  className="form-input"
                  value={minFileSize}
                  onChange={(e) => setMinFileSize(Number(e.target.value))}
                  placeholder="0"
                />
              </div>
              <div className="form-group">
                <label className="form-label">Tamanho Máximo (bytes)</label>
                <input
                  type="number"
                  className="form-input"
                  value={maxFileSize}
                  onChange={(e) => setMaxFileSize(Number(e.target.value))}
                  placeholder="5000000000"
                />
              </div>
            </div>
          </div>
        )}
      </div>

      <div style={{ textAlign: "center", marginTop: 32 }}>
        <button
          className="btn btn-primary btn-lg"
          onClick={startScan}
          disabled={!selectedDrive || !destination}
        >
          🔍 Iniciar Recuperação
        </button>
      </div>
    </div>
  );

  const renderScan = () => (
    <div className="content-area">
      <div className="scan-progress">
        <div className="progress-header">
          <span style={{ fontSize: 14, fontWeight: 600 }}>
            {isScanning ? "🔍 Escaneando..." : "✅ Scan Completo"}
          </span>
          <span style={{ fontSize: 13, color: "var(--text-muted)" }}>
            {Math.round(scanProgress)}%
          </span>
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
            <div className="stat-value">
              {scanType === "deep" ? "Profundo" : "Rápido"}
            </div>
            <div className="stat-label">Modo</div>
          </div>
        </div>
      </div>

      {isScanning && (
        <div style={{ textAlign: "center", marginTop: 32 }}>
          <div className="spinner" style={{ margin: "0 auto 16px" }} />
          <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
            Analisando {selectedDrive?.path}...
          </p>
          <button
            className="btn btn-danger"
            onClick={stopScan}
            style={{ marginTop: 20 }}
          >
            ⏹ Parar Scan
          </button>
        </div>
      )}

      {!isScanning && results.length > 0 && (
        <div style={{ textAlign: "center", marginTop: 32 }}>
          <button className="btn btn-primary btn-lg" onClick={() => setView("results")}>
            Ver Resultados →
          </button>
        </div>
      )}
    </div>
  );

  const renderResults = () => (
    <div className="content-area">
      <div className="section">
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
          <h2 className="section-title" style={{ margin: 0 }}>
            📋 Arquivos Encontrados
            <span className="badge">{results.length}</span>
          </h2>
          <div style={{ display: "flex", gap: 8 }}>
            <button className="btn btn-sm btn-secondary" onClick={selectAllFiles}>
              {selectedFiles.size === results.length ? "☐ Desmarcar" : "☑ Selecionar Tudo"}
            </button>
            <button
              className="btn btn-sm btn-primary"
              onClick={recoverSelected}
              disabled={selectedFiles.size === 0}
            >
              🚀 Recuperar ({selectedFiles.size})
            </button>
            <button className="btn btn-sm btn-primary" onClick={recoverAll}>
              🚀 Recuperar Tudo
            </button>
          </div>
        </div>

        {results.length === 0 ? (
          <div className="empty-state">
            <div className="icon">🔍</div>
            <h3>Nenhum arquivo encontrado</h3>
            <p>Tente um scan profundo para encontrar mais arquivos.</p>
            <button className="btn btn-primary" onClick={() => setView("home")} style={{ marginTop: 16 }}>
              Voltar ao Início
            </button>
          </div>
        ) : (
          <div className="results-grid">
            {results.map(file => (
              <div
                key={file.id}
                className={`file-card ${selectedFiles.has(file.id) ? "selected" : ""}`}
                onClick={() => toggleFileSelection(file.id)}
              >
                <div className="file-thumbnail">
                  {getFileIcon(file.category)}
                </div>
                <div className="file-info">
                  <div className="file-name" title={file.original_name}>
                    {file.original_name}
                  </div>
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
    </div>
  );

  const renderSettings = () => (
    <div className="content-area">
      <div className="section">
        <h2 className="section-title">⚙️ Configurações</h2>
        <div className="card">
          <div className="toggle-group">
            <div className="toggle-info">
              <span className="toggle-label">🎨 Tema Escuro</span>
              <span className="toggle-desc">Usar tema escuro na interface</span>
            </div>
            <div className="toggle-switch active" />
          </div>

          <div className="toggle-group">
            <div className="toggle-info">
              <span className="toggle-label">🔔 Notificações</span>
              <span className="toggle-desc">Receber notificações ao completar operações</span>
            </div>
            <div className="toggle-switch active" />
          </div>

          <div className="toggle-group">
            <div className="toggle-info">
              <span className="toggle-label">💾 Salvar Configurações</span>
              <span className="toggle-desc">Lembrar configurações entre sessões</span>
            </div>
            <div className="toggle-switch active" />
          </div>

          <div className="toggle-group">
            <div className="toggle-info">
              <span className="toggle-label">🔍 Preview de Arquivos</span>
              <span className="toggle-desc">Mostrar miniatura dos arquivos encontrados</span>
            </div>
            <div className="toggle-switch active" />
          </div>

          <div className="toggle-group">
            <div className="toggle-info">
              <span className="toggle-label">⚡ Modo Leve</span>
              <span className="toggle-desc">Reduzir uso de memória (ideal para PCs antigos)</span>
            </div>
            <div className="toggle-switch active" />
          </div>

          <div className="toggle-group">
            <div className="toggle-info">
              <span className="toggle-label">🔄 Verificar Integridade</span>
              <span className="toggle-desc">Verificar hash dos arquivos após recuperação</span>
            </div>
            <div className="toggle-switch active" />
          </div>
        </div>
      </div>

      <div className="section">
        <h2 className="section-title">ℹ️ Sobre</h2>
        <div className="card">
          <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
            <img src="/logo.png" alt="Restora" style={{ width: 48, height: 48, borderRadius: 10 }} />
            <div>
              <h3 style={{ fontSize: 16, fontWeight: 700 }}>Restora</h3>
              <p style={{ fontSize: 12, color: "var(--text-muted)" }}>Versão 1.0.0</p>
              <p style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 4 }}>
                Ferramenta de recuperação de arquivos leve e minimalista.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );

  return (
    <div className="app-container">
      {/* Sidebar */}
      <div className="sidebar">
        <div className="sidebar-logo">
          <img src="/logo.png" alt="Restora" />
          <div>
            <h1>Restora</h1>
            <span>File Recovery v1.0</span>
          </div>
        </div>

        <div className="sidebar-nav">
          <button
            className={`nav-item ${view === "home" ? "active" : ""}`}
            onClick={() => setView("home")}
          >
            <span className="icon">🏠</span>
            Início
          </button>
          <button
            className={`nav-item ${view === "scan" ? "active" : ""}`}
            onClick={() => setView("scan")}
          >
            <span className="icon">🔍</span>
            Escanear
          </button>
          <button
            className={`nav-item ${view === "results" ? "active" : ""}`}
            onClick={() => setView("results")}
          >
            <span className="icon">📋</span>
            Resultados
            {results.length > 0 && (
              <span className="badge" style={{ marginLeft: "auto" }}>{results.length}</span>
            )}
          </button>
          <button
            className={`nav-item ${view === "settings" ? "active" : ""}`}
            onClick={() => setView("settings")}
          >
            <span className="icon">⚙️</span>
            Configurações
          </button>
        </div>

        <div className="sidebar-footer">
          <p>Restora © 2026 • Leve & Rápido</p>
          <p style={{ marginTop: 4 }}>Funciona com 2GB RAM ✨</p>
        </div>
      </div>

      {/* Main Content */}
      <div className="main-content">
        <div className="header">
          <span className="header-title">
            {view === "home" && "🏠 Recuperação de Arquivos"}
            {view === "scan" && "🔍 Escaneamento"}
            {view === "results" && "📋 Resultados"}
            {view === "settings" && "⚙️ Configurações"}
          </span>
          <div className="header-actions">
            {view === "home" && selectedDrive && (
              <span style={{ fontSize: 12, color: "var(--text-muted)" }}>
                Disco: {selectedDrive.name}
              </span>
            )}
          </div>
        </div>

        {view === "home" && renderHome()}
        {view === "scan" && renderScan()}
        {view === "results" && renderResults()}
        {view === "settings" && renderSettings()}
      </div>

      {/* Toast */}
      {toast && (
        <div className={`toast ${toast.type}`}>
          <span>{toast.type === "success" ? "✅" : "❌"}</span>
          <span style={{ fontSize: 13 }}>{toast.message}</span>
        </div>
      )}
    </div>
  );
}

export default App;
