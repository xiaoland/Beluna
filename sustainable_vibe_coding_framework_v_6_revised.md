# Sustainable Vibe Coding Framework v6

## 1. Purpose

Sustainable Vibe Coding is a lightweight, evolvable framework for human + agent software development.

Its purpose is not to maximize documentation. Its purpose is to let a project move quickly without losing:

- alignment
- rationale
- verification quality
- runtime safety
- maintainability under continuous change

A good system should:

- preserve alignment across product intent, technical design, implementation, QA, and runtime
- preserve decision memory without forcing humans or agents to reconstruct the entire project history
- constrain execution through stable anchors and explicit guardrails
- support continuous iteration rather than pretending requirements are static
- remain cheap to maintain by promoting only stable knowledge into durable documents

Documentation should serve software evolution, not become a second software system.

---

## 2. What v6 Corrects

v6 keeps the governance model of v5, but corrects one important upstream mistake:

- **PRD is not domain-driven upstream.**
- **PRD is driven by product pressure: market, business, constraint, and operational reality.**
- **Domain structure is still first-class inside PRD, but it is derived rather than treated as the original source of truth.**

This matters because premature domain crystallization can distort product intent. A project should not begin by assuming domain boundaries are already known. It should begin from the pressures the product must answer, then stabilize behavior, then derive semantic structure that helps the system stay coherent.

v6 therefore changes PRD design, while intentionally preserving the stronger governance surfaces from v5:

- Product TDD remains first-class and explicit
- QA remains first-class
- `tasks/` remains outside `docs/`
- guarded promotion remains explicit
- cross-unit contracts, authority boundaries, and unit-to-container mapping remain named concerns

---

## 3. Core Thesis

Sustainable Vibe Coding does not come from better prompting alone. It comes from a governed system of:

- stable conceptual anchors
- explicit intake of high-entropy input
- pressure-driven product intent
- claim-centered product behavior
- layered technical realization
- explicit verification
- runtime-aware feedback
- deliberate promotion of stable truths

Software work is not a linear pipeline.

Business reality, user wishes, resource constraints, product intent, technical design, implementation discoveries, QA findings, and runtime behavior continuously reshape one another.

The documentation system should therefore be modeled as a **Living Decision Network** with a practical operating protocol layered on top of it.

---

## 4. Working Model

### 4.1 Decision Network, not Waterfall

A convenient working sequence may look like:

`User Input -> Task -> PRD / TDD / Code -> Deployment`

But the real system is not one-way.

The following continuously co-shape one another:

- strategy and business reality
- user wishes and feedback
- resource constraints
- PRD
- Product TDD
- Unit TDD
- implementation discoveries
- QA findings
- deployment and runtime reality

So the framework must support both:

- a stable layered model for durable truth
- an iterative workflow for day-to-day evolution

### 4.2 Constitutional vs Operational Structure

The framework distinguishes between:

1. **Constitutional structure**  
   Stable definitions, document meanings, and governance rules.

2. **Operational structure**  
   How work is initiated, contained, negotiated, verified, and promoted.

These two change at different rates and should not be mixed casually.

### 4.3 External Input is First-Class

In Vibe Coding, input is often not a clean requirement.

It may be:

- a vague wish
- a bug report
- a code fragment
- a stack preference
- a runtime incident
- a cost constraint
- an observed user failure

So the framework must not assume that work starts from an already-clean PRD delta.

Raw input enters the system as a **perturbation** to the Living Decision Network.

---

## 5. Ontology

### 5.1 Product

The product is not the same thing as the software.

A product includes user value, business intent, service reality, market constraints, and operational realities. Software is one realization layer within that product.

### 5.2 Product Driver

A **product driver** is a durable pressure shaping product truth.

Typical drivers include:

- market or user pressure
- business or service objectives
- hard constraints
- operational or runtime realities

Drivers explain why the product should behave a certain way.

### 5.3 Product Claim

A **product claim** is a durable product promise stated in product-observable terms.

A claim should be evaluable. It describes what value, behavior, or reliability the product intends to provide, without collapsing immediately into mechanism.

### 5.4 Domain

A **domain** is a relatively stable business meaning boundary.

A domain usually has its own:

- vocabulary
- actors
- rules and invariants
- lifecycle
- failure modes
- reasons for change

A domain is not defined by screens, API routes, database tables, or repositories.

In v6, domain structure is still important, but it is derived from stabilized product pressures and behavior rather than assumed as the first decomposition step.

### 5.5 Capability

A **capability** is something the product or system can do.

Capabilities may sit within one domain or span several domains.

### 5.6 Workflow

A **workflow** is an ordered behavior path across actors, states, and system steps.

A workflow may be local to one domain or cross-domain.

### 5.7 Technical Unit

A **technical unit** is a technical planning, coordination, and responsibility boundary.

A technical unit is the primary decomposition object in Product TDD.

A unit may be:

- one service
- one app
- one package
- one worker
- one deployable component
- one bounded subsystem inside a monorepo

A unit is defined by coherent responsibility, coordination needs, and technical ownership, not by storage layout.

### 5.8 Code Container

A **code container** is the physical code-management container used to hold one or more technical units.

A code container may be:

- a Git repository
- a monorepo package
- an application folder
- a deployable workspace
- another VCS-managed boundary

A code container is not automatically a technical unit.

The framework must therefore distinguish between:

- **technical decomposition**
- **code storage layout**

### 5.9 TDD

In this framework, TDD means **Technical Design Document**, not test-driven development.

There are two primary layers:

- **Product TDD**: system-level technical realization
- **Unit TDD**: local technical realization for one technical unit

### 5.10 Contract

A contract is a normative statement stable enough to guide implementation and verification.

Contracts may live inside:

- PRD invariants
- Product TDD rules
- Unit TDD rules
- interface specifications
- schemas
- tests
- deployment checks

A contract is not a top-level document family by default, but important cross-unit contracts should be explicitly documented at Product TDD level.

### 5.11 Guardrail

A guardrail is an enforcement mechanism for a contract.

Examples:

- automated tests
- schema validation
- lint rules
- CI checks
- smoke checks
- migration checks
- rollout policies

### 5.12 Acceptance Criteria

Acceptance criteria are change-scoped verification targets.

They are often derived from PRD and TDD. Some remain task-local. Some recurring stable ones are promoted into contracts.

### 5.13 Perturbation

A **perturbation** is any incoming signal that may force the network to evolve.

Examples:

- user wish
- user complaint
- implementation discovery
- runtime failure
- product metric anomaly
- external platform constraint
- code artifact or draft document

A perturbation is not yet durable truth.

### 5.14 Task Packet

A **task packet** is the temporary containment vessel for a perturbation.

It is where:

- raw input is captured
- impact is hypothesized
- assumptions are recorded
- conflicts are negotiated
- execution is tracked
- verification is evaluated
- promotion candidates are extracted

Tasks are the entropy buffer of the system.

---

## 6. What the System Must Prevent

This framework exists to reduce the most common failure modes in human + agent software development:

- concept drift
- naming drift
- boundary drift
- workflow drift
- rationale loss
- ops drift
- verification drift
- premature crystallization
- agent overreach
- decomposition drift
- container drift

**Decomposition drift** means unstable or implicit technical-unit boundaries.  
**Container drift** means silently treating Git repo layout as if it were the real architecture.

v6 adds one more named risk:

- **upstream domain drift**: treating derived semantic structure as if it were the original pressure source for product truth

---

## 7. System State Model

### 7.1 Stable State

`docs/` and code-adjacent guardrails hold the current crystallized state of the network.

This includes:

- product truths
- technical truths
- deployment truths
- stable terminology
- stable contracts
- enforcement mechanisms

### 7.2 Transient State

`tasks/` holds transient working state.

This includes:

- volatile user wishes
- unresolved tradeoffs
- temporary assumptions
- impact hypotheses
- local experiments
- evidence gathered during execution
- promotion candidates

### 7.3 External Events

The network is perturbed by events such as:

- user requests
- bug reports
- code discoveries
- runtime incidents
- metrics changes
- platform constraints

### 7.4 Promotion

Promotion is the act of committing a stable truth from transient task state into durable system state.

### 7.5 Guarded Transition

A state transition is valid only when:

- the affected anchors are identified
- conflicts are resolved or consciously deferred
- verification criteria are satisfied
- only stable truths are promoted

---

## 8. Minimal Durable Document Families

```text
/
├─ AGENTS.md
├─ docs/
│  ├─ 00-meta/
│  ├─ 10-prd/
│  ├─ 20-product-tdd/
│  ├─ 30-unit-tdd/
│  └─ 40-deployment/
└─ tasks/
```

This is the default. Everything else should be justified by real project pressure.

### 8.1 `docs/00-meta/`

Purpose: define how to read and maintain the system itself.

Recommended files:

- `concepts.md`
- `doc-system.md`
- `read-order.md`
- `intake-protocol.md`

This folder is the constitutional layer.

### 8.2 `docs/10-prd/`

Purpose: product intent.

In v6, PRD is pressure-driven, behavior-centered, and semantically stabilized by derived domain structure.

This layer answers:

- what pressures are we answering, and why?
- what product claims and workflows must hold?
- what user-visible rules and invariants must hold?
- what domain structure helps keep the product intelligible after behavior is stabilized?

### 8.3 `docs/20-product-tdd/`

Purpose: system-level technical realization.

Product TDD is not merely architecture in the abstract. Its core job is to define:

- technical decomposition
- realization of product claims and workflows
- cross-unit coordination
- system-wide technical constraints
- mapping from technical units to code containers

### 8.4 `docs/30-unit-tdd/`

Purpose: local technical realization per technical unit.

### 8.5 `docs/40-deployment/`

Purpose: runtime truth.

### 8.6 `AGENTS.md`

Purpose: agent governance.

This defines how agents should operate inside the project. It is not product truth and not technical architecture.

### 8.7 `tasks/`

Purpose: ephemeral execution and negotiation packets.

Tasks are not durable truth by default. Only stable learning discovered during task work should be promoted upward.

---

## 9. Intake Model

### 9.1 Typed Input Taxonomy

Incoming perturbations should be classified before action.

#### Intent Input

Examples:

- new feature request
- UX wish
- policy change
- product behavior request

Usually pressures:

- PRD first
- then Product TDD or Unit TDD

#### Constraint Input

Examples:

- budget limitation
- platform limitation
- stack preference
- performance ceiling
- team or resource limit

Usually pressures:

- Product TDD
- Unit TDD
- Deployment
- sometimes PRD if product behavior must change

#### Reality Input

Examples:

- bug report
- runtime incident
- user complaint
- metric anomaly
- unexpected operational cost

Usually pressures:

- code and verification first
- then back-propagation into PRD, TDD, or deployment if stable truths are exposed

#### Artifact Input

Examples:

- code snippet
- schema
- log
- screenshot
- draft document
- interface proposal

Usually pressures:

- interpretation first
- then classification into one of the categories above

### 9.2 Software is Also Input

Code, logs, tests, incidents, and deployment behavior are not merely outputs. They are also inputs.

A code discovery can force:

- PRD simplification
- TDD adjustment
- deployment rule
- new guardrail

So the framework must treat software reality as an active signal, not passive residue.

---

## 10. Network Intake Protocol

### Step 1: Capture the Perturbation

Record the raw input without prematurely promoting it into durable truth.

Identify:

- raw request or signal
- input type
- observed symptom or desired outcome
- any hard constraints already known

### Step 2: Localize Impact

Do not assume certainty too early. Create an impact hypothesis.

Record:

- **Primary hit**
- **Likely secondary hits**
- **Confidence level**
- **Unknowns**

Typical primary hits:

- product pressure, claim, or user-visible rule -> `10-prd/`
- cross-unit decomposition or coordination -> `20-product-tdd/`
- local algorithm or stack choice -> `30-unit-tdd/`
- runtime constraint -> `40-deployment/`
- broken executable behavior -> code + verification, then upward propagation if needed

### Step 3: Contain in a Task Packet

Do not edit durable docs too early.

Create or update a task packet in `tasks/`.

At minimum it should record:

- governing anchors
- raw perturbation
- intended change
- impact hypothesis
- temporary assumptions
- impact radius
- negotiation triggers

### Step 4: Resolve Conflicts Only When Needed

Escalate to explicit negotiation when the perturbation:

- conflicts with existing durable anchors
- introduces irreversible tradeoffs
- forces upstream truth changes
- breaks important invariants
- changes user-visible behavior materially
- has multiple plausible realizations with different consequences

Otherwise:

- make reversible assumptions
- record them in the task packet
- proceed under guardrails

### Step 5: Execute Under Task Governance

Execute code, design, or detailed logic within the task packet context.

The task should define:

- what is being changed
- what acceptance criteria apply
- what guardrails apply
- what evidence is expected

### Step 6: Verify

Check:

- intent correctness
- design consistency
- behavior correctness
- operational safety
- evidence quality

### Step 7: Decide Outcome

Allowed outcomes:

- **promote**
- **complete without promotion**
- **defer**
- **reject**
- **keep as experiment**

### Step 8: Promote Stable Truth

Only after execution and verification should stable truths be promoted.

---

## 11. PRD Design

### 11.1 Role of PRD

In agent-centric development, PRD must do more than communicate direction.

It must:

- define product pressures
- define durable product claims
- define user-visible behavior
- define invariants and rules
- define scope boundaries
- define the durable product truths that technical realization must honor

PRD should function as a product intent map.

### 11.2 PRD Starts from Pressure, not Pre-Selected Domains

A v6 PRD should begin from the pressures the product must answer.

Typical pressure families:

- market and user pressure
- business and service objectives
- hard constraints
- operational and runtime realities

These are upstream because they explain why the product exists, what tradeoffs shape it, and what realities it must survive.

### 11.3 Behavior is the Center of PRD

Once pressures are clear, PRD should stabilize the product in behavior terms.

The behavioral center of PRD includes:

- claims
- capabilities
- workflows
- rules and invariants
- scope boundaries

This is where product truth becomes operationally meaningful without prematurely falling into architecture.

### 11.4 Domain Structure is Derived but Still First-Class

Domain structure is not discarded in v6. It remains important.

But it is treated as a **derived semantic structure** that helps:

- stabilize vocabulary
- localize lifecycle and policy clusters
- clarify ownership of product meaning
- make cross-domain workflows intelligible
- reduce product discussion drift

Domain structure should therefore be explicit, but it should not silently introduce upstream obligations that are not justified by product pressure and behavior.

### 11.5 Claim-Centered Evaluation

Each major claim should embed lightweight evaluability.

A major claim should state at least:

- claim intent
- why the claim matters
- evaluation dimensions
- evidence expectation
- realization pointers into `20/30/40`

Default policy:

- do not force hard numeric gates unless the project actually needs them
- do make expected evidence visible enough that future work is not blind

### 11.6 PRD Layer Purity

PRD must not govern:

- internal mechanism ordering
- module ownership topology
- wire transport internals
- local technical contracts
- unit-to-container mapping

Those belong to Product TDD, Unit TDD, and deployment docs.

### 11.7 Recommended PRD Shape

```text
10-prd/
├─ index.md
├─ _drivers/
│  ├─ market-and-user-pressures.md
│  ├─ business-and-service-objectives.md
│  ├─ hard-constraints.md
│  └─ operational-realities.md
├─ behavior/
│  ├─ claims.md
│  ├─ capabilities.md
│  ├─ workflows.md
│  ├─ rules-and-invariants.md
│  └─ scope.md
└─ domain-structure/
   ├─ derived-boundaries.md
   ├─ vocabulary-and-lifecycle.md
   └─ cross-domain-interactions.md
```

### 11.8 PRD Layer Rule

The intended read logic is:

- `_drivers` is upstream
- `behavior` is the PRD center
- `domain-structure` is derived and cannot silently redefine upstream truth

---

## 12. Product TDD Design

### 12.1 Product TDD is About System Composition

Its central purpose is to define the technical realization of the product as a system composed of technical units.

It must answer:

1. What technical units exist?
2. Why is the decomposition shaped this way?
3. Which concerns stay internal to a unit and which become cross-unit coordination?
4. How do units coordinate to produce product behavior?
5. What system-level constraints must Unit TDDs inherit?
6. How are product claims, workflows, and rules realized across units?
7. How do technical units map to code containers?

### 12.2 Product TDD Owns Decomposition Policy

Product TDD must not merely describe the current split. It must also define the rules that govern future decomposition.

It should explicitly answer:

- when should a responsibility remain an internal module?
- when should it become a separate technical unit?
- when should multiple units remain in one code container?
- when is a separate code container justified?
- what independence dimensions matter most: release, cadence, cognition, contract stability, runtime isolation, operational ownership?

Default bias for a one-person company:

- prefer fewer code containers
- prefer stronger internal boundaries
- split only when operational benefit clearly exceeds coordination cost

### 12.3 Product Realization Trace

Product TDD should contain an explicit bridge from PRD truths to technical realization.

For each important product rule, workflow, or claim, Product TDD should be able to answer:

- which product driver or claim is being realized?
- which technical unit owns authoritative state?
- which units participate in realizing the behavior?
- what coordination pattern is used?
- what contract or interface carries the behavior across boundaries?
- what failure modes matter?
- where is verification expected to live?

This is more than architecture description. It is requirement-to-mechanism traceability.

### 12.4 Cross-Unit Contracts are First-Class at Product TDD Level

Product TDD should explicitly document important cross-unit contracts such as:

- APIs
- events
- schemas
- authority boundaries
- consistency expectations
- compatibility rules
- migration choreography
- failure semantics

These are system-level truths, not merely local implementation details.

### 12.5 Recommended Product TDD Shape

```text
20-product-tdd/
├─ index.md
├─ system-objective.md
├─ unit-topology.md
├─ unit-boundary-rules.md
├─ unit-to-container-mapping.md
├─ coordination-model.md
├─ cross-unit-contracts.md
├─ system-state-and-authority.md
├─ claim-realization-matrix.md
├─ failure-and-recovery-model.md
└─ deployment-shaping-constraints.md
```

### 12.6 What Each Core File Means

- `unit-topology.md`  
  What technical units exist and how they relate.

- `unit-boundary-rules.md`  
  Normative decomposition policy: when to split, when to keep together, and why.

- `unit-to-container-mapping.md`  
  How technical units map to repositories, packages, apps, or deployables.

- `coordination-model.md`  
  How cross-unit behavior is orchestrated.

- `cross-unit-contracts.md`  
  Stable interfaces and expectations between units.

- `system-state-and-authority.md`  
  Which unit is authoritative for what state and decisions.

- `claim-realization-matrix.md`  
  Trace from PRD truth to technical realization.

---

## 13. Unit TDD Design

Unit TDD should inherit from Product TDD rather than redefine system boundaries.

Each unit should answer:

- what responsibility does this unit own?
- what does it consume and produce?
- what are its dependencies?
- what local assumptions matter?
- what interfaces are exposed or relied upon?
- what local operational rules matter?
- what local verification rules and guardrails matter?

Suggested structure:

```text
30-unit-tdd/<unit>/
├─ README.md
├─ design.md
├─ interfaces.md
├─ data-and-state.md
├─ operations.md
└─ verification.md
```

Unit TDD should not silently change:

- unit boundaries
- authority boundaries
- cross-unit coordination rules
- unit-to-container mapping

Those belong upstream in Product TDD.

---

## 14. QA and Verification

### 14.1 QA is First-Class

QA is not a final stage after development. It is a cross-cutting assurance function.

QA asks at least four questions:

1. **Intent QA** — did we implement the right thing?
2. **Design QA** — is the change still consistent with PRD and TDD?
3. **Behavior QA** — does the software satisfy acceptance criteria and stable contracts?
4. **Operational QA** — can the system be deployed, observed, rolled back, and maintained safely?

### 14.2 Task Verification Packet

Every non-trivial task should carry a lightweight verification packet.

Suggested task fields:

```md
## Perturbation
Raw user or reality signal.

## Input Type
Intent / Constraint / Reality / Artifact.

## Governing Anchors
Stable docs this task depends on.

## Intended Change
What is being changed.

## Impact Hypothesis
Primary hit, likely secondary hits, confidence, unknowns.

## Temporary Assumptions
Reversible assumptions made to proceed.

## Negotiation Triggers
What would require user decision before continuing.

## Acceptance Criteria
What must be true for this task to count as complete.

## Guardrails Touched
Tests, schemas, CI checks, rollout checks, or monitoring this work relies on or changes.

## Evidence Expected
What proof is expected before closing the task.

## Outcome
Promote / complete without promotion / defer / reject / experiment.

## Promotion Candidates
What stable truths, if any, should be promoted back into durable docs or code guardrails.
```

---

## 15. Promotion Rules

Use the following rule of thumb:

- **User-visible truth across domains** -> PRD
- **System-wide technical realization rule** -> Product TDD
- **Unit-local technical rule** -> Unit TDD
- **Runtime or rollout truth** -> Deployment docs
- **Mechanically checkable invariant** -> guardrail near code or CI
- **One-off execution detail** -> keep in task only

### 15.1 When Acceptance Criteria Become Contracts

An acceptance criterion should be promoted into a contract when it is:

- recurring
- stable across tasks
- important enough to guide future work
- costly or risky to keep rediscovering

### 15.2 When Contracts Need Guardrails

A contract should gain a guardrail when:

- it is safety-critical
- it is frequently violated
- it is cheap enough to check mechanically
- human review alone is proving unreliable

### 15.3 What Must Not Drift Downward

The following truths should not be left implicit inside code or local task history:

- product drivers that materially shape behavior
- claim semantics and major workflows
- domain boundaries once stabilized
- unit boundaries
- authority boundaries
- cross-unit contracts
- unit-to-container mapping rationale
- decomposition rules

If these repeatedly matter, they belong in durable docs.

---

## 16. Deployment Boundary

Deployment has its own runtime truth layer.

This framework distinguishes between:

- **deployment-shaping constraints in Product TDD**
- **deployment docs in `40-deployment/`**

This prevents runtime truth from being blurred across layers.

---

## 17. Agent Governance and Read Order

### 17.1 Distributed `AGENTS.md`

Agent governance should be hierarchical.

#### Root `AGENTS.md`

The root file should define:

- authoritative doc map
- terminology rules
- workflow expectations
- change classification rules
- when docs must be updated
- what counts as stable knowledge
- boundaries on agent autonomy

#### Local `AGENTS.md`

Local `AGENTS.md` files should exist only where needed.

The nearest relevant `AGENTS.md` should refine the global one, not replace it.

### 17.2 Read Order

To reduce drift and overreach, humans and agents should default to the following read order:

1. nearest relevant `AGENTS.md`
2. `docs/00-meta/concepts.md`
3. `docs/00-meta/doc-system.md`
4. `docs/00-meta/intake-protocol.md`
5. relevant `10-prd/_drivers/*`
6. relevant `10-prd/behavior/*`
7. relevant `10-prd/domain-structure/*`
8. relevant Product TDD docs
9. relevant Unit TDD docs
10. relevant deployment docs
11. relevant task file
12. code and tests

---

## 18. Recommended Initial Scaffold

```text
/
├─ AGENTS.md
├─ docs/
│  ├─ 00-meta/
│  │  ├─ concepts.md
│  │  ├─ doc-system.md
│  │  ├─ read-order.md
│  │  └─ intake-protocol.md
│  ├─ 10-prd/
│  │  ├─ index.md
│  │  ├─ _drivers/
│  │  │  ├─ market-and-user-pressures.md
│  │  │  ├─ business-and-service-objectives.md
│  │  │  ├─ hard-constraints.md
│  │  │  └─ operational-realities.md
│  │  ├─ behavior/
│  │  │  ├─ claims.md
│  │  │  ├─ capabilities.md
│  │  │  ├─ workflows.md
│  │  │  ├─ rules-and-invariants.md
│  │  │  └─ scope.md
│  │  └─ domain-structure/
│  │     ├─ derived-boundaries.md
│  │     ├─ vocabulary-and-lifecycle.md
│  │     └─ cross-domain-interactions.md
│  ├─ 20-product-tdd/
│  │  ├─ index.md
│  │  ├─ system-objective.md
│  │  ├─ unit-topology.md
│  │  ├─ unit-boundary-rules.md
│  │  ├─ unit-to-container-mapping.md
│  │  ├─ coordination-model.md
│  │  ├─ cross-unit-contracts.md
│  │  ├─ system-state-and-authority.md
│  │  ├─ claim-realization-matrix.md
│  │  ├─ failure-and-recovery-model.md
│  │  └─ deployment-shaping-constraints.md
│  ├─ 30-unit-tdd/
│  │  ├─ index.md
│  │  └─ <unit>/
│  │     ├─ README.md
│  │     ├─ design.md
│  │     ├─ interfaces.md
│  │     ├─ data-and-state.md
│  │     ├─ operations.md
│  │     └─ verification.md
│  └─ 40-deployment/
│     ├─ environments.md
│     ├─ rollout.md
│     ├─ observability.md
│     └─ recovery.md
└─ tasks/
   └─ _template.md
```

---

## 19. Initialization Workflow

### Step 1: Establish Meta Truth

Create:

- `docs/00-meta/concepts.md`
- `docs/00-meta/doc-system.md`
- `docs/00-meta/read-order.md`
- `docs/00-meta/intake-protocol.md`
- root `AGENTS.md`

### Step 2: Capture Product Pressure and Behavior

Create `docs/10-prd/`.

Write only what is stable enough to orient the system:

- upstream product drivers
- major claims
- major workflows
- major rules and invariants
- initial domain structure where it has already become clear

Do not force full domain decomposition too early.

### Step 3: Capture System Composition

Create `docs/20-product-tdd/`.

Document:

- system objective
- unit topology
- unit boundary rules
- unit-to-container mapping
- coordination model
- cross-unit contracts
- authority model
- claim realization trace
- failure and recovery model
- deployment-shaping constraints

### Step 4: Define Units

Create `docs/30-unit-tdd/index.md` and the first set of unit folders.

### Step 5: Capture Runtime Reality

Create `docs/40-deployment/`.

### Step 6: Start Task-Based Work

Use `tasks/` for implementation work.

A task should always point back to the stable anchors it depends on.

---

## 20. Daily Operating Workflow

### 20.1 Classify the Perturbation

Determine whether the incoming signal is primarily:

- intent input
- constraint input
- reality input
- artifact input

### 20.2 Contain Before Promotion

Before implementation, contain the perturbation in a task.

Then update the smallest durable truth that should govern the change.

Examples:

- ambiguous term -> `00-meta/concepts.md`
- intake or governance rule -> `00-meta/intake-protocol.md`
- product pressure, claim, or invariant -> `10-prd/`
- derived domain structure -> `10-prd/domain-structure/`
- cross-unit boundary or coordination rule -> `20-product-tdd/`
- local technical design -> `30-unit-tdd/<unit>/`
- runtime constraint -> `40-deployment/`

### 20.3 Execute the Change

Create or update a task file if needed, then implement.

### 20.4 Verify Through the Task Packet

Check:

- intent
- design consistency
- behavior
- operational readiness
- evidence quality

### 20.5 Decide Outcome

Choose explicitly:

- promote
- complete without promotion
- defer
- reject
- experiment

### 20.6 Propagate Stable Learning Back

Promote only stable new truth.

Do not document every temporary path.

---

## 21. Migration Workflow for an Existing Project

When retrofitting this framework into an existing codebase, avoid trying to document everything at once.

### Phase 1: Map Existing Reality

Identify:

- major product intent already visible in code or docs
- major pressure sources already shaping the product
- major claims and workflows already visible
- domain structure already stabilized in practice
- major technical units already present
- current code containers
- major coordination paths already present
- deployment and runtime realities already constraining the project
- repeated misunderstandings between human and agent
- the major perturbation types the project receives most often

### Phase 2: Create Minimal Anchors

Start with:

- `00-meta/concepts.md`
- `00-meta/doc-system.md`
- `00-meta/intake-protocol.md`
- root `AGENTS.md`
- a minimal `10-prd/` split using `_drivers`, `behavior`, and only the necessary `domain-structure`
- a minimal `20-product-tdd/` split
- one or two highest-value unit docs
- `tasks/_template.md`

### Phase 3: Document Pressure Points

Promote documentation where repeated pressure exists.

Prioritize:

- unstable terminology
- unstable or missing claims
- unclear domain structure after behavior is already known
- unclear unit boundaries
- unclear authority boundaries
- repeated cross-unit misunderstandings
- runtime surprises
- recurring verification failures

### Phase 4: Let the System Grow by Need

Only split files or add folders when the current structure becomes hard to read, hard to update, or repeatedly misused.

---

## 22. Split Rules

A file should be split only when at least one of the following becomes true:

- it is too large to be read as one coherent unit
- different sections change at different rates
- different contributors or agents repeatedly touch unrelated parts
- one section is repeatedly needed without the rest
- misunderstandings occur because the current file mixes abstraction levels

Do not split solely because a category exists in theory.

---

## 23. Anti-Patterns

Avoid the following:

- treating chat history as source of truth
- treating every user wish as immediately promotable truth
- editing durable docs before containing volatility in a task
- creating many document families before the project needs them
- mixing constitutional rules and daily workflow without acknowledging the difference
- mixing PRD and TDD in the same conceptual layer
- treating a Git repository as automatically equal to a technical unit
- treating derived domain structure as upstream requirement source
- leaving unit-to-container mapping implicit
- leaving claim realization implicit
- leaving domain boundaries implicit once stabilized
- leaving unit decomposition implicit
- leaving coordination implicit
- leaving cross-unit contracts implicit
- leaving verification implicit
- promoting every task decision into durable documentation
- putting agent operating rules inside product or technical design docs
- letting deployment reality remain undocumented while pretending TDD is sufficient

---

## 24. Success Criteria for the Framework

The framework is working when:

- a new agent can understand the project shape quickly
- repeated misunderstandings decrease over time
- high-entropy input has a predictable containment path
- PRD starts from real pressures rather than premature decomposition guesses
- claims are explicit enough to evaluate
- domain structure is stable enough to guide product discussion without hijacking upstream truth
- technical unit boundaries are stable enough to guide technical work
- code-container layout does not silently distort technical design
- claim-to-unit realization is explicit enough to prevent drift
- cross-unit contracts are explicit enough to prevent rediscovery from code
- code changes have a clear governing anchor
- verification has a repeatable home instead of depending on memory
- runtime incidents reveal missing truths that can be fed back into the system
- documents remain small enough to update without resistance
- product, design, implementation, QA, and deployment stay meaningfully aligned

The framework is failing when:

- documents are ignored because they are too big or too vague
- people rely on memory or chat history instead of stable anchors
- the same misunderstandings recur without being captured
- volatile wishes are promoted too early
- deployment repeatedly surprises development
- PRD keeps collapsing into architecture
- derived domain structure keeps being mistaken for upstream truth
- domain or unit boundaries remain implicit or unstable
- repo layout keeps being used as a substitute for architecture thinking
- cross-unit behavior is repeatedly rediscovered from code
- work is declared done without clear evidence
- agent work remains high-speed but low-trust

---

## 25. Final Principle

A living documentation system should behave like a well-designed runtime:

- small stable interfaces
- explicit boundaries
- selective persistence
- continuous feedback
- minimal unnecessary state
- controlled intake of entropy

Sustainable Vibe Coding is achieved when a project can absorb vague wishes, software discoveries, and operational reality without collapsing into drift — because volatile exploration is contained in tasks, stable truths are promoted deliberately, PRD is grounded in real product pressure, and both humans and agents can act with confidence.

