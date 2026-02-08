# Adapter Preconditions Contract

## Boundary

Each adapter validates required backend profile fields before transport work.

## Scenarios

### Scenario: OpenAI-compatible adapter requires endpoint

- Given: `OpenAiCompatibleAdapter`
- Given: backend profile has `endpoint = null`
- When: `invoke_stream` is called
- Then: invocation fails with `InvalidRequest`

### Scenario: Ollama adapter requires endpoint

- Given: `OllamaAdapter`
- Given: backend profile has `endpoint = null`
- When: `invoke_stream` is called
- Then: invocation fails with `InvalidRequest`

### Scenario: GitHub Copilot adapter requires Copilot config

- Given: `GitHubCopilotAdapter`
- Given: backend profile has `copilot = null`
- When: `invoke_stream` is called
- Then: invocation setup succeeds
- Then: first stream item is an `InvalidRequest` error
