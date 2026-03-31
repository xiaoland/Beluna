use crate::app::state::AppPaths;

#[derive(Debug)]
pub struct AtroposService {
    #[allow(dead_code)]
    paths: AppPaths,
}

impl AtroposService {
    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }

    #[allow(dead_code)]
    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }
}
