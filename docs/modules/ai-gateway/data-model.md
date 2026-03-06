# Data Model

## Message Layer

Concrete message structs:

- `SystemMessage`
- `UserMessage`
- `AssistantMessage`
- `ToolCallMessage`
- `ToolCallResultMessage`

Wrapped by:

- `Message` enum

## Turn

`Turn` is the atomic unit and contains:

- `turn_id` (thread-local monotonic integer)
- ordered `messages: Vec<Message>`
- metadata/usage/finish information

Integrity rule:

- mandatory completeness only enforces tool-call/result linkage

## Thread

`Thread` contains:

- `thread_id`
- bound backend adapter context
- ordered `turns: Vec<Turn>`

Thread edit pattern for Cortex reset:

- do not mutate existing thread timeline in-place
- pick turns
- deep-copy turns/messages to a new thread
