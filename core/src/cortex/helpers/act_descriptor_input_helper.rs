use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    cortex::{
        error::CortexError,
        helpers::{self, CognitionOrgan, HelperRuntime},
        prompts,
        testing::ActDescriptorHelperRequest as TestActDescriptorHelperRequest,
    },
    types::{NeuralSignalDescriptor, build_fq_neural_signal_id},
};

#[derive(Clone, Default)]
pub(crate) struct ActDescriptorInputHelper {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl ActDescriptorInputHelper {
    pub(crate) async fn to_input_ir_section(
        &self,
        runtime: &impl HelperRuntime,
        cycle_id: u64,
        act_descriptors: &[NeuralSignalDescriptor],
    ) -> Result<String, CortexError> {
        let stage = CognitionOrgan::ActDescriptor.stage();
        let input_payload = helpers::pretty_json(&serde_json::json!({
            "act_descriptors": act_descriptors,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);

        let cache_key = act_descriptor_cache_key(act_descriptors);
        if let Some(cached) = self.get_cached_section(&cache_key).await {
            tracing::debug!(
                target: "cortex",
                cycle_id = cycle_id,
                cache_key = %cache_key,
                "act_descriptor_helper_cache_hit"
            );
            helpers::log_organ_output(cycle_id, stage, &cached);
            return Ok(cached);
        }

        let generated = if let Some(hooks) = runtime.hooks() {
            (hooks.act_descriptor_helper)(TestActDescriptorHelperRequest {
                cycle_id,
                act_descriptors: act_descriptors.to_vec(),
            })
            .await?
        } else {
            self.build_with_organ(runtime, cycle_id, act_descriptors)
                .await?
        };

        if !generated.trim().is_empty() {
            self.cache_section(cache_key, generated.clone()).await;
        }
        helpers::log_organ_output(cycle_id, stage, &generated);
        Ok(generated)
    }

    async fn build_with_organ(
        &self,
        runtime: &impl HelperRuntime,
        cycle_id: u64,
        act_descriptors: &[NeuralSignalDescriptor],
    ) -> Result<String, CortexError> {
        let mut catalog_entries = Vec::with_capacity(act_descriptors.len());
        for act_descriptor in act_descriptors {
            let prompt = prompts::build_act_descriptor_markdown_prompt(
                &serde_json::to_string_pretty(&act_descriptor.payload_schema)
                    .unwrap_or_else(|_| "{}".to_string()),
            );
            let markdown = runtime
                .run_text_organ_with_system(
                    cycle_id,
                    CognitionOrgan::ActDescriptor,
                    runtime.limits().max_sub_output_tokens,
                    prompts::act_descriptor_helper_system_prompt(),
                    prompt,
                )
                .await?;
            catalog_entries.push(wrap_act_descriptor_catalog_entry(
                act_descriptor,
                markdown.trim(),
            ));
        }
        Ok(catalog_entries.join("\n"))
    }

    async fn get_cached_section(&self, cache_key: &str) -> Option<String> {
        self.cache.read().await.get(cache_key).cloned()
    }

    async fn cache_section(&self, cache_key: String, value: String) {
        self.cache.write().await.insert(cache_key, value);
    }
}

pub(crate) fn fallback_act_descriptor_catalog_section(
    act_descriptors: &[NeuralSignalDescriptor],
) -> String {
    if act_descriptors.is_empty() {
        return "[]".to_string();
    }

    let mut entries = Vec::with_capacity(act_descriptors.len());
    for descriptor in act_descriptors {
        let payload_schema_json = serde_json::to_string_pretty(&descriptor.payload_schema)
            .unwrap_or_else(|_| "{}".to_string());
        entries.push(format!(
            "<act-descriptor fq-act-id=\"{}\">\n## payload-schema\n{}\n</act-descriptor>",
            escape_xml_attr(&build_fq_neural_signal_id(
                &descriptor.endpoint_id,
                &descriptor.neural_signal_descriptor_id,
            )),
            payload_schema_json
        ));
    }
    entries.join("\n")
}

fn act_descriptor_cache_key(act_descriptors: &[NeuralSignalDescriptor]) -> String {
    let canonical = serde_json::to_string(act_descriptors).unwrap_or_else(|_| "[]".to_string());
    format!("{:x}", md5::compute(canonical.as_bytes()))
}

pub(crate) fn wrap_act_descriptor_catalog_entry(
    descriptor: &NeuralSignalDescriptor,
    markdown: &str,
) -> String {
    let fq_act_id = build_fq_neural_signal_id(
        &descriptor.endpoint_id,
        &descriptor.neural_signal_descriptor_id,
    );
    format!(
        "<act-descriptor fq-act-id=\"{}\">\n{}\n</act-descriptor>",
        escape_xml_attr(&fq_act_id),
        markdown.trim(),
    )
}

fn escape_xml_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
