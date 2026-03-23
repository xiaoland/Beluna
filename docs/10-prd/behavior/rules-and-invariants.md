# Rules And Invariants

These are product-level invariants. They must remain true regardless of implementation details.

1. Natural language remains the primary interaction protocol at product boundaries.
2. Every dispatched product action yields an explicit terminal outcome class.
3. Product interactions remain attributable to explicit endpoint and signal identities.
4. Configuration inputs are explicit and validated before runtime behavior is admitted.
5. Runtime observability and recovery remain first-class product commitments, not optional diagnostics.
6. Product truth is not allowed to depend on transient task documents.

Layer purity rule:

- PRD invariants must not prescribe internal runtime mechanism ordering or module ownership.
- Mechanism contracts belong to `20-product-tdd`, `30-unit-tdd`, and `40-deployment`.
