# Task

## Workflow

Make progressive planning:  

1. Analyze L0 (User Input) & Context: Deconstruct the request. Explore the codebase for existing patterns, identify architectural trade-offs; and research external sources, ask clarifying questions if necessary.
2. Draft L1 (High-Level Strategy): Define the technical approach. Establish the architectural design, key technical decisions, and dependency requirements. It's suggested to interact with user actively at this stage.
3. Draft L2 (Low-level Design): Iterate on L1 until approved. Then, define the low-level specifics: interfaces, data structures, and algorithms.
4. Draft L3 (Implementation Plan): Iterate on L2 until approved. Then, outline the execution roadmap: pseudo-code, step-by-step logic, boundaries, and test plans.
5. Execute Implementation: Iterate on L3 until approved. Implement the solution strictly following the comprehensive L3 plan.
6. Finalize Documentation: Compile the output and write `docs/task/RESULT.md`.  

Notes:

- Write each stage of plan to file `docs/task/<task-name>/L<X>-PLAN.md`. Spilt into multiple files if needed.
- MUST ask user for explicit approval before getting into next planning.
- Make use of sub agent to reduct your cognitive load.
