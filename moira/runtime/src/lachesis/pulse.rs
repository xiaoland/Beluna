use crate::lachesis::model::IngestPulse;

pub fn empty_ingest_pulse() -> IngestPulse {
    IngestPulse {
        touched_run_ids: Vec::new(),
        last_batch_at: String::new(),
    }
}
