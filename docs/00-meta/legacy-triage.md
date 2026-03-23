# Legacy Durable Docs Triage

This ledger records the migration outcome for every legacy durable document removed during the hard reset.

This file is archival context only. Historical `New Location` entries may reference paths removed in later restructures.

Outcome meanings:

- `Promote`: moved as-is (or nearly as-is) into the new authoritative system.
- `Rewrite`: concept retained, content rewritten against current implementation and new layer boundaries.
- `Drop`: removed because stale, duplicated, deprecated, over-specific, or task-like.

| Legacy File | Outcome | New Location | Note |
|---|---|---|---|
| `docs/contracts/README.md` | Drop | `docs/20-product-tdd/*; docs/30-unit-tdd/*` | Dedicated contract family removed; stable contracts embedded in TDD layers |
| `docs/contracts/ai-gateway/README.md` | Drop | `docs/20-product-tdd/*; docs/30-unit-tdd/*` | Dedicated contract family removed; stable contracts embedded in TDD layers |
| `docs/contracts/ai-gateway/adapters.md` | Drop | `docs/20-product-tdd/*; docs/30-unit-tdd/*` | Dedicated contract family removed; stable contracts embedded in TDD layers |
| `docs/contracts/ai-gateway/chat-invariants.md` | Drop | `docs/20-product-tdd/*; docs/30-unit-tdd/*` | Dedicated contract family removed; stable contracts embedded in TDD layers |
| `docs/contracts/ai-gateway/gateway-stream.md` | Drop | `docs/20-product-tdd/*; docs/30-unit-tdd/*` | Dedicated contract family removed; stable contracts embedded in TDD layers |
| `docs/contracts/ai-gateway/resilience.md` | Drop | `docs/20-product-tdd/*; docs/30-unit-tdd/*` | Dedicated contract family removed; stable contracts embedded in TDD layers |
| `docs/contracts/ai-gateway/router.md` | Drop | `docs/20-product-tdd/*; docs/30-unit-tdd/*` | Dedicated contract family removed; stable contracts embedded in TDD layers |
| `docs/contracts/ai-gateway/usage.md` | Drop | `docs/20-product-tdd/*; docs/30-unit-tdd/*` | Dedicated contract family removed; stable contracts embedded in TDD layers |
| `docs/contracts/continuity/README.md` | Drop | `docs/20-product-tdd/*; docs/30-unit-tdd/*` | Dedicated contract family removed; stable contracts embedded in TDD layers |
| `docs/contracts/cortex/README.md` | Drop | `docs/20-product-tdd/*; docs/30-unit-tdd/*` | Dedicated contract family removed; stable contracts embedded in TDD layers |
| `docs/contracts/cortex/goal-forest.md` | Drop | `docs/20-product-tdd/*; docs/30-unit-tdd/*` | Dedicated contract family removed; stable contracts embedded in TDD layers |
| `docs/contracts/ledger/README.md` | Drop | `docs/20-product-tdd/*; docs/30-unit-tdd/*` | Dedicated contract family removed; stable contracts embedded in TDD layers |
| `docs/contracts/mind/README.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/contracts/mind/delegation-and-conflict.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/contracts/mind/evaluation.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/contracts/mind/evolution-trigger.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/contracts/mind/facade-loop.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/contracts/mind/goal-management.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/contracts/mind/preemption.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/contracts/spine/README.md` | Drop | `docs/20-product-tdd/*; docs/30-unit-tdd/*` | Dedicated contract family removed; stable contracts embedded in TDD layers |
| `docs/contracts/stem/README.md` | Drop | `docs/20-product-tdd/*; docs/30-unit-tdd/*` | Dedicated contract family removed; stable contracts embedded in TDD layers |
| `docs/descisions/001-observability-opentelemetry.md` | Promote | `-` | ADR family was later hard-deleted; operative conclusions remain in `20/30/40` layers |
| `docs/descisions/002-config-schema-single-source-of-truth.md` | Promote | `-` | ADR family was later hard-deleted; operative conclusions remain in `20/30/40` layers |
| `docs/descisions/README.md` | Rewrite | `-` | ADR index was later removed with `docs/90-decisions` hard deletion |
| `docs/features/README.md` | Drop | `-` | Feature-family documents removed in layered hard reset |
| `docs/features/ai-gateway/HLD.md` | Drop | `-` | Feature-family documents removed in layered hard reset |
| `docs/features/ai-gateway/LLD.md` | Drop | `-` | Feature-family documents removed in layered hard reset |
| `docs/features/ai-gateway/PRD.md` | Rewrite | `docs/10-prd/behavior/workflows.md; docs/30-unit-tdd/core/design.md` | Stable gateway role and workflow kept; feature-local packaging removed |
| `docs/features/continuity/HLD.md` | Drop | `-` | Feature-family documents removed in layered hard reset |
| `docs/features/continuity/LLD.md` | Drop | `-` | Feature-family documents removed in layered hard reset |
| `docs/features/continuity/PRD.md` | Rewrite | `docs/20-product-tdd/coordination.md; docs/30-unit-tdd/core/design.md` | Stable persistence/gate ownership retained |
| `docs/features/cortex/HLD.md` | Drop | `-` | Feature-family documents removed in layered hard reset |
| `docs/features/cortex/LLD.md` | Drop | `-` | Feature-family documents removed in layered hard reset |
| `docs/features/cortex/PRD.md` | Rewrite | `docs/10-prd/behavior/rules-and-invariants.md; docs/20-product-tdd/coordination.md; docs/30-unit-tdd/core/design.md` | Stable cognition-cycle truths retained in unit/system layers |
| `docs/features/ledger/HLD.md` | Drop | `-` | Feature-family documents removed in layered hard reset |
| `docs/features/ledger/LLD.md` | Drop | `-` | Feature-family documents removed in layered hard reset |
| `docs/features/ledger/PRD.md` | Rewrite | `docs/30-unit-tdd/core/design.md; docs/10-prd/behavior/rules-and-invariants.md` | Stable resource-control role retained |
| `docs/features/mind/HLD.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/features/mind/LLD.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/features/mind/PRD.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/features/mind/README.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/features/spine/HLD.md` | Drop | `-` | Feature-family documents removed in layered hard reset |
| `docs/features/spine/LLD.md` | Drop | `-` | Feature-family documents removed in layered hard reset |
| `docs/features/spine/PRD.md` | Rewrite | `docs/20-product-tdd/unit-boundaries.md; docs/30-unit-tdd/core/design.md` | Stable dispatch boundary retained |
| `docs/features/stem/HLD.md` | Drop | `-` | Feature-family documents removed in layered hard reset |
| `docs/features/stem/LLD.md` | Drop | `-` | Feature-family documents removed in layered hard reset |
| `docs/features/stem/PRD.md` | Rewrite | `docs/20-product-tdd/coordination.md; docs/30-unit-tdd/core/design.md` | Stable runtime orchestration boundaries retained |
| `docs/glossary.md` | Rewrite | `docs/00-meta/concepts.md` | Canonical terminology moved to meta layer |
| `docs/modules/README.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/TOPOGRAPHY.md` | Rewrite | `docs/20-product-tdd/system-shape.md; docs/20-product-tdd/coordination.md` | Top-level runtime topology retained in product TDD |
| `docs/modules/ai-gateway/README.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/ai-gateway/adapters.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/ai-gateway/architecture.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/ai-gateway/backend-compatibility.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/ai-gateway/chat-sequence.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/ai-gateway/configuration.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/ai-gateway/data-model.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/ai-gateway/execution-flow.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/ai-gateway/known-limits.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/ai-gateway/policies.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/ai-gateway/purpose.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/ai-gateway/testing-and-contracts.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/ai-gateway/topology.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/body/README.md` | Rewrite | `docs/30-unit-tdd/core/design.md` | Body boundary retained inside core unit design |
| `docs/modules/continuity/README.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/cortex/README.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/cortex/SEQUENCE.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/cortex/TOPOGRAPHY.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/cortex/TOPOLOGY_ANALYSIS.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/ledger/README.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/mind/README.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/modules/mind/architecture.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/modules/mind/execution-flow.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/modules/mind/policies.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/modules/mind/purpose.md` | Drop | `-` | Deprecated domain surface removed from authoritative model |
| `docs/modules/observability/README.md` | Rewrite | `docs/20-product-tdd/operational-constraints.md; docs/40-deployment/observability.md` | Observability ownership/constraints retained |
| `docs/modules/spine/README.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/spine/TOPOGRAPHY.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/stem/README.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/modules/stem/TOPOGRAPHY.md` | Drop | `-` | Module-family documents removed in layered hard reset |
| `docs/overview.md` | Rewrite | `docs/10-prd/behavior/claims.md; docs/10-prd/behavior/workflows.md; docs/20-product-tdd/system-shape.md` | Stable topology and product intent retained in layered form |
