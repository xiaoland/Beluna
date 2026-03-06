# AI Gateway — Topology

## Component Topology (Mermaid)

```mermaid
graph TD
    subgraph Callers["Callers"]
        Cortex[Cortex]
        Other[Other Modules]
    end

    subgraph AIGatewayChat["ai_gateway/chat"]
        Chat["Chat (Facade)"]
        Thread["Thread"]
        Turn["Turn"]
        Message["Message Layer"]
        Scheduler["ToolScheduler"]
    end

    subgraph RuntimeGroup["Runtime"]
        ChatRuntime["ChatRuntime"]
        Router["BackendRouter"]
        Creds["CredentialProvider"]
        Resilience["ResilienceEngine"]
        CapGuard["CapabilityGuard"]
    end

    subgraph Adapters["Adapters"]
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

    Thread --> ChatRuntime
    ChatRuntime --> Router
    ChatRuntime --> Creds
    ChatRuntime --> Resilience
    ChatRuntime --> CapGuard

    ChatRuntime --> OpenAI
    ChatRuntime --> Ollama
    ChatRuntime --> Copilot
```

## Notes

- `Thread` owns in-memory turns directly.
- Gateway budget rejection is removed; usage is returned to the caller.
- `ResilienceEngine` owns retry/circuit/concurrency/rate controls.
