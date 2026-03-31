use crate::app::state::AppPaths;

#[derive(Debug)]
pub struct ClothoService {
    #[allow(dead_code)]
    paths: AppPaths,
}

impl ClothoService {
    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }

    #[allow(dead_code)]
    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }
}
