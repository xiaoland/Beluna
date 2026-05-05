use opentelemetry::{SpanId, TraceId};
use sha2::{Digest, Sha256};

pub(crate) fn trace_id(run_id: &str, tick: u64) -> TraceId {
    let digest = digest_parts(
        "beluna.core.trace",
        &[run_id.as_bytes(), &tick.to_be_bytes()],
    );
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    TraceId::from_bytes(bytes)
}

pub(crate) fn span_id(run_id: &str, tick: u64, scope: &str, span_key: &str) -> SpanId {
    let digest = digest_parts(
        "beluna.core.span",
        &[
            run_id.as_bytes(),
            &tick.to_be_bytes(),
            scope.as_bytes(),
            span_key.as_bytes(),
        ],
    );
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    SpanId::from_bytes(bytes)
}

fn digest_parts(domain: &str, parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update([0]);
        hasher.update(part.len().to_be_bytes());
        hasher.update(part);
    }
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_deterministic_and_domain_separated() {
        let trace = trace_id("run-a", 1);
        assert_eq!(trace, trace_id("run-a", 1));
        assert_ne!(trace, trace_id("run-a", 2));

        let span = span_id("run-a", 1, "beluna.core.cortex.primary", "primary");
        assert_eq!(
            span,
            span_id("run-a", 1, "beluna.core.cortex.primary", "primary")
        );
        assert_ne!(
            span,
            span_id("run-a", 1, "beluna.core.cortex.cleanup", "cleanup")
        );
    }
}
