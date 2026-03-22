use std::path::PathBuf;

use validator::ValidationError;

pub fn validate_non_blank(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::new("non_blank"));
    }
    Ok(())
}

pub fn validate_non_empty_path(path: &PathBuf) -> Result<(), ValidationError> {
    if path.as_os_str().is_empty() {
        return Err(ValidationError::new("non_empty_path"));
    }
    Ok(())
}

pub fn validate_sampling_ratio(value: f64) -> Result<(), ValidationError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        return Ok(());
    }

    Err(ValidationError::new("sampling_ratio"))
}
