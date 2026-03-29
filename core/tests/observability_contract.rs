use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use beluna::observability::contract::{FIXTURE_SCHEMA_VERSION, FixtureBundle};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/observability")
}

fn fixture_paths() -> Vec<PathBuf> {
    let mut paths = fs::read_dir(fixture_dir())
        .expect("fixture dir should exist")
        .map(|entry| entry.expect("fixture entry should read").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn load_bundle(path: &Path) -> (String, FixtureBundle) {
    let source = fs::read_to_string(path).expect("fixture should read");
    let bundle = serde_json::from_str::<FixtureBundle>(&source)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));
    bundle
        .validate()
        .unwrap_or_else(|err| panic!("failed to validate {}: {err}", path.display()));
    (source, bundle)
}

#[test]
fn observability_fixture_bundles_parse_and_validate() {
    let paths = fixture_paths();
    assert_eq!(paths.len(), 3, "expected cortex, stem, and spine bundles");

    for path in paths {
        let (_, bundle) = load_bundle(&path);
        assert_eq!(bundle.schema_version, FIXTURE_SCHEMA_VERSION);
        assert!(!bundle.fixtures.is_empty(), "{}", path.display());
    }
}

#[test]
fn observability_fixture_bundles_round_trip_stably() {
    for path in fixture_paths() {
        let (source, bundle) = load_bundle(&path);
        let rendered = serde_json::to_string_pretty(&bundle).expect("bundle should render");
        assert_eq!(
            rendered,
            source.trim_end_matches('\n'),
            "{}",
            path.display()
        );
    }
}

#[test]
fn observability_fixture_bundles_cover_minimum_contract_scenarios() {
    let actual = fixture_paths()
        .into_iter()
        .flat_map(|path| {
            let (_, bundle) = load_bundle(&path);
            bundle
                .fixtures
                .into_iter()
                .map(|fixture| fixture.fixture_id)
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>();

    let expected = BTreeSet::from([
        "cortex.goal_forest.snapshot.inline".to_string(),
        "cortex.goal_forest.snapshot.reference".to_string(),
        "cortex.organ.request.nominal".to_string(),
        "cortex.organ.response.error".to_string(),
        "cortex.organ.response.nominal".to_string(),
        "cortex.tick.minimal".to_string(),
        "cortex.tick.nominal".to_string(),
        "spine.adapter.lifecycle.enable".to_string(),
        "spine.adapter.lifecycle.fault".to_string(),
        "spine.dispatch.outcome.acknowledged".to_string(),
        "spine.dispatch.outcome.lost".to_string(),
        "spine.endpoint.lifecycle.connected".to_string(),
        "spine.endpoint.lifecycle.dropped".to_string(),
        "stem.descriptor.catalog.snapshot".to_string(),
        "stem.descriptor.catalog.update".to_string(),
        "stem.dispatch.transition.queue".to_string(),
        "stem.dispatch.transition.result".to_string(),
        "stem.signal.transition.afferent".to_string(),
        "stem.signal.transition.efferent".to_string(),
    ]);

    assert_eq!(actual, expected);
}
