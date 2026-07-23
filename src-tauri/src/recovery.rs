use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;

/// File signatures (magic bytes) for identification
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

/// Supported file signatures database
pub fn get_signatures() -> Vec<FileSignature> {
    vec![
        // Images
        FileSignature { name: "JPEG", extensions: &["jpg", "jpeg"], category: "images", magic: &[0xFF, 0xD8, 0xFF], offset: 0, max_size: 20_000_000, footer: Some(&[0xFF, 0xD9]) },
        FileSignature { name: "PNG", extensions: &["png"], category: "images", magic: &[0x89, 0x50, 0x4E, 0x47], offset: 0, max_size: 50_000_000, footer: Some(&[0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82]) },
        FileSignature { name: "GIF", extensions: &["gif"], category: "images", magic: b"GIF8", offset: 0, max_size: 20_000_000, footer: None },
        FileSignature { name: "BMP", extensions: &["bmp"], category: "images", magic: b"BM", offset: 0, max_size: 100_000_000, footer: None },
        FileSignature { name: "WEBP", extensions: &["webp"], category: "images", magic: b"RIFF", offset: 0, max_size: 20_000_000, footer: None },
        FileSignature { name: "TIFF", extensions: &["tiff", "tif"], category: "images", magic: &[0x49, 0x49, 0x2A, 0x00], offset: 0, max_size: 100_000_000, footer: None },
        FileSignature { name: "RAW (CR2)", extensions: &["cr2"], category: "images", magic: &[0x49, 0x49, 0x2A, 0x00, 0x10, 0x00, 0x00, 0x00, 0x43, 0x52], offset: 0, max_size: 100_000_000, footer: None },
        FileSignature { name: "HEIF/HEIC", extensions: &["heic", "heif"], category: "images", magic: &[0x00, 0x00, 0x00], offset: 0, max_size: 50_000_000, footer: None },
        // Videos
        FileSignature { name: "MP4", extensions: &["mp4"], category: "videos", magic: &[0x00, 0x00, 0x00], offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "AVI", extensions: &["avi"], category: "videos", magic: b"RIFF", offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "MKV", extensions: &["mkv"], category: "videos", magic: &[0x1A, 0x45, 0xDF, 0xA3], offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "MOV", extensions: &["mov"], category: "videos", magic: &[0x00, 0x00, 0x00, 0x14, 0x66, 0x74, 0x79, 0x70], offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "FLV", extensions: &["flv"], category: "videos", magic: b"FLV", offset: 0, max_size: 2_000_000_000, footer: None },
        FileSignature { name: "WMV", extensions: &["wmv"], category: "videos", magic: &[0x30, 0x26, 0xB2, 0x75], offset: 0, max_size: 4_000_000_000, footer: None },
        // Audio
        FileSignature { name: "MP3", extensions: &["mp3"], category: "audio", magic: &[0xFF, 0xFB], offset: 0, max_size: 50_000_000, footer: None },
        FileSignature { name: "MP3 (ID3)", extensions: &["mp3"], category: "audio", magic: b"ID3", offset: 0, max_size: 50_000_000, footer: None },
        FileSignature { name: "WAV", extensions: &["wav"], category: "audio", magic: b"RIFF", offset: 0, max_size: 2_000_000_000, footer: None },
        FileSignature { name: "FLAC", extensions: &["flac"], category: "audio", magic: b"fLaC", offset: 0, max_size: 500_000_000, footer: None },
        FileSignature { name: "OGG", extensions: &["ogg"], category: "audio", magic: b"OggS", offset: 0, max_size: 200_000_000, footer: None },
        FileSignature { name: "AAC", extensions: &["aac", "m4a"], category: "audio", magic: &[0xFF, 0xF1], offset: 0, max_size: 50_000_000, footer: None },
        // Documents
        FileSignature { name: "PDF", extensions: &["pdf"], category: "documents", magic: b"%PDF", offset: 0, max_size: 500_000_000, footer: Some(b"%%EOF") },
        FileSignature { name: "DOCX", extensions: &["docx"], category: "documents", magic: &[0x50, 0x4B, 0x03, 0x04], offset: 0, max_size: 200_000_000, footer: None },
        FileSignature { name: "DOC", extensions: &["doc"], category: "documents", magic: &[0xD0, 0xCF, 0x11, 0xE0], offset: 0, max_size: 200_000_000, footer: None },
        FileSignature { name: "XLSX", extensions: &["xlsx"], category: "documents", magic: &[0x50, 0x4B, 0x03, 0x04], offset: 0, max_size: 200_000_000, footer: None },
        FileSignature { name: "PPTX", extensions: &["pptx"], category: "documents", magic: &[0x50, 0x4B, 0x03, 0x04], offset: 0, max_size: 200_000_000, footer: None },
        FileSignature { name: "RTF", extensions: &["rtf"], category: "documents", magic: b"{\\rtf", offset: 0, max_size: 100_000_000, footer: None },
        FileSignature { name: "TXT", extensions: &["txt"], category: "documents", magic: b"", offset: 0, max_size: 10_000_000, footer: None },
        // Archives
        FileSignature { name: "ZIP", extensions: &["zip"], category: "archives", magic: &[0x50, 0x4B, 0x03, 0x04], offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "RAR", extensions: &["rar"], category: "archives", magic: b"Rar!", offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "7Z", extensions: &["7z"], category: "archives", magic: &[0x37, 0x7A, 0xBC, 0xAF], offset: 0, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "TAR", extensions: &["tar"], category: "archives", magic: b"ustar", offset: 257, max_size: 4_000_000_000, footer: None },
        FileSignature { name: "GZ", extensions: &["gz"], category: "archives", magic: &[0x1F, 0x8B], offset: 0, max_size: 4_000_000_000, footer: None },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub name: String,
    pub path: String,
    pub total_size: u64,
    pub free_space: u64,
    pub file_system: String,
    pub drive_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveredFile {
    pub id: String,
    pub original_name: String,
    pub file_type: String,
    pub category: String,
    pub size: u64,
    pub path: String,
    pub recovered_path: String,
    pub status: String,
    pub is_damaged: bool,
    pub is_thumbnail: bool,
    pub confidence: f32,
    pub found_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub drive_path: String,
    pub scan_type: String, // "fast" or "deep"
    pub categories: Vec<String>,
    pub destination: String,
    pub filter_thumbnails: bool,
    pub repair_damaged: bool,
    pub max_file_size: u64,
    pub min_file_size: u64,
    pub skip_duplicates: bool,
    pub preserve_structure: bool,
    auto_recover: bool,
    pub recover_metadata: bool,
    pub verbose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub phase: String,
    pub current_file: String,
    pub files_found: u64,
    pub files_recovered: u64,
    pub total_size_recovered: u64,
    pub progress_percent: f32,
    pub elapsed_seconds: u64,
    pub estimated_remaining: u64,
    pub scan_speed_mbps: f32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub success: bool,
    pub files_found: Vec<RecoveredFile>,
    pub message: String,
    pub total_recovered: u64,
    pub total_size: u64,
}

pub fn get_drives() -> Vec<DriveInfo> {
    let mut drives = Vec::new();

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("lsblk")
            .args(["-o", "NAME,SIZE,FSTYPE,MOUNTPOINT,TYPE", "-J", "-b"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(blockdevices) = parsed["blockdevices"].as_array() {
                    for device in blockdevices {
                        if let Some(children) = device["children"].as_array() {
                            for child in children {
                                let name = child["name"].as_str().unwrap_or("").to_string();
                                let mountpoint = child["mountpoint"].as_str().unwrap_or("").to_string();
                                let size = child["size"].as_u64().unwrap_or(0);
                                let fstype = child["fstype"].as_str().unwrap_or("unknown").to_string();

                                if !mountpoint.is_empty() && mountpoint != "[SWAP]" {
                                    drives.push(DriveInfo {
                                        name: name.clone(),
                                        path: mountpoint.clone(),
                                        total_size: size,
                                        free_space: get_free_space(&mountpoint),
                                        file_system: fstype,
                                        drive_type: "partition".to_string(),
                                    });
                                }
                            }
                        }
                        let mountpoint = device["mountpoint"].as_str().unwrap_or("").to_string();
                        if !mountpoint.is_empty() && mountpoint != "[SWAP]" {
                            let name = device["name"].as_str().unwrap_or("").to_string();
                            let size = device["size"].as_u64().unwrap_or(0);
                            let fstype = device["fstype"].as_str().unwrap_or("unknown").to_string();
                            drives.push(DriveInfo {
                                name,
                                path: mountpoint.clone(),
                                total_size: size,
                                free_space: get_free_space(&mountpoint),
                                file_system: fstype,
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
                let size = get_total_space(&path);
                drives.push(DriveInfo {
                    name: format!("Drive {}", letter as char),
                    path,
                    total_size: size,
                    free_space: get_free_space(&format!("{}:\\", letter as char)),
                    file_system: "NTFS".to_string(),
                    drive_type: "local".to_string(),
                });
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let volumes_path = "/Volumes";
        if let Ok(entries) = fs::read_dir(volumes_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let mount = path.to_string_lossy().to_string();
                    drives.push(DriveInfo {
                        name: path.file_name().unwrap().to_string_lossy().to_string(),
                        path: mount.clone(),
                        total_size: get_total_space(&mount),
                        free_space: get_free_space(&mount),
                        file_system: "APFS".to_string(),
                        drive_type: "volume".to_string(),
                    });
                }
            }
        }
    }

    if drives.is_empty() {
        drives.push(DriveInfo {
            name: "Home".to_string(),
            path: std::env::var("HOME").unwrap_or_else(|_| "/home".to_string()),
            total_size: 0,
            free_space: 0,
            file_system: "unknown".to_string(),
            drive_type: "home".to_string(),
        });
    }

    drives
}

fn get_free_space(path: &str) -> u64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("df")
            .args(["-B1", path])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();
            if lines.len() > 1 {
                let parts: Vec<&str> = lines[1].split_whitespace().collect();
                if parts.len() > 3 {
                    return parts[3].parse().unwrap_or(0);
                }
            }
        }
    }
    0
}

fn get_total_space(_path: &str) -> u64 {
    0
}

/// Fast scan: walks the filesystem looking for deleted file traces
pub fn fast_scan(config: &ScanConfig) -> Vec<RecoveredFile> {
    let mut files = Vec::new();
    let signatures = get_signatures();
    let scan_path = &config.drive_path;

    // For fast scan, look in common locations for recoverable files
    let search_dirs = get_search_directories(scan_path);

    for dir in search_dirs {
        if !Path::new(&dir).exists() {
            continue;
        }

        for entry in WalkDir::new(&dir).max_depth(5).follow_links(false) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if entry.file_type().is_file() {
                let path = entry.path().to_string_lossy().to_string();
                let ext = entry.path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                let matching_sig = signatures.iter().find(|s| {
                    s.extensions.iter().any(|&se| se.to_lowercase() == ext)
                });

                if let Some(sig) = matching_sig {
                    if !config.categories.is_empty() && !config.categories.contains(&sig.category.to_string()) {
                        continue;
                    }

                    let metadata = entry.metadata().ok();
                    let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

                    if size < config.min_file_size || size > config.max_file_size {
                        continue;
                    }

                    let is_thumbnail = is_thumbnail_file(&path, size);

                    if config.filter_thumbnails && is_thumbnail {
                        continue;
                    }

                    files.push(RecoveredFile {
                        id: Uuid::new_v4().to_string(),
                        original_name: entry.file_name().to_string_lossy().to_string(),
                        file_type: ext.clone(),
                        category: sig.category.to_string(),
                        size,
                        path: path.clone(),
                        recovered_path: String::new(),
                        status: "found".to_string(),
                        is_damaged: false,
                        is_thumbnail,
                        confidence: 0.95,
                        found_at: chrono::Utc::now().to_rfc3339(),
                    });
                }
            }
        }
    }

    files
}

/// Deep scan: reads raw data looking for file signatures
pub fn deep_scan(config: &ScanConfig) -> Vec<RecoveredFile> {
    let mut files = Vec::new();
    let signatures = get_signatures();
    let scan_path = &config.drive_path;

    // For deep scan, we search through the directory tree reading file headers
    let search_dirs = get_search_directories(scan_path);

    for dir in search_dirs {
        if !Path::new(&dir).exists() {
            continue;
        }

        for entry in WalkDir::new(&dir).max_depth(10).follow_links(false) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path().to_string_lossy().to_string();
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size = metadata.len();
            if size < config.min_file_size || size > config.max_file_size {
                continue;
            }

            // Read the first bytes to check signature
            if let Ok(mut file) = fs::File::open(entry.path()) {
                let mut header = [0u8; 512];
                let bytes_read = file.read(&mut header).unwrap_or(0);

                if bytes_read < 4 {
                    continue;
                }

                for sig in &signatures {
                    if sig.magic.is_empty() {
                        continue;
                    }

                    if !config.categories.is_empty() && !config.categories.contains(&sig.category.to_string()) {
                        continue;
                    }

                    let magic_match = if sig.offset + sig.magic.len() <= bytes_read {
                        header[sig.offset..sig.offset + sig.magic.len()] == *sig.magic
                    } else {
                        false
                    };

                    if magic_match {
                        let is_thumbnail = is_thumbnail_file(&path, size);

                        if config.filter_thumbnails && is_thumbnail {
                            continue;
                        }

                        // Check if file is potentially damaged
                        let is_damaged = check_file_integrity(&mut file, sig, size);

                        let ext = sig.extensions.first().unwrap_or(&"unknown").to_string();
                        let name = entry.file_name().to_string_lossy().to_string();

                        files.push(RecoveredFile {
                            id: Uuid::new_v4().to_string(),
                            original_name: name,
                            file_type: ext,
                            category: sig.category.to_string(),
                            size,
                            path: path.clone(),
                            recovered_path: String::new(),
                            status: if is_damaged { "damaged".to_string() } else { "found".to_string() },
                            is_damaged,
                            is_thumbnail,
                            confidence: if is_damaged { 0.6 } else { 0.95 },
                            found_at: chrono::Utc::now().to_rfc3339(),
                        });

                        break; // Only match first signature
                    }
                }
            }
        }
    }

    files
}

/// Recover files to destination
pub fn recover_files(
    files_to_recover: &[RecoveredFile],
    config: &ScanConfig,
) -> Vec<RecoveredFile> {
    let mut recovered = Vec::new();
    let dest = PathBuf::from(&config.destination);

    // Create destination directory
    let _ = fs::create_dir_all(&dest);

    for file in files_to_recover {
        let src = PathBuf::from(&file.path);
        if !src.exists() {
            continue;
        }

        // Create category subdirectory
        let category_dir = dest.join(&file.category);
        let _ = fs::create_dir_all(&category_dir);

        // Generate unique filename to avoid conflicts
        let ext = &file.file_type;
        let base_name = Path::new(&file.original_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("recovered");

        let recovered_name = format!("{}_{}.{}", base_name, &file.id[..8], ext);
        let dest_path = category_dir.join(&recovered_name);

        // Copy file
        match fs::copy(&src, &dest_path) {
            Ok(_) => {
                let mut rec = file.clone();
                rec.recovered_path = dest_path.to_string_lossy().to_string();
                rec.status = if file.is_damaged && config.repair_damaged {
                    "repaired".to_string()
                } else if file.is_damaged {
                    "recovered_damaged".to_string()
                } else {
                    "recovered".to_string()
                };

                // Attempt repair if configured
                if file.is_damaged && config.repair_damaged {
                    repair_file(&dest_path, &rec);
                }

                recovered.push(rec);
            }
            Err(e) => {
                let mut rec = file.clone();
                rec.status = format!("error: {}", e);
                recovered.push(rec);
            }
        }
    }

    recovered
}

fn get_search_directories(base: &str) -> Vec<String> {
    let mut dirs = vec![base.to_string()];

    // Add common directories where photos/files are stored
    let common_dirs = [
        "DCIM", "Camera", "Photos", "Pictures", "Images",
        "Documents", "Downloads", "Videos", "Music",
        "WhatsApp", "Telegram", "Screenshots",
    ];

    for dir in &common_dirs {
        let full_path = format!("{}/{}", base, dir);
        if Path::new(&full_path).exists() {
            dirs.push(full_path);
        }
    }

    // On Linux, also check home directories
    #[cfg(target_os = "linux")]
    {
        let home_dirs = [
            format!("{}/home", base),
            format!("{}/root", base),
        ];
        for hd in &home_dirs {
            let hd_path = Path::new(hd);
            if hd_path.exists() {
                if let Ok(entries) = fs::read_dir(hd_path) {
                    for entry in entries.flatten() {
                        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            dirs.push(entry.path().to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    dirs
}

fn is_thumbnail_file(path: &str, size: u64) -> bool {
    let path_lower = path.to_lowercase();

    // Check filename patterns
    let thumb_patterns = [
        "thumb", "thumbnail", ".thumb", "tn_", "_tn",
        "preview", "icon", "cache", ".tmp", "~$",
    ];

    for pattern in &thumb_patterns {
        if path_lower.contains(pattern) {
            return true;
        }
    }

    // Check if file is very small (likely a thumbnail)
    if size < 5_000 && (path_lower.contains(".jpg") || path_lower.contains(".png")) {
        return true;
    }

    // Check for thumbnail directories
    let thumb_dirs = [".thumbnails", "thumbs", "thumbnails", "@__thumb"];
    for dir in &thumb_dirs {
        if path_lower.contains(dir) {
            return true;
        }
    }

    false
}

fn check_file_integrity(file: &mut fs::File, sig: &FileSignature, _size: u64) -> bool {
    // Check footer if available
    if let Some(footer) = sig.footer {
        if file.seek(SeekFrom::End(-(footer.len() as i64))).is_ok() {
            let mut end_bytes = vec![0u8; footer.len()];
            if file.read(&mut end_bytes).is_ok() {
                if end_bytes != footer {
                    return true; // Missing footer = potentially damaged
                }
            }
        }
    }

    // Check for zero-filled sections (data corruption)
    if file.seek(SeekFrom::Start(1024)).is_ok() {
        let mut check_buf = [0u8; 4096];
        if let Ok(n) = file.read(&mut check_buf) {
            if n > 0 {
                let zero_count = check_buf[..n].iter().filter(|&&b| b == 0).count();
                if zero_count > n * 90 / 100 {
                    return true; // Mostly zeros = likely damaged
                }
            }
        }
    }

    false
}

fn repair_file(path: &PathBuf, file: &RecoveredFile) {
    let ext = file.file_type.to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" => repair_jpeg(path),
        "png" => repair_png(path),
        "mp4" | "mov" | "avi" | "mkv" => repair_video(path),
        "mp3" => repair_mp3(path),
        "pdf" => repair_pdf(path),
        _ => {}
    }
}

fn repair_jpeg(path: &PathBuf) {
    // Ensure JPEG has proper SOI and EOI markers
    if let Ok(mut data) = fs::read(path) {
        let mut modified = false;

        // Check SOI marker
        if data.len() >= 2 && data[0] != 0xFF && data[1] != 0xD8 {
            // Prepend SOI
            let mut new_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
            new_data.extend_from_slice(&data);
            data = new_data;
            modified = true;
        }

        // Check EOI marker
        if data.len() >= 2 {
            let len = data.len();
            if data[len - 2] != 0xFF || data[len - 1] != 0xD9 {
                data.push(0xFF);
                data.push(0xD9);
                modified = true;
            }
        }

        if modified {
            let _ = fs::write(path, &data);
        }
    }
}

fn repair_png(path: &PathBuf) {
    if let Ok(mut data) = fs::read(path) {
        // Check PNG signature
        let png_sig: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        if data.len() < 8 || data[..8] != png_sig {
            let mut new_data = png_sig.to_vec();
            new_data.extend_from_slice(&data);
            data = new_data;
        }

        // Ensure IEND chunk
        let iend: [u8; 12] = [0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82];
        let len = data.len();
        if len < 12 || data[len-12..] != iend {
            data.extend_from_slice(&iend);
        }

        let _ = fs::write(path, &data);
    }
}

fn repair_video(path: &PathBuf) {
    // Basic video repair: check and fix container headers
    if let Ok(mut data) = fs::read(path) {
        if data.len() < 12 {
            return;
        }

        // Check for moov atom (MP4/MOV)
        let has_moov = data.windows(4).any(|w| w == b"moov");
        let has_mdat = data.windows(4).any(|w| w == b"mdat");

        if !has_moov && has_mdat {
            // Video might be unplayable without moov atom
            // Mark as needing external repair tool
            log::warn!("Video file missing moov atom, needs external repair: {:?}", path);
        }

        let _ = fs::write(path, &data);
    }
}

fn repair_mp3(path: &PathBuf) {
    if let Ok(data) = fs::read(path) {
        // Check for valid MP3 frame sync
        if data.len() < 3 {
            return;
        }

        let has_sync = data[0] == 0xFF && (data[1] & 0xE0) == 0xE0;
        let has_id3 = &data[0..3] == b"ID3";

        if !has_sync && !has_id3 {
            // Try to find frame sync in first 4KB
            if let Some(pos) = data[..std::cmp::min(4096, data.len())].windows(2).position(|w| {
                w[0] == 0xFF && (w[1] & 0xE0) == 0xE0
            }) {
                if pos > 0 {
                    let trimmed = data[pos..].to_vec();
                    let _ = fs::write(path, &trimmed);
                }
            }
        }
    }
}

fn repair_pdf(path: &PathBuf) {
    if let Ok(mut data) = fs::read(path) {
        // Ensure PDF header
        if !data.starts_with(b"%PDF") {
            let mut new_data = b"%PDF-1.4\n".to_vec();
            new_data.extend_from_slice(&data);
            data = new_data;
        }

        // Ensure EOF marker
        let data_str = String::from_utf8_lossy(&data);
        if !data_str.contains("%%EOF") {
            data.extend_from_slice(b"\n%%EOF\n");
        }

        let _ = fs::write(path, &data);
    }
}
