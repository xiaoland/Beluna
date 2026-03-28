# PRD Glossary

This file is the canonical home for product and domain semantic definitions.

If a term changes product meaning, update this file first and then propagate downstream.

## Product Semantics

- **Product Driver**: market, business, constraint, or operational pressure that shapes product truth.
- **Product Claim**: durable product promise used to evaluate whether Beluna is delivering intended value.
- **Capability**: what Beluna can do to realize product claims.
- **Workflow**: ordered user-visible behavior path across actors, states, and system steps.
- **Product Invariant**: user-visible truth that must remain valid across implementations.

## Domain Semantics

- **Cognition Continuity**: product expectation that intent and cognitive context survive normal runtime cycles and bounded recovery.
- **World Interaction**: product-level exchange between endpoints and core runtime through senses and actions.
- **Operational Trust**: user-visible reliability expectation supported by observability and bounded recovery behavior.

## Boundary Note

Cross-layer operational/governance terminology remains in `docs/00-meta/concepts.md`.
This glossary owns product/domain meaning; meta docs must not redefine these semantics.
