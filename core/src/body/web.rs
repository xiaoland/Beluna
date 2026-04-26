use reqwest::{
    Client, Method, Url,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use tokio::time::{Duration, timeout};

use crate::{
    body::{
        WEB_SENSE_FETCH_RESULT_ID,
        payloads::{WebFetchRequest, WebLimits},
    },
    spine::adapters::inline::InlineSenseDatum,
    spine::types::EndpointExecutionOutcome,
    types::Act,
};

pub struct WebHandlerOutput {
    pub outcome: EndpointExecutionOutcome,
    pub sense: Option<InlineSenseDatum>,
}

pub async fn handle_web_invoke(
    _request_id: &str,
    act: &Act,
    limits: &WebLimits,
) -> WebHandlerOutput {
    let web_request: WebFetchRequest = match serde_json::from_value(act.payload.clone()) {
        Ok(request) => request,
        Err(_) => {
            return WebHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "invalid_payload".to_string(),
                    reference_id: format!("body.std.web:invalid_payload:{}", act.act_instance_id),
                },
                sense: None,
            };
        }
    };

    let url = match Url::parse(&web_request.url) {
        Ok(url) if matches!(url.scheme(), "http" | "https") => url,
        Ok(_) => {
            return WebHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "unsupported_scheme".to_string(),
                    reference_id: format!(
                        "body.std.web:unsupported_scheme:{}",
                        act.act_instance_id
                    ),
                },
                sense: None,
            };
        }
        Err(_) => {
            return WebHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "invalid_payload".to_string(),
                    reference_id: format!("body.std.web:invalid_url:{}", act.act_instance_id),
                },
                sense: None,
            };
        }
    };

    let method = match Method::from_bytes(web_request.method().as_bytes()) {
        Ok(method) => method,
        Err(_) => {
            return WebHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "invalid_payload".to_string(),
                    reference_id: format!("body.std.web:invalid_method:{}", act.act_instance_id),
                },
                sense: None,
            };
        }
    };

    let headers = match build_headers(&web_request.headers) {
        Ok(headers) => headers,
        Err(reason) => {
            return WebHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "invalid_payload".to_string(),
                    reference_id: format!("body.std.web:{reason}:{}", act.act_instance_id),
                },
                sense: None,
            };
        }
    };

    let client = match Client::builder().no_proxy().build() {
        Ok(client) => client,
        Err(_) => {
            return WebHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "network_error".to_string(),
                    reference_id: format!(
                        "body.std.web:client_build_error:{}",
                        act.act_instance_id
                    ),
                },
                sense: None,
            };
        }
    };
    let timeout_ms = web_request.timeout_ms(limits);
    let response_cap = web_request.response_max_bytes(limits);

    let mut builder = client.request(method, url.clone()).headers(headers);
    if let Some(body_text) = web_request.body_text.clone() {
        builder = builder.body(body_text);
    }

    let response = match timeout(Duration::from_millis(timeout_ms), builder.send()).await {
        Ok(Ok(response)) => response,
        Ok(Err(_)) => {
            return WebHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "network_error".to_string(),
                    reference_id: format!("body.std.web:network_error:{}", act.act_instance_id),
                },
                sense: None,
            };
        }
        Err(_) => {
            return WebHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "timeout".to_string(),
                    reference_id: format!("body.std.web:timeout:{}", act.act_instance_id),
                },
                sense: None,
            };
        }
    };

    let status_code = response.status().as_u16();
    let final_url = response.url().to_string();

    let body_bytes = match timeout(Duration::from_millis(timeout_ms), response.bytes()).await {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(_)) => {
            return WebHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "network_error".to_string(),
                    reference_id: format!("body.std.web:body_read_error:{}", act.act_instance_id),
                },
                sense: None,
            };
        }
        Err(_) => {
            return WebHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "timeout".to_string(),
                    reference_id: format!("body.std.web:body_timeout:{}", act.act_instance_id),
                },
                sense: None,
            };
        }
    };

    let (body_text, body_truncated) = truncate_to_text(body_bytes.as_ref(), response_cap);

    WebHandlerOutput {
        outcome: EndpointExecutionOutcome::Applied {
            actual_cost_micro: 0,
            reference_id: format!("body.std.web:applied:{}", act.act_instance_id),
        },
        sense: Some(InlineSenseDatum {
            sense_instance_id: uuid::Uuid::new_v4().to_string(),
            neural_signal_descriptor_id: WEB_SENSE_FETCH_RESULT_ID.to_string(),
            payload: format!(
                concat!(
                    "web_fetch_result act_instance_id={}; neural_signal_descriptor_id={}; ",
                    "url={}; status_code={}; body_truncated={}; success=true\n",
                    "body:\n{}"
                ),
                act.act_instance_id,
                act.neural_signal_descriptor_id,
                final_url,
                status_code,
                body_truncated,
                body_text
            ),
            weight: 0.0,
            act_instance_id: Some(act.act_instance_id.clone()),
        }),
    }
}

fn build_headers(
    values: &std::collections::BTreeMap<String, String>,
) -> Result<HeaderMap, &'static str> {
    let mut headers = HeaderMap::new();
    for (key, value) in values {
        let name = HeaderName::from_bytes(key.as_bytes()).map_err(|_| "invalid_header_name")?;
        let header_value = HeaderValue::from_str(value).map_err(|_| "invalid_header_value")?;
        headers.insert(name, header_value);
    }

    Ok(headers)
}

fn truncate_to_text(bytes: &[u8], cap: usize) -> (String, bool) {
    if bytes.len() <= cap {
        return (String::from_utf8_lossy(bytes).to_string(), false);
    }

    (String::from_utf8_lossy(&bytes[..cap]).to_string(), true)
}
