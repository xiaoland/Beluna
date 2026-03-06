# Execution Flow

## `Thread.complete`

1. `Thread` builds dispatch context from existing turns + current input messages.
2. `ChatRuntime` dispatches to the thread-bound backend under `ResilienceEngine` control.
3. Backend response is normalized to `TurnResponse`.
4. A new `Turn` is created and input/assistant messages are appended.
5. If assistant emits tool calls and a `ToolExecutor` exists:
   - append `ToolCallMessage`
   - run `ToolScheduler`
   - append `ToolCallResultMessage`
6. The turn is finalized and appended to thread history.

## Thread Open / Clone

- `Chat.open_thread*` resolves route/credentials/adapter once and binds backend to the thread.
- `Chat.clone_thread_with_turns` deep-copies selected turns (caller-provided order, free reorder) into a new thread.

## Determinism Guarantees

- No hidden multi-backend fallback.
- No turn-level backend switching.
- Turn tool linkage completeness is validated on append/truncate.
