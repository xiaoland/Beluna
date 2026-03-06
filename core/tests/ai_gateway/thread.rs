use std::sync::Arc;

use async_trait::async_trait;
use beluna::ai_gateway::{
    chat::{Chat, CloneThreadOptions, ContentPart, Message, ThreadOptions, Turn, UserMessage},
    credentials::CredentialProvider,
    error::GatewayError,
    types::{
        AIGatewayConfig, BackendDialect, BackendProfile, ChatConfig, CredentialRef, ModelProfile,
        ResilienceConfig, ResolvedCredential,
    },
};

#[derive(Default)]
struct StaticCredentialProvider;

#[async_trait]
impl CredentialProvider for StaticCredentialProvider {
    async fn resolve(
        &self,
        _reference: &CredentialRef,
        _backend: &BackendProfile,
    ) -> Result<ResolvedCredential, GatewayError> {
        Ok(ResolvedCredential::none())
    }
}

fn gateway_config() -> AIGatewayConfig {
    AIGatewayConfig {
        backends: vec![BackendProfile {
            id: "primary".to_string(),
            dialect: BackendDialect::OpenAiCompatible,
            endpoint: Some("https://example.com/v1".to_string()),
            credential: CredentialRef::None,
            models: vec![ModelProfile {
                id: "m1".to_string(),
                aliases: vec!["default".to_string()],
            }],
            capabilities: None,
            copilot: None,
        }],
        chat: ChatConfig::default(),
        resilience: ResilienceConfig::default(),
    }
}

async fn seed_turn(turn_id: u64, text: &str) -> Turn {
    let mut turn = Turn::new(turn_id);
    turn.append_one(
        Message::User(UserMessage {
            id: format!("user-{turn_id}"),
            created_at_ms: turn_id,
            parts: vec![ContentPart::Text {
                text: text.to_string(),
            }],
        }),
        None,
    )
    .await
    .expect("seed append should succeed");
    turn
}

#[tokio::test]
async fn clone_thread_with_turns_reorders_turns_and_reindexes_ids() {
    let chat = Chat::new(&gateway_config(), Arc::new(StaticCredentialProvider))
        .expect("chat should build");
    let source = chat
        .open_thread(ThreadOptions::default())
        .await
        .expect("thread should open");
    source
        .append_turn(seed_turn(1, "first").await)
        .await
        .expect("first turn should append");
    source
        .append_turn(seed_turn(2, "second").await)
        .await
        .expect("second turn should append");

    let cloned = chat
        .clone_thread_with_turns(&source, &[2, 1], CloneThreadOptions::default())
        .await
        .expect("clone should succeed");
    let turns = cloned.turns().await;

    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].turn_id(), 1);
    assert_eq!(turns[1].turn_id(), 2);
    assert_eq!(
        turns[0].metadata().get("source_turn_id").map(String::as_str),
        Some("2")
    );
    assert_eq!(
        turns[1].metadata().get("source_turn_id").map(String::as_str),
        Some("1")
    );

    let first_message_text = match &turns[0].messages()[0] {
        Message::User(message) => match &message.parts[0] {
            ContentPart::Text { text } => text.as_str(),
            other => panic!("expected text part, got {other:?}"),
        },
        other => panic!("expected user message, got {other:?}"),
    };
    assert_eq!(first_message_text, "second");
}
