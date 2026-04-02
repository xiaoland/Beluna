use std::fs;

use super::{
    ClothoService,
    model::{ProfileDocument, ProfileDocumentSummary, ProfileRef, SaveProfileDocumentRequest},
    service::{canonicalize_file, validate_ref_id},
};

impl ClothoService {
    pub fn list_profile_documents(&self) -> Result<Vec<ProfileDocumentSummary>, String> {
        let mut documents = Vec::new();
        let directory = self.paths().profiles_dir();

        let entries = fs::read_dir(&directory).map_err(|err| {
            format!(
                "failed to read profiles directory `{}`: {err}",
                directory.display()
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|err| {
                format!(
                    "failed to read an entry from profiles directory `{}`: {err}",
                    directory.display()
                )
            })?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("jsonc") {
                continue;
            }

            let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };
            let Ok(profile_id) = validate_ref_id("profile", stem) else {
                continue;
            };

            documents.push(ProfileDocumentSummary {
                profile_id,
                profile_path: canonicalize_file("profile document", &path)?,
            });
        }

        documents.sort_by(|left, right| left.profile_id.cmp(&right.profile_id));
        Ok(documents)
    }

    pub fn load_profile_document(&self, profile: &ProfileRef) -> Result<ProfileDocument, String> {
        let profile_id = validate_ref_id("profile", &profile.profile_id)?;
        let profile_path = self.paths().profile_document_path(&profile_id);
        let contents = fs::read_to_string(&profile_path).map_err(|err| {
            format!(
                "failed to read profile document `{}`: {err}",
                profile_path.display()
            )
        })?;

        Ok(ProfileDocument {
            profile_id,
            profile_path: canonicalize_file("profile document", &profile_path)?,
            contents,
        })
    }

    pub fn save_profile_document(
        &self,
        request: SaveProfileDocumentRequest,
    ) -> Result<ProfileDocument, String> {
        let profile_id = validate_ref_id("profile", &request.profile_id)?;
        let profile_path = self.paths().profile_document_path(&profile_id);

        fs::create_dir_all(self.paths().profiles_dir()).map_err(|err| {
            format!(
                "failed to create profiles directory `{}`: {err}",
                self.paths().profiles_dir().display()
            )
        })?;

        let contents = ensure_trailing_newline(request.contents);
        fs::write(&profile_path, contents.as_bytes()).map_err(|err| {
            format!(
                "failed to write profile document `{}`: {err}",
                profile_path.display()
            )
        })?;

        self.load_profile_document(&ProfileRef { profile_id })
    }
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
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use uuid::Uuid;

    use crate::app::state::AppPaths;

    use super::{super::model::SaveProfileDocumentRequest, ClothoService};

    #[test]
    fn save_list_and_load_profile_documents_round_trip() {
        let sandbox = TestSandbox::new();
        let paths = AppPaths::from_root(sandbox.root.clone());
        paths.ensure_dirs().expect("app paths should initialize");

        let service = ClothoService::new(paths);
        let saved = service
            .save_profile_document(SaveProfileDocumentRequest {
                profile_id: "default".to_string(),
                contents: "{\n  // local config\n}\n".to_string(),
            })
            .expect("profile document should save");

        assert_eq!(saved.profile_id, "default");
        assert!(saved.profile_path.is_file());

        let listed = service
            .list_profile_documents()
            .expect("profile documents should list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].profile_id, "default");

        let loaded = service
            .load_profile_document(&super::super::model::ProfileRef {
                profile_id: "default".to_string(),
            })
            .expect("profile document should load");
        assert_eq!(loaded.contents, "{\n  // local config\n}\n");
    }

    #[test]
    fn list_profile_documents_skips_non_jsonc_files() {
        let sandbox = TestSandbox::new();
        let paths = AppPaths::from_root(sandbox.root.clone());
        paths.ensure_dirs().expect("app paths should initialize");

        fs::write(paths.profiles_dir().join("notes.txt"), "ignore").expect("fixture should write");

        let service = ClothoService::new(paths);
        let listed = service
            .list_profile_documents()
            .expect("profile documents should list");

        assert!(listed.is_empty());
    }

    struct TestSandbox {
        root: PathBuf,
    }

    impl TestSandbox {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("moira-profile-test-{}", Uuid::now_v7()));
            fs::create_dir_all(&root).expect("sandbox should create");
            Self { root }
        }
    }

    impl Drop for TestSandbox {
        fn drop(&mut self) {
            let _ = remove_dir_all_if_exists(&self.root);
        }
    }

    fn remove_dir_all_if_exists(path: &Path) -> std::io::Result<()> {
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }
}
