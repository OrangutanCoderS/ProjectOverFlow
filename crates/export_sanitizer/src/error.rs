use thiserror::Error;

/// Errors for export_sanitizer operations (mostly config / rule loading).
#[derive(Debug, Error)]
pub enum ExportSanitizerError {
    #[error("I/O error loading sanitizer rules: {0}")]
    IoError(#[from] std::io::Error),

    #[error("failed to parse sanitizer YAML rules: {0}")]
    YamlError(#[from] serde_yaml::Error),
}
