use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use schemars::schema_for;
use serde_json::Value;

use super::Config;

pub fn generate_schema_value() -> Result<Value> {
    let root_schema = schema_for!(Config);
    let mut value =
        serde_json::to_value(root_schema).context("failed to serialize generated schema")?;

    if let Value::Object(map) = &mut value {
        map.insert(
            "$schema".to_string(),
            Value::String("https://json-schema.org/draft-07/schema#".to_string()),
        );
    }

    Ok(value)
}

pub fn generate_schema_json_pretty() -> Result<String> {
    let schema_value = generate_schema_value()?;
    let mut schema_text =
        serde_json::to_string_pretty(&schema_value).context("failed to render schema as JSON")?;
    schema_text.push('\n');
    Ok(schema_text)
}

pub fn write_schema_to_path(output_path: &Path) -> Result<()> {
    let schema_text = generate_schema_json_pretty()?;
    let output_dir = output_path.parent().ok_or_else(|| {
        anyhow!(
            "schema output path '{}' has no parent directory",
            output_path.display()
        )
    })?;

    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "failed to create schema output directory {}",
            output_dir.display()
        )
    })?;

    let temp_path = build_temp_path(output_path, std::process::id());
    fs::write(&temp_path, schema_text.as_bytes()).with_context(|| {
        format!(
            "failed to write temporary schema file {}",
            temp_path.display()
        )
    })?;

    fs::rename(&temp_path, output_path).with_context(|| {
        format!(
            "failed to move temporary schema file {} to {}",
            temp_path.display(),
            output_path.display()
        )
    })?;

    Ok(())
}

fn build_temp_path(output_path: &Path, process_id: u32) -> PathBuf {
    let file_name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("beluna.schema.json");
    let tmp_name = format!(".{file_name}.{process_id}.tmp");
    output_path.with_file_name(tmp_name)
}
