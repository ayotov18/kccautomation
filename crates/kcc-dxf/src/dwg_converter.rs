use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    #[error(
        "DWG conversion requires ODA File Converter. Install from https://www.opendesign.com/guestfiles/oda_file_converter or upload a DXF file instead."
    )]
    NoConverterAvailable,
    #[error("ODA File Converter not found at: {0}")]
    NotFound(String),
    #[error("conversion failed: {0}")]
    Failed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("output DXF file not found after conversion")]
    OutputNotFound,
}

/// Wrapper for ODA File Converter.
pub struct DwgConverter {
    binary_path: PathBuf,
}

impl DwgConverter {
    /// Try to find a DWG converter in common locations.
    pub fn auto_detect() -> Result<Self, ConvertError> {
        let candidates = [
            // Linux
            "/usr/local/bin/ODAFileConverter",
            "/opt/ODAFileConverter/ODAFileConverter",
            "/usr/bin/ODAFileConverter",
            // macOS app bundle
            "/Applications/ODAFileConverter.app/Contents/MacOS/ODAFileConverter",
            // Custom env var
        ];

        // Check ODA_PATH env var first
        if let Ok(path) = std::env::var("ODA_CONVERTER_PATH") {
            let p = PathBuf::from(&path);
            if p.exists() {
                return Ok(Self { binary_path: p });
            }
        }

        for path in &candidates {
            let p = PathBuf::from(path);
            if p.exists() {
                return Ok(Self { binary_path: p });
            }
        }

        // Check PATH
        if let Ok(output) = Command::new("which").arg("ODAFileConverter").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(Self {
                        binary_path: PathBuf::from(path),
                    });
                }
            }
        }

        Err(ConvertError::NoConverterAvailable)
    }

    /// Convert DWG bytes to DXF bytes. Returns the DXF content.
    pub fn convert_bytes(&self, dwg_bytes: &[u8], filename: &str) -> Result<Vec<u8>, ConvertError> {
        let temp_dir = tempfile::tempdir().map_err(ConvertError::Io)?;
        let input_dir = temp_dir.path().join("input");
        let output_dir = temp_dir.path().join("output");
        std::fs::create_dir_all(&input_dir)?;
        std::fs::create_dir_all(&output_dir)?;

        // Write DWG to temp input
        let dwg_path = input_dir.join(filename);
        std::fs::write(&dwg_path, dwg_bytes)?;

        // Run ODA: ODAFileConverter "input" "output" ACAD2018 DXF 0 1
        let output = Command::new(&self.binary_path)
            .arg(input_dir.to_str().unwrap())
            .arg(output_dir.to_str().unwrap())
            .arg("ACAD2018")
            .arg("DXF")
            .arg("0")
            .arg("1")
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ConvertError::Failed(format!(
                "ODA exited with {}: {stderr}",
                output.status
            )));
        }

        // Find the output DXF file
        let stem = Path::new(filename)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let dxf_path = output_dir.join(format!("{stem}.dxf"));

        if dxf_path.exists() {
            return Ok(std::fs::read(&dxf_path)?);
        }

        // Try to find any .dxf file in output_dir
        for entry in std::fs::read_dir(&output_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|e| e.to_str()) == Some("dxf") {
                return Ok(std::fs::read(entry.path())?);
            }
        }

        Err(ConvertError::OutputNotFound)
    }

    /// Convert a DWG file on disk to DXF.
    pub fn convert_file(
        &self,
        input_dwg: &Path,
        output_dir: &Path,
    ) -> Result<PathBuf, ConvertError> {
        if !self.binary_path.exists() {
            return Err(ConvertError::NotFound(
                self.binary_path.display().to_string(),
            ));
        }

        let temp_input = tempfile::tempdir().map_err(ConvertError::Io)?;
        let dwg_filename = input_dwg
            .file_name()
            .ok_or_else(|| ConvertError::Failed("invalid input path".to_string()))?;

        std::fs::copy(input_dwg, temp_input.path().join(dwg_filename))?;
        std::fs::create_dir_all(output_dir)?;

        let output = Command::new(&self.binary_path)
            .arg(temp_input.path().to_str().unwrap())
            .arg(output_dir.to_str().unwrap())
            .arg("ACAD2018")
            .arg("DXF")
            .arg("0")
            .arg("1")
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ConvertError::Failed(format!(
                "ODA exited with {}: {stderr}",
                output.status
            )));
        }

        let stem = input_dwg.file_stem().unwrap_or_default().to_string_lossy();
        let dxf_path = output_dir.join(format!("{stem}.dxf"));

        if dxf_path.exists() {
            return Ok(dxf_path);
        }

        for entry in std::fs::read_dir(output_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|e| e.to_str()) == Some("dxf") {
                return Ok(entry.path());
            }
        }

        Err(ConvertError::OutputNotFound)
    }
}

/// Check if the given bytes look like a DWG file (binary magic check).
pub fn is_dwg_bytes(bytes: &[u8]) -> bool {
    // DWG files start with "AC10" version string
    bytes.len() >= 6 && bytes[0..4] == *b"AC10"
}

/// Check if the given bytes look like a DXF file (text format check).
pub fn is_dxf_bytes(bytes: &[u8]) -> bool {
    // DXF text files typically start with "0\nSECTION" or whitespace
    if bytes.is_empty() {
        return false;
    }
    // Check first meaningful characters — DXF starts with group code "0" or whitespace
    let text = String::from_utf8_lossy(&bytes[..bytes.len().min(100)]);
    let trimmed = text.trim();
    trimmed.starts_with('0') || trimmed.starts_with("999")
}
