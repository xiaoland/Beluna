use tokio::time::{Duration, timeout};

use beluna::{
    afferent_pathway::{AfferentPathwayErrorKind, SenseAfferentPathway},
    types::{Sense, SenseDatum},
};

#[tokio::test]
async fn shutdown_gate_rejects_new_senses_and_sleep_send_blocks_until_space() {
    let (afferent_pathway, mut sense_rx) = SenseAfferentPathway::new(1);

    afferent_pathway
        .send(Sense::Domain(SenseDatum {
            sense_id: "sense:1".to_string(),
            endpoint_id: "ep.demo".to_string(),
            neural_signal_descriptor_id: "sense.demo".to_string(),
            payload: serde_json::json!({}),
        }))
        .await
        .expect("first sense should be accepted");

    afferent_pathway.close_gate().await;

    let err = afferent_pathway
        .send(Sense::Domain(SenseDatum {
            sense_id: "sense:2".to_string(),
            endpoint_id: "ep.demo".to_string(),
            neural_signal_descriptor_id: "sense.demo".to_string(),
            payload: serde_json::json!({}),
        }))
        .await
        .expect_err("closed afferent pathway should reject producer sends");
    assert_eq!(err.kind, AfferentPathwayErrorKind::Closed);

    let afferent_pathway_for_sleep = afferent_pathway.clone();
    assert!(
        timeout(
            Duration::from_millis(100),
            afferent_pathway_for_sleep.send_sleep_blocking(),
        )
        .await
        .is_err()
    );

    let first = sense_rx
        .recv()
        .await
        .expect("first queued sense should exist");
    assert!(matches!(first, Sense::Domain(_)));

    afferent_pathway
        .send_sleep_blocking()
        .await
        .expect("sleep should enqueue once queue has space");
    let second = sense_rx.recv().await.expect("sleep should be queued");
    assert!(matches!(second, Sense::Sleep));
}
