use reqwest::{
    Client, Method, Url,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use tokio::time::{Duration, timeout};

use crate::{
    body::payloads::{WebFetchRequest, WebLimits},
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
                    reference_id: format!("body.std.web:invalid_payload:{}", act.act_id),
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
                    reference_id: format!("body.std.web:unsupported_scheme:{}", act.act_id),
                },
                sense: None,
            };
        }
        Err(_) => {
            return WebHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "invalid_payload".to_string(),
                    reference_id: format!("body.std.web:invalid_url:{}", act.act_id),
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
                    reference_id: format!("body.std.web:invalid_method:{}", act.act_id),
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
                    reference_id: format!("body.std.web:{reason}:{}", act.act_id),
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
                    reference_id: format!("body.std.web:client_build_error:{}", act.act_id),
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
                    reference_id: format!("body.std.web:network_error:{}", act.act_id),
                },
                sense: None,
            };
        }
        Err(_) => {
            return WebHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "timeout".to_string(),
                    reference_id: format!("body.std.web:timeout:{}", act.act_id),
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
                    reference_id: format!("body.std.web:body_read_error:{}", act.act_id),
                },
                sense: None,
            };
        }
        Err(_) => {
            return WebHandlerOutput {
                outcome: EndpointExecutionOutcome::Rejected {
                    reason_code: "timeout".to_string(),
                    reference_id: format!("body.std.web:body_timeout:{}", act.act_id),
                },
                sense: None,
            };
        }
    };

    let (body_text, body_truncated) = truncate_to_text(body_bytes.as_ref(), response_cap);

    WebHandlerOutput {
        outcome: EndpointExecutionOutcome::Applied {
            actual_cost_micro: 0,
            reference_id: format!("body.std.web:applied:{}", act.act_id),
        },
        sense: Some(InlineSenseDatum {
            sense_id: uuid::Uuid::new_v4().to_string(),
            neural_signal_descriptor_id: "body.std.web.result".to_string(),
            payload: serde_json::json!({
                "kind": "web_fetch_result",
                "act_id": act.act_id,
                "neural_signal_descriptor_id": act.neural_signal_descriptor_id,
                "url": final_url,
                "status_code": status_code,
                "body_text": body_text,
                "body_truncated": body_truncated,
                "success": true
            }),
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

#[cfg(test)]
mod tests {
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    use crate::{spine::types::EndpointExecutionOutcome, types::Act};

    use super::{WebLimits, handle_web_invoke};

    fn build_request(act_id: &str, payload: serde_json::Value) -> Act {
        Act {
            act_id: act_id.to_string(),
            endpoint_id: "ep:body:std:web".to_string(),
            neural_signal_descriptor_id: "tool.web.fetch".to_string(),
            payload: payload,
        }
    }

    async fn spawn_http_server(body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind should succeed");
        let address = listener.local_addr().expect("local addr should exist");

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept should succeed");
            let mut request_buffer = vec![0u8; 2048];
            let _ = stream.read(&mut request_buffer).await;

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .await
                .expect("response should be written");
        });

        format!("http://{}", address)
    }

    #[tokio::test]
    async fn rejects_unsupported_scheme() {
        let request = build_request(
            "act:unsupported",
            serde_json::json!({"url": "ftp://example.com/file.txt"}),
        );

        let output = handle_web_invoke("req:unsupported", &request, &WebLimits::default()).await;

        assert!(matches!(
            output.outcome,
            EndpointExecutionOutcome::Rejected { ref reason_code, .. } if reason_code == "unsupported_scheme"
        ));
        assert!(output.sense.is_none());
    }

    #[tokio::test]
    async fn applies_and_truncates_response_body() {
        let url = spawn_http_server("hello-from-web-endpoint").await;
        let request = build_request(
            "act:web",
            serde_json::json!({
                "url": url,
                "method": "GET",
                "response_max_bytes": 5
            }),
        );

        let output = handle_web_invoke("req:web", &request, &WebLimits::default()).await;

        assert!(matches!(
            output.outcome,
            EndpointExecutionOutcome::Applied {
                actual_cost_micro: 0,
                ..
            }
        ));

        let sense = output.sense.expect("sense should be emitted");
        assert_eq!(
            sense.neural_signal_descriptor_id,
            "body.std.web.result".to_string()
        );
        assert_eq!(sense.payload["status_code"], serde_json::json!(200));
        assert_eq!(sense.payload["body_text"], serde_json::json!("hello"));
        assert_eq!(sense.payload["body_truncated"], serde_json::json!(true));
    }
}
