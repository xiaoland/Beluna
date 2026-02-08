# Glossary

> Product top-level glossary (cross-feature terms only)

- Agent: A system that iteratively turns intent into environment changes based on feedback.
- One mind: Beluna uses one primary reasoning model as the decision center.
- Helper (sub-agent): Task-scoped temporary assistant process used to reduce cognitive load.
- Framework: Beluna code that routes data/control and connects mind, tools, and environment.
- Body abstraction: Environment-facing adapters for input/output surfaces (shell, filesystem, GUI automation, retrieval, runtime tools).
- Feedback loop: Core operating cycle `Goal -> Action -> Environment change -> Feedback -> Next action`.
- Natural language protocol: Human-facing and agent-to-helper interaction interface expressed in natural language.
