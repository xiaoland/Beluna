use std::sync::Arc;

use async_trait::async_trait;
use beluna::ai_gateway::{
    chat::{
        Chat, ContentPart, ContextControlReason, DeriveContextOptions, Message,
        RewriteContextOptions, SystemPromptAction, ThreadContextRequest, ThreadOptions, Turn,
        TurnRetentionPolicy, UserMessage,
    },
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
async fn derive_context_keeps_selected_turn_ids_in_source_order() {
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
    source
        .append_turn(seed_turn(3, "third").await)
        .await
        .expect("third turn should append");

    let (derived, result) = chat
        .derive_context(
            &source,
            ThreadContextRequest {
                retention: TurnRetentionPolicy::KeepSelectedTurnIds {
                    turn_ids: vec![3, 1],
                },
                system_prompt: SystemPromptAction::Keep,
                drop_unfinished_continuation: false,
                reason: ContextControlReason::Manual,
            },
            DeriveContextOptions::default(),
        )
        .await
        .expect("derive should succeed");

    let derived_turns = derived.turns().await;
    assert_eq!(derived_turns.len(), 2);
    assert_eq!(derived_turns[0].turn_id(), 1);
    assert_eq!(derived_turns[1].turn_id(), 3);
    assert_eq!(result.kept_turn_ids, vec![1, 3]);
    assert_eq!(result.dropped_turn_ids, vec![2]);
    assert!(!result.continuation_dropped);

    derived
        .append_turn(seed_turn(4, "after-derive").await)
        .await
        .expect("append should continue from max(turn_id)+1");
    let derived_after_append = derived.turns().await;
    assert_eq!(derived_after_append[2].turn_id(), 4);
}

#[tokio::test]
async fn rewrite_context_keeps_last_turns_and_preserves_ids() {
    let chat = Chat::new(&gateway_config(), Arc::new(StaticCredentialProvider))
        .expect("chat should build");
    let thread = chat
        .open_thread(ThreadOptions::default())
        .await
        .expect("thread should open");
    thread
        .append_turn(seed_turn(1, "first").await)
        .await
        .expect("first turn should append");
    thread
        .append_turn(seed_turn(2, "second").await)
        .await
        .expect("second turn should append");
    thread
        .append_turn(seed_turn(3, "third").await)
        .await
        .expect("third turn should append");

    let result = chat
        .rewrite_context(
            &thread,
            ThreadContextRequest {
                retention: TurnRetentionPolicy::KeepLastTurns { count: 2 },
                system_prompt: SystemPromptAction::Clear,
                drop_unfinished_continuation: true,
                reason: ContextControlReason::CortexReset,
            },
            RewriteContextOptions::default(),
        )
        .await
        .expect("rewrite should succeed");

    let rewritten_turns = thread.turns().await;
    assert_eq!(rewritten_turns.len(), 2);
    assert_eq!(rewritten_turns[0].turn_id(), 2);
    assert_eq!(rewritten_turns[1].turn_id(), 3);
    assert_eq!(result.kept_turn_ids, vec![2, 3]);
    assert_eq!(result.dropped_turn_ids, vec![1]);
    assert!(result.continuation_dropped);

    thread
        .append_turn(seed_turn(4, "after-rewrite").await)
        .await
        .expect("append should continue from max(turn_id)+1");
    let turns_after_append = thread.turns().await;
    assert_eq!(turns_after_append[2].turn_id(), 4);
}

#[tokio::test]
async fn derive_context_rejects_duplicate_selected_turn_ids() {
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

    let result = chat
        .derive_context(
            &source,
            ThreadContextRequest {
                retention: TurnRetentionPolicy::KeepSelectedTurnIds {
                    turn_ids: vec![1, 1],
                },
                system_prompt: SystemPromptAction::Keep,
                drop_unfinished_continuation: false,
                reason: ContextControlReason::Manual,
            },
            DeriveContextOptions::default(),
        )
        .await;
    match result {
        Ok(_) => panic!("duplicate selected ids should fail"),
        Err(err) => assert!(err.message.contains("duplicate turn_id")),
    }
}
