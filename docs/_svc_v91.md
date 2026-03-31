# **Sustainable Vibe Coding Framework v9.1**

## **0\. Purpose**

Sustainable Vibe Coding exists to make AI-assisted software development maintainable for a small team or a one-person company.

The framework is not a document-heavy process system. It is a **selective memory system** for preserving truths that are expensive to rediscover and risky to lose.

Its core job is to help humans and agents answer:

* what the product must be and why  
* what technical truths must remain stable across iterations  
* what local complexities are dangerous enough to deserve explicit design memory  
* what runtime truths matter operationally  
* how to align at the correct level of granularity when natural language alone is not enough  
* what should stay ephemeral in tasks rather than being promoted into durable docs  
* **\[v9.1\] how agents should dynamically navigate ambiguity without falling into rigid waterfall processes or chaotic guesswork**

The framework should remain as small as possible. Every durable document must justify its existence.

## **1\. Core principles**

### **1.1 PRD is the SSoT for product what and why**

The PRD owns product intent: pressures, user-visible claims, workflows, rules, scope, and canonical semantics. It does **not** own implementation structure.

### **1.2 Code, tests, and guardrails are the SSoT for implementation truth**

Implementation truth should live in code, tests, type systems, lint rules, CI checks, and runtime assertions. Do not use prose docs where an executable guardrail can do the job better.

### **1.3 TDD exists only where code alone is not enough**

Technical design docs are not mandatory layers of ceremony. They exist only when the system contains truths that code and tests cannot cheaply preserve or communicate.

### **1.4 Tasks absorb volatility**

Tasks are where exploration, iteration, temporary reasoning, and unstable decisions live. Durable docs should contain only truths that have survived enough change to deserve preservation.

### **1.5 Docs are for expensive unknowns**

A durable doc should exist only when it helps future humans or agents answer questions that are easy to get wrong, expensive to rediscover, cross-cutting, and not better enforced mechanically.

### **1.6 Do not build a second software system out of docs**

Documentation is support structure, not a parallel runtime.

### **1.7 Alignment docs are coordination artifacts, not a new truth layer**

When human-agent drift repeatedly comes from ambiguous references, unstable naming, or mismatched granularity, a small alignment pack may be justified.

### **1.8 Use medium-native address systems**

Different project types need different reference systems (e.g., frontend vs. backend). Do not force one medium's map onto another.

## **2\. Layer model**

V9.1 keeps the minimal durable truth layers of V9.

**2.1 PRD layer:** preserve product what and why.

**2.2 Product TDD layer:** preserve cross-unit technical truth.

**2.3 Unit TDD layer:** preserve the complexity-dissolving design memory of hard local units.

**2.4 Deployment layer:** preserve runtime and operational truths.

**2.5 Task layer:** hold volatile work, exploration, iteration plans, and temporary reasoning.

**2.6 Alignment substrate:** reduce coordination drift.

## **3\. Minimal filesystem**

The default starting point should still be minimal:

/  
├─ AGENTS.md  
├─ docs/  
│  └─ 10-prd/  
└─ tasks/

Expanded form when justified:

/  
├─ AGENTS.md  
├─ docs/  
│  ├─ 10-prd/  
│  ├─ 15-alignment/  
│  ├─ 20-product-tdd/  
│  ├─ 30-unit-tdd/  
│  └─ 40-deployment/  
└─ tasks/

## **4\. AGENTS.md & The Execution Protocol**

AGENTS.md is the local operating guide for humans and coding agents. It should stay short and practical.

It is not a static list of rules; it is the **entry point state machine** that tells the agent how to behave based on the ambiguity of the request.

### **4.1 The Pre-Execution Restatement Rule (The V9 Anchor)**

At minimum, for reference-sensitive or logic-altering changes, the agent must restate the following before writing code:

* target (path or anchor)  
* state/context  
* operation  
* scope (what is included, what is excluded)  
* invariants (what must not break)  
* likely affected files  
* uncertainty

### **4.2 The Dynamic Execution Protocol (The V9.1 Addition)**

Agents often fail by applying a single, rigid workflow to every prompt (e.g., immediately updating PRDs for a vague idea, or skipping straight to code for a complex architectural change).

AGENTS.md must instruct the agent to assess the **volatility/ambiguity** of the human's request and select one of three operating modes:

#### **Mode A: Exploration (High Volatility)**

* **Trigger:** The human provides a vague idea, a fuzzy requirement, or an open-ended problem (e.g., "Let's add a trust score system").  
* **Agent Action:** 1\. **DO NOT** modify PRD, TDD, or production code.  
  2\. Confine all work to the tasks/ directory (e.g., create tasks/explore-feature-x.md).  
  3\. Engage in Q\&A, brainstorm options, and deduce first-principle requirements with the human.

#### **Mode B: Solidification (Transitioning from Chaos to Order)**

* **Trigger:** Mode A concludes, or the human provides a clear but unrecorded set of product/technical rules.  
* **Agent Action:**  
  1. Categorize the new truths: Do they belong in PRD (product logic) or TDD (cross-unit technical contracts)?  
  2. Execute the **Pre-Execution Restatement Rule** to confirm what durable docs will be updated and what the code impact will be.  
  3. Await human confirmation, then update durable docs before coding.

#### **Mode C: Execution (Low Volatility)**

* **Trigger:** The task is highly specific, localized, or is a clear bug fix (e.g., "Fix the race condition in the matching service").  
* **Agent Action:**  
  1. Consult relevant local docs (if any exist).  
  2. Execute the **Pre-Execution Restatement Rule** to lock in scope and invariants.  
  3. Await confirmation, then write tests and code directly.  
  4. Ask the human if the original task document can be deleted/archived.

This dynamic protocol provides a "paved road" for creativity without turning the framework into a rigid waterfall.

## **5\. Alignment pack**

*(Content remains the same as V9 \- used for reducing coordination drift via surface glossaries, UI maps, and operation taxonomies. Only implement when repeated naming/targeting drift occurs).*

## **6\. PRD**

*(Content remains the same as V9 \- SSoT for product truth, claims, workflows, invariants, and canonical domain terms. No technical implementation details).*

## **7\. Product TDD**

*(Content remains the same as V9 \- Cross-unit technical truth, system state authority, cross-unit contracts).*

## **8\. Unit TDD**

*(Content remains the same as V9 \- Complexity-dissolving contracts for genuinely hard local units only).*

## **9\. Deployment docs**

*(Content remains the same as V9 \- Runtime topologies and operational truths).*

## **10\. Tasks**

## **10.1 Role of tasks**

Tasks are the entropy buffer of the system.

In V9.1, the tasks/ directory is explicitly the battleground for **Mode A (Exploration)**. It is the only safe space for an agent to hallucinate, propose temporary decisions, and map out uncertainties without corrupting the durable memory system.

## **10.2 Principle**

Do not try to make task docs permanent. Their job is to absorb volatility so the durable doc system can remain lean.

## **10.3 Promotion rule**

At task completion (moving through Mode B), review what was learned and promote only truths that are stable, reusable, costly to rediscover, and not better enforced mechanically.

## **11\. Promotion rules**

*(Content remains the same as V9 \- Promote only stable truths, demote/delete docs that no longer answer expensive questions).*

## **12\. Anti-patterns**

*(Content remains the same as V9, with one vital addition).*

### **12.13 (V9.1 Addition) Bypassing the Task Layer for New Features**

Agents updating the PRD or TDD based on a single vague prompt without first opening a temporary space in tasks/ to deduce the actual requirements.

## **13\. Reading strategy for agents and humans**

*(Content remains the same as V9 \- Read AGENTS.md first, then alignment/PRD/TDD as needed, driven by the current task).*

## **14\. Migration guidance from V8/V9 to V9.1**

### **14.1 Update AGENTS.md with the Dynamic Protocol**

To migrate to V9.1, immediately update your AGENTS.md to include the **Dynamic Execution Protocol (Mode A, B, C)**. Explicitly instruct your agent to evaluate the ambiguity of your prompts before taking action.

### **14.2 Enforce the Task-First Rule for Agents**

Ensure your agent knows that tasks/ is its mandatory sandbox for Mode A. If it attempts to write to 10-prd/ when you are just brainstorming, correct it by pointing to the Mode A protocol in AGENTS.md.

## **15\. Summary**

V9.1 keeps the V8/V9 idea intact: Sustainable Vibe Coding is a selective memory system, not a document empire.

Its governing idea is now augmented with an execution protocol:

* keep PRD as the SSoT for product what and why  
* keep implementation truth in code and tests  
* use Product TDD and Unit TDD only for complex, multi-unit or hard-local technical truths  
* **\[v9.1\] use the Dynamic Execution Protocol in AGENTS.md to force agents to buffer ambiguity in tasks/ before committing to durable docs or code**  
* promote only what future humans and agents would otherwise struggle to recover

This keeps the system maintainable, agent-friendly, aligned at the right granularity, and dynamically adaptable to the chaos of software creation.