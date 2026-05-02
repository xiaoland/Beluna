use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::case::FileExpectationSpec;

#[derive(Debug, Clone)]
pub struct CaseWorkspace {
    root: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct FileSeedSpec {
    path: String,
    kind: String,
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileTreeSnapshot {
    pub root: PathBuf,
    pub entries: BTreeMap<String, FileTreeEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileTreeEntry {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub len: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_utf8: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileTreeDiff {
    pub created: Vec<String>,
    pub deleted: Vec<String>,
    pub modified: Vec<String>,
}

impl CaseWorkspace {
    pub fn create(artifact_dir: &Path) -> Result<Self> {
        let root = artifact_dir.join("world");
        fs::create_dir_all(&root)
            .with_context(|| format!("failed to create case workspace {}", root.display()))?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn materialize(&self, files: &[Value]) -> Result<()> {
        for value in files {
            let spec: FileSeedSpec =
                serde_json::from_value(value.clone()).context("failed to parse world file seed")?;
            let path = self.resolve_relative_path(&spec.path)?;
            match spec.kind.as_str() {
                "directory" => {
                    fs::create_dir_all(&path).with_context(|| {
                        format!("failed to create directory seed {}", path.display())
                    })?;
                }
                "file" => {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent).with_context(|| {
                            format!("failed to create parent directory {}", parent.display())
                        })?;
                    }
                    fs::write(&path, spec.content.unwrap_or_default()).with_context(|| {
                        format!("failed to create file seed {}", path.display())
                    })?;
                }
                other => bail!("unsupported world file seed kind '{}'", other),
            }
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Result<FileTreeSnapshot> {
        let mut entries = BTreeMap::new();
        collect_entries(&self.root, &self.root, &mut entries)?;
        Ok(FileTreeSnapshot {
            root: self.root.clone(),
            entries,
        })
    }

    pub fn evaluate_expectations(
        &self,
        expectations: &[FileExpectationSpec],
        snapshot: &FileTreeSnapshot,
    ) -> Vec<String> {
        expectations
            .iter()
            .flat_map(|expectation| self.evaluate_one(expectation, snapshot))
            .collect()
    }

    fn evaluate_one(
        &self,
        expectation: &FileExpectationSpec,
        snapshot: &FileTreeSnapshot,
    ) -> Vec<String> {
        let mut failures = Vec::new();
        let relative = match normalize_case_relative_path(&expectation.path) {
            Ok(relative) => relative,
            Err(err) => return vec![format!("invalid file expectation path: {err}")],
        };
        let key = relative.to_string_lossy().to_string();
        let entry = snapshot.entries.get(&key);

        if expectation.absent {
            if entry.is_some() {
                failures.push(format!("expected '{}' to be absent", key));
            }
            return failures;
        }

        if expectation.exists.unwrap_or(false) && entry.is_none() {
            failures.push(format!("expected '{}' to exist", key));
        }

        if let Some(expected_content) = &expectation.content_exact {
            match entry {
                Some(actual) if actual.kind == "file" => match actual.content_utf8.as_ref() {
                    Some(actual_content) if actual_content == expected_content => {}
                    Some(actual_content) => failures.push(format!(
                        "file '{}' content mismatch: expected {:?}, got {:?}",
                        key, expected_content, actual_content
                    )),
                    None => failures.push(format!("file '{}' content is not UTF-8", key)),
                },
                Some(actual) => failures.push(format!(
                    "expected '{}' to be a file, got {}",
                    key, actual.kind
                )),
                None => failures.push(format!("expected '{}' to exist", key)),
            }
        }

        if let Some(expected_content) = &expectation.content_trimmed_exact {
            match entry {
                Some(actual) if actual.kind == "file" => match actual.content_utf8.as_ref() {
                    Some(actual_content)
                        if trim_terminal_newlines(actual_content) == expected_content => {}
                    Some(actual_content) => failures.push(format!(
                        "file '{}' trimmed content mismatch: expected {:?}, got {:?}",
                        key,
                        expected_content,
                        trim_terminal_newlines(actual_content)
                    )),
                    None => failures.push(format!("file '{}' content is not UTF-8", key)),
                },
                Some(actual) => failures.push(format!(
                    "expected '{}' to be a file, got {}",
                    key, actual.kind
                )),
                None => failures.push(format!("expected '{}' to exist", key)),
            }
        }

        failures
    }

    fn resolve_relative_path(&self, value: &str) -> Result<PathBuf> {
        Ok(self.root.join(normalize_case_relative_path(value)?))
    }
}

impl FileTreeDiff {
    pub fn between(before: &FileTreeSnapshot, after: &FileTreeSnapshot) -> Self {
        let before_keys = before.entries.keys().cloned().collect::<BTreeSet<_>>();
        let after_keys = after.entries.keys().cloned().collect::<BTreeSet<_>>();
        let created = after_keys.difference(&before_keys).cloned().collect();
        let deleted = before_keys.difference(&after_keys).cloned().collect();
        let modified = before_keys
            .intersection(&after_keys)
            .filter(|key| before.entries.get(*key) != after.entries.get(*key))
            .cloned()
            .collect();
        Self {
            created,
            deleted,
            modified,
        }
    }
}

fn collect_entries(
    root: &Path,
    current: &Path,
    entries: &mut BTreeMap<String, FileTreeEntry>,
) -> Result<()> {
    for entry in fs::read_dir(current)
        .with_context(|| format!("failed to read workspace directory {}", current.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .with_context(|| format!("failed to relativize {}", path.display()))?
            .to_string_lossy()
            .to_string();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            entries.insert(
                relative,
                FileTreeEntry {
                    kind: "directory".to_string(),
                    len: None,
                    content_utf8: None,
                },
            );
            collect_entries(root, &path, entries)?;
        } else if metadata.is_file() {
            let bytes = fs::read(&path)
                .with_context(|| format!("failed to read workspace file {}", path.display()))?;
            let content_utf8 = String::from_utf8(bytes).ok();
            entries.insert(
                relative,
                FileTreeEntry {
                    kind: "file".to_string(),
                    len: Some(metadata.len()),
                    content_utf8,
                },
            );
        }
    }
    Ok(())
}

fn normalize_case_relative_path(value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute() {
        bail!("absolute paths are outside the case workspace: {}", value);
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => normalized.push(segment),
            Component::CurDir => {}
            Component::ParentDir => bail!("parent components are outside the case workspace"),
            Component::RootDir | Component::Prefix(_) => {
                bail!("path component is outside the case workspace")
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        bail!("path cannot be empty");
    }
    Ok(normalized)
}

fn trim_terminal_newlines(value: &str) -> &str {
    value.trim_end_matches(['\r', '\n'])
}
