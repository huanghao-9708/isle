use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct AudioFile {
    pub path: PathBuf,
    pub file_name: String,
    pub file_size: u64,
}

pub struct FileSystem {}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem {
    pub fn new() -> Self {
        FileSystem {}
    }

    pub fn scan_folder(&self, path: &str) -> Result<Vec<AudioFile>, String> {
        // 路径归一化：处理 Windows 反斜杠
        let normalized_path = path.replace('\\', "/");
        let p = Path::new(&normalized_path);

        info!("FileSystem: 开始扫描目录 -> {}", normalized_path);

        // 检查路径是否存在且是目录
        if !p.exists() {
            error!("FileSystem: 路径不存在 -> {}", normalized_path);
            return Err(format!("Path does not exist: {}", normalized_path));
        }
        if !p.is_dir() {
            error!("FileSystem: 路径不是目录 -> {}", normalized_path);
            return Err(format!("Path is not a directory: {}", normalized_path));
        }

        let mut audio_files = Vec::new();
        let audio_extensions = ["mp3", "flac", "wav", "m4a", "ogg", "aac", "ape"];
        let mut skipped_extensions = std::collections::HashSet::new();
        let mut total_files_seen = 0;

        let walk = WalkDir::new(p);
        for entry_result in walk {
            match entry_result {
                Ok(entry) => {
                    if entry.file_type().is_file() {
                        total_files_seen += 1;
                        if let Some(ext) = entry.path().extension() {
                            if let Some(ext_str) = ext.to_str() {
                                let ext_lower = ext_str.to_lowercase();
                                if audio_extensions.contains(&ext_lower.as_str()) {
                                    if let Ok(metadata) = entry.metadata() {
                                        audio_files.push(AudioFile {
                                            path: entry.path().to_path_buf(),
                                            file_name: entry
                                                .file_name()
                                                .to_string_lossy()
                                                .to_string(),
                                            file_size: metadata.len(),
                                        });
                                    }
                                } else {
                                    skipped_extensions.insert(ext_lower);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("FileSystem: 遍历过程中出错 (Entry Error): {}", e);
                }
            }
        }

        info!(
            "FileSystem: 扫描完成. 总计处理文件: {}, 提取音轨: {}, 发现非音频后缀: {:?}",
            total_files_seen,
            audio_files.len(),
            skipped_extensions
        );

        if audio_files.is_empty() && total_files_seen > 0 {
            warn!("FileSystem: 该目录下包含文件但均不符合支持的音频后缀。");
        }

        Ok(audio_files)
    }

    pub fn compute_file_hash(&self, path: &Path) -> Result<String, String> {
        // 为了向后兼容，保留此名称但内部使用快速逻辑，或者由调用者决定
        self.compute_quick_hash(path)
    }

    /// 快速哈希：基于路径、修改时间和文件大小生成标识符 (毫秒级)
    pub fn compute_quick_hash(&self, path: &Path) -> Result<String, String> {
        let metadata = path.metadata().map_err(|e| e.to_string())?;
        let mtime = metadata
            .modified()
            .map_err(|e| e.to_string())?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let size = metadata.len();
        let path_str = path.to_string_lossy();

        let mut hasher = Sha256::new();
        hasher.update(path_str.as_bytes());
        hasher.update(mtime.to_le_bytes());
        hasher.update(size.to_le_bytes());

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// 慢速哈希：读取整个文件内容生成哈希 (原解析逻辑)
    pub fn compute_full_hash(&self, path: &Path) -> Result<String, String> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    pub fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        std::fs::read(path).map_err(|e| e.to_string())
    }

    pub fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, data).map_err(|e| e.to_string())
    }

    pub fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    pub fn create_dir_all(&self, path: &Path) -> Result<(), String> {
        std::fs::create_dir_all(path).map_err(|e| e.to_string())
    }
}
