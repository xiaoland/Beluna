use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use metrics::{Unit, describe_gauge, gauge};
use metrics_exporter_prometheus::{BuildError, PrometheusBuilder};

pub const CORTEX_CYCLE_ID_METRIC: &str = "beluna_cortex_cycle_id";
pub const CORTEX_INPUT_IR_ACT_DESCRIPTOR_CATALOG_COUNT_METRIC: &str =
    "beluna_cortex_input_ir_act_descriptor_catalog_count";

const DEFAULT_METRICS_PORT: u16 = 9464;

#[derive(Debug, Clone, Copy)]
pub struct MetricsRuntime {
    pub listen_addr: SocketAddr,
}

impl MetricsRuntime {
    pub fn default_listen_addr() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), DEFAULT_METRICS_PORT)
    }
}

pub fn start_prometheus_exporter(listen_addr: SocketAddr) -> Result<MetricsRuntime, BuildError> {
    describe_gauge!(
        CORTEX_CYCLE_ID_METRIC,
        Unit::Count,
        "Latest cortex cycle id processed by stem."
    );
    describe_gauge!(
        CORTEX_INPUT_IR_ACT_DESCRIPTOR_CATALOG_COUNT_METRIC,
        Unit::Count,
        "Count of act descriptors included in cortex input IR catalog."
    );

    PrometheusBuilder::new()
        .with_http_listener(listen_addr)
        .install()?;

    Ok(MetricsRuntime { listen_addr })
}

pub fn record_cortex_cycle_id(cycle_id: u64) {
    gauge!(CORTEX_CYCLE_ID_METRIC).set(cycle_id as f64);
}

pub fn record_cortex_input_ir_act_descriptor_catalog_count(count: usize) {
    gauge!(CORTEX_INPUT_IR_ACT_DESCRIPTOR_CATALOG_COUNT_METRIC).set(count as f64);
}
