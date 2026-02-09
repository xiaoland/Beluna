# Glossary

> Product top-level glossary (cross-feature terms only)

- Agent: A system that iteratively turns intent into environment changes based on feedback.
- One mind: Beluna uses one primary reasoning model as the decision center.
- Helper (sub-agent): Task-scoped temporary assistant process used to reduce cognitive load.
- Framework: Beluna code that routes data/control and connects mind, tools, and environment.
- Body abstraction: Environment-facing adapters for input/output surfaces (shell, filesystem, GUI automation, retrieval, runtime tools).
- Feedback loop: Core operating cycle `Goal -> Action -> Environment change -> Feedback -> Next action`.
- Natural language protocol: Human-facing and agent-to-helper interaction interface expressed in natural language.
- MindState: In-process continuity state owned by Mind across decision cycles.
- GoalManager: Invariant-enforcing controller for goal lifecycle and active-goal ownership.
- Safe point: Snapshot declaring whether current active goal is preemptable, with optional checkpoint token.
- Preemption disposition: Explicit goal-switch decision from the closed set `pause|cancel|continue|merge`.
- Memory policy port: Trait boundary for remember/forget policy decisions without requiring persistent storage in MVP.
