use uuid::Uuid;

use crate::types::SenseId;

pub fn derive_act_id(
    cycle_id: u64,
    based_on: &[SenseId],
    endpoint_id: &str,
    neural_signal_descriptor_id: &str,
    payload: &serde_json::Value,
) -> String {
    let _ = (
        cycle_id,
        based_on,
        endpoint_id,
        neural_signal_descriptor_id,
        payload,
    );
    Uuid::now_v7().to_string()
}
