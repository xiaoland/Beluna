use std::sync::Arc;

use beluna::ai_gateway::{
    chat::{
        Chat, ChatToolDefinition, FinishReason, OutputMode, ThreadOptions, TurnInput, TurnLimits,
    },
    credentials::EnvCredentialProvider,
    types::{
        AIGatewayConfig, BackendDialect, BackendProfile, ChatConfig, CredentialRef, ModelProfile,
        ResilienceConfig,
    },
};
use serde_json::json;

use crate::kit::{
    chat::{EchoToolExecutor, chat_for_responses_endpoint, text_response, user_message},
    local_http::LocalJsonServer,
};

#[tokio::test]
async fn openai_responses_complete_posts_expected_request() {
    let mut server = LocalJsonServer::start(vec![text_response("ack")]).await;
    let chat = chat_for_responses_endpoint(server.endpoint());
    let thread = chat
        .open_thread(ThreadOptions {
            thread_id: Some("thread-openai-responses".to_string()),
            tools: vec![ChatToolDefinition {
                name: "break-primary-phase".to_string(),
                description: Some("Stop Primary.".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }),
            }],
            system_prompt: Some("You are Cortex Primary.".to_string()),
            default_limits: Some(TurnLimits {
                max_output_tokens: Some(512),
                max_request_time_ms: Some(30_000),
            }),
            ..ThreadOptions::default()
        })
        .await
        .expect("open thread");

    let output = thread
        .complete(TurnInput {
            messages: vec![user_message("Input IR")],
            ..TurnInput::default()
        })
        .await
        .expect("complete");

    assert_eq!(output.response.output_text, "ack");
    let request = server.next_request().await;
    assert_eq!(request.path, "/v1/responses");
    assert_eq!(request.body["model"], "gpt-5");
    assert_eq!(request.body["store"], false);
    assert_eq!(request.body["instructions"], "You are Cortex Primary.");
    assert_eq!(request.body["input"][0]["type"], "message");
    assert_eq!(request.body["input"][0]["role"], "user");
    assert_eq!(request.body["input"][0]["content"], "Input IR");
    assert_eq!(request.body["tools"][0]["type"], "function");
    assert_eq!(request.body["tools"][0]["name"], "break-primary-phase");
    assert_eq!(request.body["tool_choice"], "auto");
    assert_eq!(request.body["parallel_tool_calls"], true);
    assert_eq!(request.body["max_output_tokens"], 512);
}

#[tokio::test]
async fn openai_responses_complete_maps_text_output_usage_and_finish_reason() {
    let mut server = LocalJsonServer::start(vec![json!({
        "status": "completed",
        "output": [{
            "type": "message",
            "role": "assistant",
            "content": [
                { "type": "output_text", "text": "hello" },
                { "type": "output_text", "text": " world" }
            ]
        }],
        "usage": {
            "input_tokens": 10,
            "output_tokens": 4,
            "total_tokens": 14
        }
    })])
    .await;
    let thread = chat_for_responses_endpoint(server.endpoint())
        .open_thread(ThreadOptions::default())
        .await
        .expect("open thread");

    let output = thread
        .complete(TurnInput {
            messages: vec![user_message("Say hello.")],
            ..TurnInput::default()
        })
        .await
        .expect("complete");

    assert_eq!(output.response.output_text, "hello world");
    assert!(matches!(output.response.finish_reason, FinishReason::Stop));
    let usage = output.response.usage.expect("usage");
    assert_eq!(usage.input_tokens, Some(10));
    assert_eq!(usage.output_tokens, Some(4));
    assert_eq!(usage.total_tokens, Some(14));
    let _ = server.next_request().await;
}

#[tokio::test]
async fn openai_responses_complete_maps_tool_call_output() {
    let mut server = LocalJsonServer::start(vec![json!({
        "status": "completed",
        "output": [{
            "type": "function_call",
            "call_id": "call_abc",
            "name": "emit_act",
            "arguments": "{\"payload\":{\"text\":\"hi\"}}"
        }]
    })])
    .await;
    let thread = chat_for_responses_endpoint(server.endpoint())
        .open_thread(ThreadOptions::default())
        .await
        .expect("open thread");

    let output = thread
        .complete(TurnInput {
            messages: vec![user_message("Emit an act.")],
            ..TurnInput::default()
        })
        .await
        .expect("complete");

    assert!(matches!(
        output.response.finish_reason,
        FinishReason::ToolCalls
    ));
    assert_eq!(output.response.tool_calls.len(), 1);
    assert_eq!(output.response.tool_calls[0].id, "call_abc");
    assert_eq!(output.response.tool_calls[0].name, "emit_act");
    assert_eq!(
        output.response.tool_calls[0].arguments_json,
        "{\"payload\":{\"text\":\"hi\"}}"
    );
    let _ = server.next_request().await;
}

#[tokio::test]
async fn openai_responses_complete_maps_json_schema_output_mode() {
    let mut server = LocalJsonServer::start(vec![text_response("{\"items\":[]} ")]).await;
    let thread = chat_for_responses_endpoint(server.endpoint())
        .open_thread(ThreadOptions::default())
        .await
        .expect("open thread");

    let output = thread
        .complete(TurnInput {
            messages: vec![user_message("Extract items.")],
            output_mode: Some(OutputMode::JsonSchema {
                name: "extract_result".to_string(),
                schema: json!({
                    "type": "object",
                    "properties": {
                        "items": { "type": "array" }
                    },
                    "required": ["items"],
                    "additionalProperties": false
                }),
                strict: true,
            }),
            ..TurnInput::default()
        })
        .await
        .expect("complete");

    assert_eq!(output.response.output_text, "{\"items\":[]} ");
    let request = server.next_request().await;
    assert_eq!(request.body["text"]["format"]["type"], "json_schema");
    assert_eq!(request.body["text"]["format"]["name"], "extract_result");
    assert_eq!(request.body["text"]["format"]["strict"], true);
    assert_eq!(
        request.body["text"]["format"]["schema"]["required"][0],
        "items"
    );
}

#[tokio::test]
async fn openai_responses_replays_function_call_output_items() {
    let mut server = LocalJsonServer::start(vec![
        json!({
            "status": "completed",
            "output": [{
                "type": "function_call",
                "call_id": "call_abc",
                "name": "emit_act",
                "arguments": "{\"payload\":{\"text\":\"hi\"}}"
            }]
        }),
        text_response("done"),
    ])
    .await;
    let thread = chat_for_responses_endpoint(server.endpoint())
        .open_thread(ThreadOptions {
            tools: vec![ChatToolDefinition {
                name: "emit_act".to_string(),
                description: Some("Emit an act.".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "payload": { "type": "object" }
                    },
                    "required": ["payload"],
                    "additionalProperties": false
                }),
            }],
            ..ThreadOptions::default()
        })
        .await
        .expect("open thread");

    let first_output = thread
        .complete(TurnInput {
            messages: vec![user_message("Emit an act.")],
            tool_executor: Some(Arc::new(EchoToolExecutor)),
            ..TurnInput::default()
        })
        .await
        .expect("first complete");
    assert!(first_output.response.pending_tool_call_continuation);

    let second_output = thread
        .complete(TurnInput {
            messages: vec![user_message("Continue.")],
            ..TurnInput::default()
        })
        .await
        .expect("second complete");
    assert_eq!(second_output.response.output_text, "done");

    let _first_request = server.next_request().await;
    let second_request = server.next_request().await;
    let input = second_request.body["input"]
        .as_array()
        .expect("input array");
    assert!(input.iter().any(|item| item["type"] == "function_call"));
    let function_output = input
        .iter()
        .find(|item| item["type"] == "function_call_output")
        .expect("function_call_output item");
    assert_eq!(function_output["call_id"], "call_abc");
    assert_eq!(function_output["output"], "{\"ok\":true}");
}

#[tokio::test]
async fn openai_responses_rejects_missing_endpoint() {
    let chat = Chat::new(
        &AIGatewayConfig {
            backends: vec![BackendProfile {
                id: "openai".to_string(),
                dialect: BackendDialect::OpenAiResponses,
                endpoint: None,
                credential: CredentialRef::None,
                models: vec![ModelProfile {
                    id: "gpt-5".to_string(),
                    aliases: vec!["default".to_string()],
                }],
                capabilities: None,
                copilot: None,
            }],
            chat: ChatConfig::default(),
            resilience: ResilienceConfig::default(),
        },
        Arc::new(EnvCredentialProvider),
    )
    .expect("chat");
    let thread = chat
        .open_thread(ThreadOptions::default())
        .await
        .expect("open thread");

    let err = thread
        .complete(TurnInput {
            messages: vec![user_message("Hello.")],
            ..TurnInput::default()
        })
        .await
        .expect_err("missing endpoint should fail");

    assert!(err.message.contains("requires endpoint"));
}
