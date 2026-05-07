mod common;

use std::{fs, path::Path};

use moira_runtime::clotho::model::{
    KnownLocalBuildRegistration, LaunchTargetRef, ProfileRef, WakeInputRequest,
};

use crate::common::{RuntimeSandbox, wait_for_receiver_ready};

#[tokio::test]
async fn runtime_clotho_registers_profile_and_prepares_wake_input() {
    let sandbox = RuntimeSandbox::new();
    let runtime = sandbox.open_runtime().await;
    wait_for_receiver_ready(runtime.as_ref()).await;

    let bin_dir = sandbox.root().join("bin");
    fs::create_dir_all(&bin_dir).expect("bin dir should create");
    let executable = bin_dir.join("beluna-core-fixture");
    fs::write(&executable, "#!/bin/sh\nexit 0\n").expect("fixture executable should write");
    make_executable(&executable);

    let target = runtime
        .clotho()
        .register_known_local_build(KnownLocalBuildRegistration {
            build_id: "dev-core".to_string(),
            executable_path: executable.clone(),
            working_dir: Some(bin_dir.clone()),
            source_dir: None,
        })
        .expect("known local build should register");

    let profiles_dir = sandbox.root().join("profiles");
    fs::write(
        profiles_dir.join("local.env"),
        "OPENAI_API_KEY=file-openai\nQWEN_API_KEY=file-qwen\n",
    )
    .expect("env file should write");
    fs::write(
        profiles_dir.join("default.jsonc"),
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
              OPENAI_API_KEY: "inline-openai"
            }
          }
        }
        "#,
    )
    .expect("profile should write");

    let prepared = runtime
        .clotho()
        .prepare_wake_input(&WakeInputRequest {
            target: target.clone(),
            profile: Some(ProfileRef {
                profile_id: "default".to_string(),
            }),
        })
        .expect("wake input should prepare");

    assert_eq!(
        target,
        LaunchTargetRef::KnownLocalBuild {
            build_id: "dev-core".to_string()
        }
    );
    assert_eq!(
        prepared.target.executable_path,
        fs::canonicalize(&executable).expect("executable should canonicalize")
    );
    assert_eq!(
        prepared.target.working_dir,
        fs::canonicalize(&bin_dir).expect("working dir should canonicalize")
    );
    let expected_profile_path =
        fs::canonicalize(profiles_dir.join("default.jsonc")).expect("profile should canonicalize");
    assert_eq!(
        prepared.profile_path.as_deref(),
        Some(expected_profile_path.as_path())
    );

    let materialized_config = profiles_dir.join(".default.core.json");
    let config = fs::read_to_string(&materialized_config)
        .expect("prepared Core config should materialize beside profile");
    assert!(config.contains("\"logging\""));
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .expect("fixture executable metadata should load")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("fixture executable should become executable");
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}
