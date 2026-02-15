use tokio::{
    sync::mpsc,
    time::{Duration, timeout},
};

use beluna::{
    ingress::{IngressErrorKind, SenseIngress},
    runtime_types::{Sense, SenseDatum},
};

#[tokio::test]
async fn shutdown_gate_rejects_new_senses_and_sleep_send_blocks_until_space() {
    let (sense_tx, mut sense_rx) = mpsc::channel(1);
    let ingress = SenseIngress::new(sense_tx);

    ingress
        .send(Sense::Domain(SenseDatum {
            sense_id: "41f25f33-99f5-4250-99c3-020f8a92e199".to_string(),
            source: "test".to_string(),
            payload: serde_json::json!({}),
        }))
        .await
        .expect("first sense should be accepted");

    ingress.close_gate().await;

    let err = ingress
        .send(Sense::Domain(SenseDatum {
            sense_id: "sense:2".to_string(),
            source: "test".to_string(),
            payload: serde_json::json!({}),
        }))
        .await
        .expect_err("closed ingress should reject producer sends");
    assert_eq!(err.kind, IngressErrorKind::Closed);

    let ingress_for_sleep = ingress.clone();
    assert!(
        timeout(
            Duration::from_millis(100),
            ingress_for_sleep.send_sleep_blocking(),
        )
        .await
        .is_err()
    );

    let first = sense_rx
        .recv()
        .await
        .expect("first queued sense should exist");
    assert!(matches!(first, Sense::Domain(_)));

    ingress
        .send_sleep_blocking()
        .await
        .expect("sleep should enqueue once queue has space");
    let second = sense_rx.recv().await.expect("sleep should be queued");
    assert!(matches!(second, Sense::Sleep));
}
