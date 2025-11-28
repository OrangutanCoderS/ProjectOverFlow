use std::io::Write;

use policy_diff_viewer::{diff_files, PolicyDiffFormat};
use tempfile::NamedTempFile;

#[test]
fn json_diff_detects_modified_field() -> Result<(), Box<dyn std::error::Error>> {
    let mut f_current = NamedTempFile::new()?;
    let mut f_proposed = NamedTempFile::new()?;

    write!(f_current, r#"{{"rule": 1, "plugin_chain": ["plugin_A", "plugin_B"]}}"#)?;
    write!(f_proposed, r#"{{"rule": 1, "plugin_chain": ["plugin_A", "plugin_X", "plugin_B"]}}"#)?;

    let entries = diff_files(
        f_current.path(),
        f_proposed.path(),
        Some(PolicyDiffFormat::Json),
    )?;

    assert!(
        !entries.is_empty(),
        "expected at least one diff entry for modified plugin_chain"
    );

    Ok(())
}