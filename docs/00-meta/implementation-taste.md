# Implementation Taste

Implementation taste is the framework-level judgment surface for non-trivial code design and implementation changes.

It is language- and tech-stack-neutral. It is not a style guide, a pattern catalog, or a mandatory design phase.

## Role

Use implementation taste when work will shape or change implementation structure, boundary shape, data shape, state flow, authority flow, durable naming, abstraction, or complexity budget.

Do not load it for purely mechanical edits whose owner, surface, and verification are already obvious.

Implementation taste has two layers:

1. Design formation taste: the primary home of implementation principles. It asks what model, boundary, authority, naming, and complexity tradeoff should exist.
2. Implementation shape taste: the projection of those principles onto concrete code surfaces such as APIs, DTOs, component props, commands, events, state shape, control flow, tests, assertions, and observability.

Implementation shape taste does not own the principles. If applying a principle in code creates repeated friction, the mode engine owns the feedback path.

## Design Formation Taste

Design formation taste is the continuing judgment that shapes implementation across modes. It is a judgment layer, not a phase.

### Preserve SSoT

Every durable fact, state, relationship, or decision should have one authority.

Replicas, caches, views, client state, derived data, and denormalized fields must be treated as references, projections, or performance artifacts unless they are explicitly promoted to authority.

When two surfaces appear to own the same truth, resolve authority before implementation.

### Respect Trust and Provenance

Values crossing a boundary must be classified by provenance:

- authority fact
- stable reference
- command or proposal
- user-authored value
- derived projection

The frontend is not authoritative for server-owned or business-owned facts. Passing an id or command is often better than passing detail because the receiver can resolve authoritative state itself.

User-authored values are the key exception. A user can be authoritative for their own input, expression, preference, or intent, but not for server-owned facts such as permission, price, inventory, eligibility, or existing entity state.

### Name Durable Semantics Directly

Names are part of the semantic model, not just local style.

Durable model fields, cross-boundary fields, commands, events, and business operations should be self-explanatory, direct, and searchable. The same semantic should use the same name unless an explicit boundary translation is being modeled.

Core data model names are usually design formation concerns. DTO names, component prop names, adapter names, and local variable names are implementation shape concerns unless they define a durable contract or reveal a missing design distinction.

### Spend Complexity for Return

Complexity is an input. Useful behavior, reliability, clarity, maintainability, and evolvability are outputs.

Evaluate both total ROI and marginal ROI. Each abstraction, layer, state holder, protocol, configuration switch, indirection, dependency, and design pattern must explain what it earns.

Do not treat OOP, design patterns, generality, or optimization as taste by default. Over-application is a taste failure. Premature optimization and premature abstraction both consume complexity budget before the return is proven.

## Implementation Shape Taste

Implementation shape taste asks whether concrete code surfaces carry the design without distorting it.

Use it to check:

- whether a boundary receives a fact, reference, command, proposal, or user-authored value
- whether code names expose the durable semantic instead of hiding it behind generic containers
- whether local structure matches real complexity instead of flattening stateful logic into accidental branches or wrapping simple flow in machinery
- whether assertions, tests, or observability can prove the intended authority, boundary, and behavior without invasive workarounds
- whether local idioms of the repository are preserved unless the change intentionally renegotiates them

Implementation shape taste should stay close to the code surface being changed. Promote stable lessons only after they pass the normal promotion test.

## Application Path

For non-trivial code design or implementation changes:

1. Load this guidance through the root AGENTS entry point.
2. Use design formation taste to expose authority, trust, naming, and complexity pressure.
3. Use the current route and mode guidance for verification timing and feedback loops.
4. Use implementation shape taste while editing concrete code surfaces.
5. If a verified lesson is stable and expensive to rediscover, promote it to the proper durable owner: Product TDD, Unit TDD, local AGENTS, Deployment, or Meta Engine.
