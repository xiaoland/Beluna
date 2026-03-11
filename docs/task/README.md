# docs/task README

## Complex task workflow

Make progressive planning (ask user's approval before getting into next planning stage):

1. Analyze L0 (User Input) & Context: Deconstruct the request. Explore the codebase for existing patterns, identify architectural trade-offs; and research external sources, ask clarifying questions if necessary.
2. Draft L1 (High-Level Strategy): Define the technical approach. Establish the architectural design, key technical decisions, and dependency requirements. It's suggested to interact with user actively at this stage.
3. Draft L2 (Low-level Design): Iterate on L1 until approved. Then, define the low-level specifics: interfaces, data structures, and algorithms.
4. Draft L3 (Implementation Plan): Iterate on L2 until approved. Then, outline the execution roadmap: pseudo-code, step-by-step logic, boundaries, and test plans.
5. Execute Implementation: Iterate on L3 until approved. Implement the solution strictly following the comprehensive L3 plan.
6. Finalize Documentation: Compile the output and write `docs/task/RESULT.md`.  

Notes:

- Write each stage of plan to file `docs/task/<task-name>/L<X>-PLAN.md`. Spilt into multiple sub-files if needed.
- You are encouraged to doubt user's requests/decisions for better maintainability/readability.
- Avoid preconceived assumptions; should reason from first principles and leverage both inductive and deductive reasoning.

## Simple task workflow

Draft a simple implementation plan (expand key changes) and ask for my approval. If you find out key decisions needed, stop and discuss with me.

## Refactoring task workflow

Refactoring in all these three levels:

- Local Refactoring: Rename, Extract function, Remove duplication, Improve naming -> improves readability.
- Structural Refactoring: Split modules, Introduce boundaries, Change data flow, Replace patterns -> improves maintainability.
- Architectural Reset: Remove features, Break backward compatibility, Redesign abstractions, Rewrite subsystems -> improves long-term velocity.
