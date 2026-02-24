use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use tokio::time::{Duration, timeout};

use crate::{
    cortex::{
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
        deadline: Duration,
        act_descriptors: &[NeuralSignalDescriptor],
    ) -> String {
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
            return cached;
        }

        let generated_result = timeout(deadline, async {
            if let Some(hooks) = runtime.hooks() {
                (hooks.act_descriptor_helper)(TestActDescriptorHelperRequest {
                    cycle_id,
                    act_descriptors: act_descriptors.to_vec(),
                })
                .await
            } else {
                self.build_with_organ(runtime, cycle_id, act_descriptors)
                    .await
            }
        })
        .await;

        match generated_result {
            Ok(Ok(generated)) if !generated.trim().is_empty() => {
                self.cache_section(cache_key, generated.clone()).await;
                helpers::log_organ_output(cycle_id, stage, &generated);
                generated
            }
            Ok(Ok(_)) => {
                let fallback = fallback_act_descriptor_catalog_section(act_descriptors);
                helpers::log_organ_output(cycle_id, stage, &fallback);
                fallback
            }
            Ok(Err(err)) => {
                runtime.emit_stage_failed(cycle_id, stage);
                tracing::warn!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    stage = stage,
                    error = %err,
                    "act_descriptor_helper_failed_fallback_raw"
                );
                let fallback = fallback_act_descriptor_catalog_section(act_descriptors);
                helpers::log_organ_output(cycle_id, stage, &fallback);
                fallback
            }
            Err(_) => {
                runtime.emit_stage_failed(cycle_id, stage);
                tracing::warn!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    stage = stage,
                    "act_descriptor_helper_timeout_fallback_raw"
                );
                let fallback = fallback_act_descriptor_catalog_section(act_descriptors);
                helpers::log_organ_output(cycle_id, stage, &fallback);
                fallback
            }
        }
    }

    async fn build_with_organ(
        &self,
        runtime: &impl HelperRuntime,
        cycle_id: u64,
        act_descriptors: &[NeuralSignalDescriptor],
    ) -> Result<String, crate::cortex::error::CortexError> {
        let mut catalog_entries = Vec::with_capacity(act_descriptors.len());
        for act_descriptor in act_descriptors {
            let markdown = if is_complex_payload_schema(&act_descriptor.payload_schema) {
                let prompt = prompts::build_act_descriptor_markdown_prompt(
                    &serde_json::to_string_pretty(&act_descriptor.payload_schema)
                        .unwrap_or_else(|_| "{}".to_string()),
                );
                runtime
                    .run_text_organ_with_system(
                        cycle_id,
                        CognitionOrgan::ActDescriptor,
                        runtime.limits().max_sub_output_tokens,
                        prompts::act_descriptor_helper_system_prompt(),
                        prompt,
                    )
                    .await?
            } else {
                simple_payload_schema_markdown(&act_descriptor.payload_schema)
            };
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
            "<somatic-act-descriptor somatic-act-id=\"{}\">\n## payload-schema\n{}\n</somatic-act-descriptor>",
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

fn is_complex_payload_schema(schema: &serde_json::Value) -> bool {
    top_level_property_count(schema) > 10
        || has_second_level_object(schema)
        || has_array_with_nested_array_or_object(schema)
}

fn top_level_property_count(schema: &serde_json::Value) -> usize {
    schema
        .get("properties")
        .and_then(|props| props.as_object())
        .map_or(0, |props| props.len())
}

fn has_second_level_object(schema: &serde_json::Value) -> bool {
    let Some(properties) = schema.get("properties").and_then(|props| props.as_object()) else {
        return false;
    };
    properties.values().any(is_object_schema)
}

fn has_array_with_nested_array_or_object(schema: &serde_json::Value) -> bool {
    if array_items_contain_array_or_object(schema) {
        return true;
    }
    let Some(properties) = schema.get("properties").and_then(|props| props.as_object()) else {
        return false;
    };
    properties.values().any(array_items_contain_array_or_object)
}

fn array_items_contain_array_or_object(schema: &serde_json::Value) -> bool {
    if !is_array_schema(schema) {
        return false;
    }
    let Some(items) = schema.get("items") else {
        return false;
    };
    match items {
        serde_json::Value::Array(item_schemas) => item_schemas
            .iter()
            .any(|item| is_array_schema(item) || is_object_schema(item)),
        _ => is_array_schema(items) || is_object_schema(items),
    }
}

fn is_object_schema(schema: &serde_json::Value) -> bool {
    json_schema_type_contains(schema, "object")
        || schema
            .get("properties")
            .and_then(|props| props.as_object())
            .is_some_and(|props| !props.is_empty())
}

fn is_array_schema(schema: &serde_json::Value) -> bool {
    json_schema_type_contains(schema, "array") || schema.get("items").is_some()
}

fn json_schema_type_contains(schema: &serde_json::Value, expected: &str) -> bool {
    match schema.get("type") {
        Some(serde_json::Value::String(kind)) => kind == expected,
        Some(serde_json::Value::Array(kinds)) => kinds
            .iter()
            .any(|kind| kind.as_str().is_some_and(|value| value == expected)),
        _ => false,
    }
}

fn simple_payload_schema_markdown(payload_schema: &serde_json::Value) -> String {
    let payload_schema_json =
        serde_json::to_string_pretty(payload_schema).unwrap_or_else(|_| "{}".to_string());
    format!("## payload-schema\n{}", payload_schema_json)
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
        "<somatic-act-descriptor somatic-act-id=\"{}\">\n{}\n</somatic-act-descriptor>",
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
