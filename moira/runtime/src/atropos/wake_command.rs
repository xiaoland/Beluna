use std::process::Stdio;

use tokio::process::Command;

use crate::clotho::model::PreparedRuntimeWakeInput;

pub(super) fn build_wake_command(wake_input: &PreparedRuntimeWakeInput) -> Command {
    let mut command = Command::new(&wake_input.target.executable_path);
    if let Some(config_path) = wake_input.config_path.as_ref() {
        command.arg("--config").arg(config_path);
    }
    command
        .envs(&wake_input.environment_overrides)
        .current_dir(&wake_input.target.working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    command
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        ffi::OsString,
        path::{Path, PathBuf},
    };

    use crate::clotho::model::{LaunchTargetProvenance, PreparedLaunchTarget};

    use super::*;

    #[test]
    fn build_wake_command_sets_config_path_and_environment_overrides() {
        let wake_input = PreparedRuntimeWakeInput {
            target: PreparedLaunchTarget {
                target_label: "dev-core".to_string(),
                executable_path: PathBuf::from("/tmp/bin/beluna"),
                working_dir: PathBuf::from("/tmp/workdir"),
                provenance: LaunchTargetProvenance::Registered,
            },
            profile_path: Some(PathBuf::from("/tmp/profiles/default.jsonc")),
            config_path: Some(PathBuf::from("/tmp/profiles/.default.core.json")),
            environment_overrides: BTreeMap::from([
                ("BAILIAN_API_KEY".to_string(), "bailian".to_string()),
                ("OPENAI_API_KEY".to_string(), "openai".to_string()),
            ]),
        };

        let command = build_wake_command(&wake_input);
        let std_command = command.as_std();
        let args = std_command
            .get_args()
            .map(|value| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let envs = std_command
            .get_envs()
            .map(|(key, value)| (key.to_os_string(), value.map(|item| item.to_os_string())))
            .collect::<BTreeMap<OsString, Option<OsString>>>();

        assert_eq!(std_command.get_program(), Path::new("/tmp/bin/beluna"));
        assert_eq!(
            std_command.get_current_dir(),
            Some(Path::new("/tmp/workdir"))
        );
        assert_eq!(
            args,
            vec![
                "--config".to_string(),
                "/tmp/profiles/.default.core.json".to_string()
            ]
        );
        assert_eq!(
            envs.get(&OsString::from("BAILIAN_API_KEY")),
            Some(&Some(OsString::from("bailian")))
        );
        assert_eq!(
            envs.get(&OsString::from("OPENAI_API_KEY")),
            Some(&Some(OsString::from("openai")))
        );
    }

    #[test]
    fn build_wake_command_omits_config_argument_when_no_profile_is_selected() {
        let wake_input = PreparedRuntimeWakeInput {
            target: PreparedLaunchTarget {
                target_label: "dev-core".to_string(),
                executable_path: PathBuf::from("/tmp/bin/beluna"),
                working_dir: PathBuf::from("/tmp/workdir"),
                provenance: LaunchTargetProvenance::Registered,
            },
            profile_path: None,
            config_path: None,
            environment_overrides: BTreeMap::new(),
        };

        let command = build_wake_command(&wake_input);
        let std_command = command.as_std();

        assert_eq!(std_command.get_args().count(), 0);
        assert_eq!(std_command.get_envs().count(), 0);
    }
}
