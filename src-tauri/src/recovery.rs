use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FileSignature {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub category: &'static str,
    pub magic: &'static [u8],
    pub offset: usize,
    pub max_size: u64,
    pub footer: Option<&'static [u8]>,
}

pub fn get_signatures() -> Vec<FileSignature> {
    vec![
        FileSignature { name: "JPEG", extensions: &["jpg", "jpeg"], category: "images", magic: &[0xFF, 0xD8, 0xFF], offset: 0, max_size: 20_000_000, footer: Some(&[0xFF, 0xD9]) },
        FileSignature { name: "PNG", extensions: &["png"], category: "images", magic: &[0x89, 0x50, 0x4E, 0x47], offset: 0, max_size: 50_000_000, footer: Some(&[0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82]) },
        FileSignature { name: "GIF", extensions: &["gif"], category: "images", magic: b"GIF8", offset: 0, max_size: 20_000_000, footer: None },
        FileSignature { name: "BMP", extensions: &["bmp"], category: "images", magic: b"BM", offset: 0, max_size: 100_000_000, footer: None },
        FileSignature { name: "WEBP", extensions: &["webp"], category: "images", magic: b"RIFF", offset: 0, max_size: 20_000_000, footer: None },
        FileSignature { name: "TIFF", extensions: &["tiff", "tif"], category: "images", magic: &[0x49, 0x49, 0x2A, 0x00], offset: 0, max_size: 100_000_000, footer: None },
        FileSignature { name: "CR2", extensions: &["cr2"], category: "images", magic: &[0x49, 0x49, 0x2A, 0x00, 0x10, 0x00, 0x00, 0x00, 0x43, 0x52], offset: 0, max_size: 100_000_000, footer: None },
        FileSignature { name: "HEIC", extensions: &["heic", "heif"], category: "images", magic: &[0x00, 0x00, 0x00], offset: 0, max_size: 50_000_000, footer: None },
        FileSignature { name: "MP4", extensions: &["mp4"], category: "videos", magic: &[0x00, 0x00, 0x00], offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "AVI", extensions: &["avi"], category: "videos", magic: b"RIFF", offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "MKV", extensions: &["mkv"], category: "videos", magic: &[0x1A, 0x45, 0xDF, 0xA3], offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "MOV", extensions: &["mov"], category: "videos", magic: &[0x00, 0x00, 0x00, 0x14, 0x66, 0x74, 0x79, 0x70], offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "FLV", extensions: &["flv"], category: "videos", magic: b"FLV", offset: 0, max_size: 2_000_000_000, footer: None },
        FileSignature { name: "WMV", extensions: &["wmv"], category: "videos", magic: &[0x30, 0x26, 0xB2, 0x75], offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "MP3", extensions: &["mp3"], category: "audio", magic: &[0xFF, 0xFB], offset: 0, max_size: 50_000_000, footer: None },
        FileSignature { name: "MP3v2", extensions: &["mp3"], category: "audio", magic: b"ID3", offset: 0, max_size: 50_000_000, footer: None },
        FileSignature { name: "WAV", extensions: &["wav"], category: "audio", magic: b"RIFF", offset: 0, max_size: 2_000_000_000, footer: None },
        FileSignature { name: "FLAC", extensions: &["flac"], category: "audio", magic: b"fLaC", offset: 0, max_size: 500_000_000, footer: None },
        FileSignature { name: "OGG", extensions: &["ogg"], category: "audio", magic: b"OggS", offset: 0, max_size: 200_000_000, footer: None },
        FileSignature { name: "AAC", extensions: &["aac", "m4a"], category: "audio", magic: &[0xFF, 0xF1], offset: 0, max_size: 50_000_000, footer: None },
        FileSignature { name: "PDF", extensions: &["pdf"], category: "documents", magic: b"%PDF", offset: 0, max_size: 500_000_000, footer: Some(b"%%EOF") },
        FileSignature { name: "DOCX", extensions: &["docx"], category: "documents", magic: &[0x50, 0x4B, 0x03, 0x04], offset: 0, max_size: 200_000_000, footer: None },
        FileSignature { name: "DOC", extensions: &["doc"], category: "documents", magic: &[0xD0, 0xCF, 0x11, 0xE0], offset: 0, max_size: 200_000_000, footer: None },
        FileSignature { name: "XLSX", extensions: &["xlsx"], category: "documents", magic: &[0x50, 0x4B, 0x03, 0x04], offset: 0, max_size: 200_000_000, footer: None },
        FileSignature { name: "PPTX", extensions: &["pptx"], category: "documents", magic: &[0x50, 0x4B, 0x03, 0x04], offset: 0, max_size: 200_000_000, footer: None },
        FileSignature { name: "RTF", extensions: &["rtf"], category: "documents", magic: b"{\\rtf", offset: 0, max_size: 100_000_000, footer: None },
        FileSignature { name: "ZIP", extensions: &["zip"], category: "archives", magic: &[0x50, 0x4B, 0x03, 0x04], offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "RAR", extensions: &["rar"], category: "archives", magic: b"Rar!", offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "7Z", extensions: &["7z"], category: "archives", magic: &[0x37, 0x7A, 0xBC, 0xAF], offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "TAR", extensions: &["tar"], category: "archives", magic: b"ustar", offset: 257, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "GZ", extensions: &["gz"], category: "archives", magic: &[0x1F, 0x8B], offset: 0, max_size: 4_000_000_000, footer: None },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub name: String, pub path: String, pub total_size: u64,
    pub free_space: u64, pub file_system: String, pub drive_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveredFile {
    pub id: String, pub original_name: String, pub file_type: String,
    pub category: String, pub size: u64, pub path: String,
    pub recovered_path: String, pub status: String,
    pub is_damaged: bool, pub is_thumbnail: bool,
    pub confidence: f32, pub found_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub drive_path: String, pub scan_type: String, pub categories: Vec<String>,
    pub destination: String, pub filter_thumbnails: bool, pub repair_damaged: bool,
    pub max_file_size: u64, pub min_file_size: u64, pub skip_duplicates: bool,
    pub preserve_structure: bool, pub auto_recover: bool,
    pub recover_metadata: bool, pub verbose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub phase: String, pub current_file: String, pub files_found: u64,
    pub files_recovered: u64, pub total_size_recovered: u64,
    pub progress_percent: f32, pub elapsed_seconds: u64,
    pub estimated_remaining: u64, pub scan_speed_mbps: f32, pub status: String,
}

pub fn get_drives() -> Vec<DriveInfo> {
    let mut drives = Vec::new();
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("lsblk")
            .args(["-o", "NAME,SIZE,FSTYPE,MOUNTPOINT,TYPE", "-J", "-b"]).output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(bds) = parsed["blockdevices"].as_array() {
                    for device in bds {
                        if let Some(children) = device["children"].as_array() {
                            for child in children {
                                let mp = child["mountpoint"].as_str().unwrap_or("").to_string();
                                if !mp.is_empty() && mp != "[SWAP]" {
                                    drives.push(DriveInfo {
                                        name: child["name"].as_str().unwrap_or("").to_string(),
                                        path: mp.clone(), total_size: child["size"].as_u64().unwrap_or(0),
                                        free_space: get_free_space(&mp),
                                        file_system: child["fstype"].as_str().unwrap_or("unknown").to_string(),
                                        drive_type: "partition".to_string(),
                                    });
                                }
                            }
                        }
                        let mp = device["mountpoint"].as_str().unwrap_or("").to_string();
                        if !mp.is_empty() && mp != "[SWAP]" {
                            drives.push(DriveInfo {
                                name: device["name"].as_str().unwrap_or("").to_string(),
                                path: mp.clone(), total_size: device["size"].as_u64().unwrap_or(0),
                                free_space: get_free_space(&mp),
                                file_system: device["fstype"].as_str().unwrap_or("unknown").to_string(),
                                drive_type: "disk".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        for letter in b'A'..=b'Z' {
            let path = format!("{}:\\", letter as char);
            if Path::new(&path).exists() {
                drives.push(DriveInfo {
                    name: format!("Drive {}", letter as char), path: path.clone(),
                    total_size: get_total_space(&path), free_space: get_free_space(&path),
                    file_system: "NTFS".to_string(), drive_type: "local".to_string(),
                });
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(entries) = fs::read_dir("/Volumes") {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let mount = p.to_string_lossy().to_string();
                    drives.push(DriveInfo {
                        name: p.file_name().unwrap().to_string_lossy().to_string(),
                        path: mount.clone(), total_size: get_total_space(&mount),
                        free_space: get_free_space(&mount),
                        file_system: "APFS".to_string(), drive_type: "volume".to_string(),
                    });
                }
            }
        }
    }
    if drives.is_empty() {
        drives.push(DriveInfo {
            name: "Home".to_string(),
            path: std::env::var("HOME").unwrap_or_else(|_| "/home".to_string()),
            total_size: 0, free_space: 0,
            file_system: "unknown".to_string(), drive_type: "home".to_string(),
        });
    }
    drives
}

fn get_free_space(path: &str) -> u64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("df").args(["-B1", path]).output() {
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout_str.lines().collect();
            if lines.len() > 1 {
                let parts: Vec<&str> = lines[1].split_whitespace().collect();
                if parts.len() > 3 { return parts[3].parse().unwrap_or(0); }
            }
        }
    }
    0
}

fn get_total_space(_path: &str) -> u64 { 0 }

fn is_active_file_location(path: &str) -> bool {
    let p = path.to_lowercase();
    [
        "/documents/", "/downloads/", "/pictures/", "/videos/", "/music/",
        "/desktop/", "/public/", "/templates/",
        "\\documents\\", "\\downloads\\", "\\pictures\\", "\\videos\\", "\\music\\",
        "\\desktop\\", "\\public\\", "\\templates\\",
        "/appdata/roaming/", "\\appdata\\roaming\\",
        "/usr/share/", "/etc/", "/var/", "/proc/", "/sys/",
    ].iter().any(|pat| p.contains(pat))
}

fn is_thumbnail_file(path: &str, size: u64) -> bool {
    let p = path.to_lowercase();
    if ["thumb", "thumbnail", ".thumb", "tn_", "_tn", "preview", "icon", "cache", ".tmp", "~$"]
        .iter().any(|pat| p.contains(pat)) { return true; }
    if size < 5_000 && (p.ends_with(".jpg") || p.ends_with(".png")) { return true; }
    [".thumbnails", "thumbs", "thumbnails", "@__thumb"]
        .iter().any(|dir| p.contains(dir))
}

const HEADER_SIZE: usize = 512;

fn check_header_match(header: &[u8], sig: &FileSignature) -> bool {
    if sig.magic.is_empty() { return true; }
    let end = sig.offset + sig.magic.len();
    if end > header.len() { return false; }
    header[sig.offset..end] == *sig.magic
}

fn check_footer_fast(path: &Path, sig: &FileSignature) -> bool {
    if let Some(footer) = sig.footer {
        if let Ok(file_size) = fs::metadata(path).map(|m| m.len()) {
            if file_size >= footer.len() as u64 {
                if let Ok(mut file) = fs::File::open(path) {
                    if file.seek(SeekFrom::End(-(footer.len() as i64))).is_ok() {
                        let mut end_bytes = vec![0u8; footer.len()];
                        if file.read_exact(&mut end_bytes).is_ok() {
                            if end_bytes != footer { return true; }
                        }
                    }
                }
            }
        }
    }
    false
}

fn get_recovery_dirs(base: &str) -> Vec<String> {
    let mut dirs = Vec::new();
    for dir in [".Trash", ".Trash-1000", ".local/share/Trash/files", "$Recycle.Bin", "RECYCLER",
                ".cache/thumbnails", ".thumbnails", "System Volume Information",
                ".Spotlight-V100", ".TemporaryItems"] {
        let full = format!("{}/{}", base, dir);
        if Path::new(&full).exists() { dirs.push(full); }
    }
    #[cfg(target_os = "windows")]
    {
        let rp = format!("{}\\$Recycle.Bin", base);
        if Path::new(&rp).exists() {
            if let Ok(entries) = fs::read_dir(&rp) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        dirs.push(entry.path().to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    dirs
}

// ===== OPTIMIZED: Sequential scan with yield points to avoid freezing =====
pub fn fast_scan(config: &ScanConfig) -> Vec<RecoveredFile> {
    let signatures = get_signatures();
    let search_dirs = get_recovery_dirs(&config.drive_path);
    if search_dirs.is_empty() { return deep_scan_optimized(config); }

    let mut files = Vec::new();
    let mut seen = HashSet::new();

    for dir in &search_dirs {
        if !Path::new(dir).exists() { continue; }
        let entries: Vec<_> = WalkDir::new(dir).max_depth(4).follow_links(false)
            .into_iter().filter_map(|e| e.ok()).collect();

        for entry in entries {
            if !entry.file_type().is_file() { continue; }
            let path = entry.path().to_path_buf();
            let path_str = path.to_string_lossy().to_string();
            let metadata = match entry.metadata() { Ok(m) => m, Err(_) => continue };
            let size = metadata.len();
            if size < config.min_file_size || size > config.max_file_size { continue; }

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if !config.categories.is_empty() && !signatures.iter().any(|s|
                s.extensions.iter().any(|&se| se == ext) && config.categories.contains(&s.category.to_string()))
            { continue; }

            if config.filter_thumbnails && is_thumbnail_file(&path_str, size) { continue; }

            if config.skip_duplicates {
                let key = format!("{}:{}", path_str, size);
                if !seen.insert(key) { continue; }
            }

            if let Some(file) = process_file(&path, &path_str, size, &ext, &signatures, config) {
                files.push(file);
            }

            // Yield to avoid freezing UI
            if files.len() % 50 == 0 {
                thread::sleep(Duration::from_millis(5));
            }
        }
    }
    files
}

pub fn deep_scan(config: &ScanConfig) -> Vec<RecoveredFile> {
    deep_scan_optimized(config)
}

fn deep_scan_optimized(config: &ScanConfig) -> Vec<RecoveredFile> {
    let signatures = get_signatures();
    let mut files = Vec::new();
    let mut seen = HashSet::new();
    let max_depth = if config.scan_type == "deep" { 8 } else { 4 };

    let entries: Vec<_> = WalkDir::new(&config.drive_path).max_depth(max_depth).follow_links(false)
        .into_iter().filter_map(|e| e.ok()).collect();

    for entry in entries {
        if !entry.file_type().is_file() { continue; }
        let path = entry.path().to_path_buf();
        let path_str = path.to_string_lossy().to_string();

        if is_active_file_location(&path_str) { continue; }

        let metadata = match entry.metadata() { Ok(m) => m, Err(_) => continue };
        let size = metadata.len();
        if size < config.min_file_size || size > config.max_file_size { continue; }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        if !config.categories.is_empty() && !signatures.iter().any(|s|
            s.extensions.iter().any(|&se| se == ext) && config.categories.contains(&s.category.to_string()))
        { continue; }

        if config.filter_thumbnails && is_thumbnail_file(&path_str, size) { continue; }

        if config.skip_duplicates {
            let key = format!("{}:{}", path_str, size);
            if !seen.insert(key) { continue; }
        }

        if let Some(file) = process_file(&path, &path_str, size, &ext, &signatures, config) {
            files.push(file);
        }

        // Yield every 100 files to keep UI responsive
        if files.len() % 100 == 0 {
            thread::sleep(Duration::from_millis(10));
        }
    }
    files
}

fn process_file(
    path: &Path, path_str: &str, size: u64, ext: &str,
    signatures: &[FileSignature], config: &ScanConfig,
) -> Option<RecoveredFile> {
    let mut header = [0u8; HEADER_SIZE];
    let mut file = fs::File::open(path).ok()?;
    let bytes_read = file.read(&mut header).ok()?;
    if bytes_read < 4 { return None; }

    for sig in signatures {
        if check_header_match(&header[..bytes_read], sig) {
            let is_damaged = check_footer_fast(path, sig);
            let file_type = sig.extensions.first().unwrap_or(&"unknown").to_string();

            return Some(RecoveredFile {
                id: Uuid::new_v4().to_string(),
                original_name: path.file_name()?.to_string_lossy().to_string(),
                file_type, category: sig.category.to_string(),
                size, path: path_str.to_string(), recovered_path: String::new(),
                status: if is_damaged { "damaged".to_string() } else { "found".to_string() },
                is_damaged, is_thumbnail: false,
                confidence: if is_damaged { 0.6 } else { 0.95 },
                found_at: chrono::Utc::now().to_rfc3339(),
            });
        }
    }
    None
}

pub fn recover_files(files_to_recover: &[RecoveredFile], config: &ScanConfig) -> Vec<RecoveredFile> {
    let dest_base = PathBuf::from(&config.destination);
    fs::create_dir_all(&dest_base).ok();

    files_to_recover.iter().filter_map(|file| {
        let src = PathBuf::from(&file.path);
        if !src.exists() { return None; }

        let category_dir = dest_base.join(&file.category);
        fs::create_dir_all(&category_dir).ok()?;

        let base_name = Path::new(&file.original_name)
            .file_stem().and_then(|s| s.to_str()).unwrap_or("recovered");
        let recovered_name = format!("{}_{}.{}", base_name, &file.id[..8], file.file_type);
        let dest_path = category_dir.join(&recovered_name);

        fs::copy(&src, &dest_path).ok()?;

        let mut rec = file.clone();
        rec.recovered_path = dest_path.to_string_lossy().to_string();
        rec.status = if file.is_damaged && config.repair_damaged {
            repair_file(&dest_path, &rec); "repaired".to_string()
        } else if file.is_damaged {
            "recovered_damaged".to_string()
        } else { "recovered".to_string() };

        Some(rec)
    }).collect()
}

fn repair_file(path: &PathBuf, file: &RecoveredFile) {
    match file.file_type.to_lowercase().as_str() {
        "jpg" | "jpeg" => repair_jpeg(path),
        "png" => repair_png(path),
        "mp3" => repair_mp3(path),
        "pdf" => repair_pdf(path),
        _ => {}
    }
}

fn repair_jpeg(path: &PathBuf) {
    let mut data = match fs::read(path) { Ok(d) => d, Err(_) => return };
    let mut modified = false;
    if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
        let mut new_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        new_data.extend_from_slice(&data); data = new_data; modified = true;
    }
    let len = data.len();
    if len < 2 || data[len-2] != 0xFF || data[len-1] != 0xD9 {
        data.extend_from_slice(&[0xFF, 0xD9]); modified = true;
    }
    if modified { let _ = fs::write(path, &data); }
}

fn repair_png(path: &PathBuf) {
    let mut data = match fs::read(path) { Ok(d) => d, Err(_) => return };
    let png_sig: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if data.len() < 8 || data[..8] != png_sig {
        let mut new_data = png_sig.to_vec();
        new_data.extend_from_slice(&data); data = new_data;
    }
    let iend: [u8; 12] = [0x00,0x00,0x00,0x00,0x49,0x45,0x4E,0x44,0xAE,0x42,0x60,0x82];
    let len = data.len();
    if len < 12 || data[len-12..] != iend { data.extend_from_slice(&iend); }
    let _ = fs::write(path, &data);
}

fn repair_mp3(path: &PathBuf) {
    let data = match fs::read(path) { Ok(d) => d, Err(_) => return };
    if data.len() < 3 { return; }
    if data[0] == 0xFF && (data[1] & 0xE0) == 0xE0 { return; }
    if &data[0..3] == b"ID3" { return; }
    if let Some(pos) = data[..std::cmp::min(4096, data.len())]
        .windows(2).position(|w| w[0] == 0xFF && (w[1] & 0xE0) == 0xE0)
    { if pos > 0 { let _ = fs::write(path, &data[pos..]); } }
}

fn repair_pdf(path: &PathBuf) {
    let mut data = match fs::read(path) { Ok(d) => d, Err(_) => return };
    if !data.starts_with(b"%PDF") {
        let mut new_data = b"%PDF-1.4\n".to_vec();
        new_data.extend_from_slice(&data); data = new_data;
    }
    if !data.windows(5).any(|w| w == b"%%EOF") { data.extend_from_slice(b"\n%%EOF\n"); }
    let _ = fs::write(path, &data);
}
