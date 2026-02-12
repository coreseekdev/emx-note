use std::path::PathBuf;

/// A Capsa represents a collection of notes
/// (Latin: box/container - used for storing scrolls in ancient Rome)
#[derive(Debug, Clone)]
pub struct Capsa {
    /// Name of the capsa
    pub name: String,
    /// Path to the capsa directory
    pub path: PathBuf,
}

impl Capsa {
    /// Create a new Capsa with the given name and base path
    pub fn new(name: impl Into<String>, base_path: impl Into<PathBuf>) -> Self {
        let name = name.into();
        let path = base_path.into().join(&name);
        Self { name, path }
    }

    /// Get the path to this capsa
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Check if this capsa exists on disk
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Create this capsa directory
    pub fn create(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.path)
    }

    /// List all capsae in the given base directory
    pub fn list_all(base_path: impl AsRef<std::path::Path>) -> std::io::Result<Vec<String>> {
        let mut capsae = Vec::new();
        if base_path.as_ref().exists() {
            for entry in std::fs::read_dir(base_path)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        capsae.push(name.to_string());
                    }
                }
            }
        }
        capsae.sort();
        Ok(capsae)
    }
}
