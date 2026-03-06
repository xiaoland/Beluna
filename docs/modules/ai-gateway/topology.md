# AI Gateway — Topology

## Component Topology (Mermaid)

```mermaid
graph TD
    subgraph Callers
        Cortex[Cortex]
        Other[Other Modules]
    end

    subgraph "ai_gateway/chat"
        Chat["Chat (Facade)"]
        Thread["Thread"]
        Turn["Turn"]
        Message["Message Layer"]
        Scheduler["ToolScheduler"]
    end

    subgraph "Runtime"
        Runtime["ChatRuntime"]
        Router["BackendRouter"]
        Creds["CredentialProvider"]
        Resilience["ResilienceEngine"]
        CapGuard["CapabilityGuard"]
    end

    subgraph Adapters
        OpenAI["OpenAI Compatible"]
        Ollama["Ollama"]
        Copilot["GitHub Copilot"]
    end

    Cortex --> Chat
    Other --> Chat
    Chat --> Thread
    Thread --> Turn
    Turn --> Message
    Turn --> Scheduler

    Thread --> Runtime
    Runtime --> Router
    Runtime --> Creds
    Runtime --> Resilience
    Runtime --> CapGuard

    Runtime --> OpenAI
    Runtime --> Ollama
    Runtime --> Copilot
```

## Notes

- `ChatDispatcher` was removed.
- `ThreadStore`/`TurnStore` were removed in this version; thread owns in-memory turns directly.
- Budget enforcement was removed from gateway core.
- Resilience now includes retry/circuit/concurrency/rate controls.
