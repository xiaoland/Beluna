use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use serde_json::Value;

use super::service::canonicalize_file;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PreparedProfileRuntime {
    pub profile_path: PathBuf,
    pub config_path: PathBuf,
    pub environment_overrides: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WrapperProfileDocument {
    core_config: Value,
    #[serde(default)]
    environment: ProfileEnvironment,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileEnvironment {
    #[serde(default)]
    env_files: Vec<ProfileEnvFile>,
    #[serde(default)]
    inline: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileEnvFile {
    path: PathBuf,
    #[serde(default = "default_env_file_required")]
    required: bool,
}

fn default_env_file_required() -> bool {
    true
}

pub(super) fn prepare_profile_runtime(
    profile_id: &str,
    profile_path: &Path,
) -> Result<PreparedProfileRuntime, String> {
    let profile_path = canonicalize_file("profile document", profile_path)?;
    let contents = fs::read_to_string(&profile_path).map_err(|err| {
        format!(
            "failed to read profile document `{}`: {err}",
            profile_path.display()
        )
    })?;
    let wrapper: WrapperProfileDocument = json5::from_str(&contents).map_err(|err| {
        format!(
            "failed to parse profile document `{}` as wrapper profile: {err}",
            profile_path.display()
        )
    })?;
    if !matches!(wrapper.core_config, Value::Object(_)) {
        return Err(format!(
            "profile document `{}` must define `core_config` as an object",
            profile_path.display()
        ));
    }

    let config_path = materialize_core_config(profile_id, &profile_path, &wrapper.core_config)?;
    let environment_overrides = resolve_environment_overrides(&profile_path, wrapper.environment)?;

    Ok(PreparedProfileRuntime {
        profile_path,
        config_path,
        environment_overrides,
    })
}

fn materialize_core_config(
    profile_id: &str,
    profile_path: &Path,
    core_config: &Value,
) -> Result<PathBuf, String> {
    let Some(profile_dir) = profile_path.parent() else {
        return Err(format!(
            "profile document `{}` must have a parent directory",
            profile_path.display()
        ));
    };

    let rendered = serde_json::to_string_pretty(core_config).map_err(|err| {
        format!(
            "failed to render `core_config` for profile document `{}`: {err}",
            profile_path.display()
        )
    })?;
    let config_path = profile_dir.join(format!(".{profile_id}.core.json"));
    fs::write(&config_path, ensure_trailing_newline(rendered)).map_err(|err| {
        format!(
            "failed to materialize Core config for profile document `{}` at `{}`: {err}",
            profile_path.display(),
            config_path.display()
        )
    })?;

    canonicalize_file("prepared Core config", &config_path)
}

fn resolve_environment_overrides(
    profile_path: &Path,
    environment: ProfileEnvironment,
) -> Result<BTreeMap<String, String>, String> {
    let mut overrides = BTreeMap::new();

    for env_file in environment.env_files {
        let candidate = resolve_env_file_path(profile_path, &env_file.path)?;
        if !candidate.exists() {
            if env_file.required {
                return Err(format!(
                    "required profile env file `{}` does not exist",
                    candidate.display()
                ));
            }
            continue;
        }

        let env_file_path = canonicalize_file("profile env file", &candidate)?;
        let iter = dotenvy::from_path_iter(&env_file_path).map_err(|err| {
            format!(
                "failed to open profile env file `{}`: {err}",
                env_file_path.display()
            )
        })?;
        for entry in iter {
            let (key, value) = entry.map_err(|err| {
                format!(
                    "failed to parse profile env file `{}`: {err}",
                    env_file_path.display()
                )
            })?;
            validate_env_entry(
                &key,
                &value,
                &format!("profile env file `{}`", env_file_path.display()),
            )?;
            overrides.insert(key, value);
        }
    }

    for (key, value) in environment.inline {
        validate_env_entry(&key, &value, "profile inline environment")?;
        overrides.insert(key, value);
    }

    Ok(overrides)
}

fn resolve_env_file_path(profile_path: &Path, env_file_path: &Path) -> Result<PathBuf, String> {
    if env_file_path.as_os_str().is_empty() {
        return Err(format!(
            "profile document `{}` contains an empty env file path",
            profile_path.display()
        ));
    }

    Ok(if env_file_path.is_absolute() {
        env_file_path.to_path_buf()
    } else {
        profile_path
            .parent()
            .expect("validated profile path should have a parent directory")
            .join(env_file_path)
    })
}

fn validate_env_entry(key: &str, value: &str, source: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err(format!(
            "{source} contains an empty environment variable name"
        ));
    }
    if key.contains('=') {
        return Err(format!(
            "{source} contains invalid environment variable name `{key}`: `=` is not allowed"
        ));
    }
    if key.contains('\0') {
        return Err(format!(
            "{source} contains invalid environment variable name `{key}`: NUL is not allowed"
        ));
    }
    if value.contains('\0') {
        return Err(format!(
            "{source} contains invalid environment variable `{key}`: NUL is not allowed in values"
        ));
    }

    Ok(())
}

fn ensure_trailing_newline(contents: String) -> String {
    if contents.ends_with('\n') {
        contents
    } else {
        format!("{contents}\n")
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs, path::PathBuf};

    use serde_json::json;
    use uuid::Uuid;

    use super::prepare_profile_runtime;

    #[test]
    fn prepare_profile_runtime_materializes_core_config_and_merges_environment_sources() {
        let sandbox = TestSandbox::new();
        let profile_path = sandbox.create_file_with_contents(
            "profiles/default.jsonc",
            r#"
            {
              core_config: {
                logging: {
                  dir: "./logs"
                }
              },
              environment: {
                env_files: [{ path: "./local.env" }],
                inline: {
                  OPENAI_API_KEY: "inline-openai",
                  BAILIAN_API_KEY: "inline-bailian"
                }
              }
            }
            "#,
        );
        sandbox.create_file_with_contents(
            "profiles/local.env",
            "OPENAI_API_KEY=file-openai\nQWEN_API_KEY=file-qwen\n",
        );

        let prepared = prepare_profile_runtime("default", &profile_path)
            .expect("profile runtime should prepare");

        assert_eq!(
            prepared.profile_path,
            fs::canonicalize(&profile_path).expect("profile should canonicalize")
        );
        assert_eq!(
            prepared.config_path.parent(),
            prepared.profile_path.parent(),
            "materialized config must stay beside the wrapper profile so Core relative paths remain stable"
        );
        assert_eq!(
            fs::read_to_string(&prepared.config_path).expect("prepared config should read"),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "logging": {
                        "dir": "./logs"
                    }
                }))
                .expect("config should render")
            )
        );
        assert_eq!(
            prepared.environment_overrides.get("OPENAI_API_KEY"),
            Some(&"inline-openai".to_string())
        );
        assert_eq!(
            prepared.environment_overrides.get("QWEN_API_KEY"),
            Some(&"file-qwen".to_string())
        );
        assert_eq!(
            prepared.environment_overrides.get("BAILIAN_API_KEY"),
            Some(&"inline-bailian".to_string())
        );
    }

    #[test]
    fn prepare_profile_runtime_skips_missing_optional_env_file() {
        let sandbox = TestSandbox::new();
        let profile_path = sandbox.create_file_with_contents(
            "profiles/default.jsonc",
            r#"
            {
              core_config: {
                logging: {
                  dir: "./logs"
                }
              },
              environment: {
                env_files: [{ path: "./missing.env", required: false }],
                inline: {
                  ONLY_INLINE: "present"
                }
              }
            }
            "#,
        );

        let prepared = prepare_profile_runtime("default", &profile_path)
            .expect("optional env file should skip");

        assert_eq!(
            prepared.environment_overrides,
            BTreeMap::from([("ONLY_INLINE".to_string(), "present".to_string())])
        );
    }

    #[test]
    fn prepare_profile_runtime_rejects_missing_required_env_file() {
        let sandbox = TestSandbox::new();
        let profile_path = sandbox.create_file_with_contents(
            "profiles/default.jsonc",
            r#"
            {
              core_config: {
                logging: {
                  dir: "./logs"
                }
              },
              environment: {
                env_files: [{ path: "./missing.env" }]
              }
            }
            "#,
        );

        let err = prepare_profile_runtime("default", &profile_path)
            .expect_err("missing required env file should fail");

        assert!(err.contains("required profile env file"));
        assert!(err.contains("missing.env"));
    }

    struct TestSandbox {
        root: PathBuf,
    }

    impl TestSandbox {
        fn new() -> Self {
            let root =
                std::env::temp_dir().join(format!("moira-profile-runtime-test-{}", Uuid::now_v7()));
            fs::create_dir_all(&root).expect("sandbox should create");
            Self { root }
        }

        fn create_file_with_contents(&self, relative: &str, contents: impl AsRef<[u8]>) -> PathBuf {
            let path = self.root.join(relative);
            let parent = path.parent().expect("file should have parent");
            fs::create_dir_all(parent).expect("file parent should create");
            fs::write(&path, contents).expect("file should write");
            path
        }
    }

    impl Drop for TestSandbox {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
