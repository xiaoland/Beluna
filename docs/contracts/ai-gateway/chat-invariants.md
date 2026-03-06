# Chat Invariants Contract

## Boundary

`Turn` and `Thread` enforce structural chat integrity before backend dispatch.

## Scenarios

### Scenario: Tool call append requires a scheduler

- Given: a `Turn`
- Given: one `ToolCallMessage`
- When: `append_one` is called without `ToolScheduler`
- Then: append fails with `InvalidRequest`

### Scenario: Tool call append adds linked result in one operation

- Given: a `Turn`
- Given: one `ToolCallMessage`
- Given: a `ToolScheduler`
- When: `append_one` is called
- Then: the turn contains both the tool call and the matching tool result
- Then: turn validation succeeds

### Scenario: Tool execution failure still closes the tool bundle

- Given: a `Turn`
- Given: one `ToolCallMessage`
- Given: tool execution returns an error
- When: `append_one` is called
- Then: append still succeeds
- Then: the turn contains a structured `ToolCallResultMessage` error payload

### Scenario: Truncate removes whole tool bundle atomically

- Given: a turn whose tail is `ToolCallMessage` + `ToolCallResultMessage`
- When: `truncate_one` is called
- Then: both messages are removed together
- Then: the `ToolCallResultMessage` is not truncated without truncating its matching `ToolCallMessage`
- Then: turn validation still succeeds
