# Decisions - Drift Disposition

## Rule
- `SVCv9` is normative SSoT.
- `SVCv8` is contextual reference only.

## Prefill Status
- This file is prefilled with recommended dispositions as of March 27, 2026.
- Recommendations are ready for approval/edit before executing Phase 1 changes.

## Disposition Table

| Drift Item | Disposition (`align_now` / `align_later` / `keep_intentional_drift`) | Rationale | Owner Docs | Verification Signal | Due Date |
|---|---|---|---|---|---|
| `00-meta` mandatory posture | `align_now` | Keep `00-meta` as a repository-specific governance layer, but remove wording that implies universal mandatory adoption. This aligns with V9 minimal-default philosophy without deleting useful Beluna governance assets. | `docs/00-meta/index.md`, `docs/00-meta/doc-system.md`, `AGENTS.md` | Language no longer states or implies globally mandatory meta layer; text explicitly frames it as Beluna-local operating model. | April 3, 2026 |
| Ritualized read path | `align_now` | V8/V9 both reject universal mandatory read order. Keep ordering as advisory and context-sensitive. | `docs/00-meta/read-order.md`, `docs/00-meta/index.md` | Read-order text uses guidance language (context-dependent), not ritualized must-read sequence. | April 3, 2026 |
| Canonical semantics owner in `00-meta` | `align_now` | V9 explicitly places canonical product/domain semantics in PRD. `00-meta` should retain cross-layer operational terminology only. | `docs/10-prd/index.md`, `docs/10-prd/glossary.md` (if added), `docs/00-meta/concepts.md`, `docs/00-meta/promotion-rules.md` | Promotion targets route canonical product/domain terms to PRD; `concepts.md` scope excludes PRD semantic ownership. | April 6, 2026 |
| Unit TDD all-unit template policy | `align_now` | Hard-unit admission audit completed (`core`, `apple-universal` hard; `cli`, `monitor` lightweight). Policy now adopts hard-unit-first profiles and removes universal full-template requirement. | `docs/30-unit-tdd/index.md`, `docs/20-product-tdd/index.md`, `docs/00-meta/doc-system.md`, `tasks/svcv9-doc-system-drift/L2-PLAN-03-unit-tdd-admission-audit.md` | Hard-unit-first language is consistent across `00/20/30`, and unit catalog profile assignments are explicit. | March 27, 2026 |
| Large mutable snapshots in component AGENTS | `align_now` | V8/V9 both prefer short, practical AGENTS. Move volatile state snapshots out of AGENTS or sharply reduce them to lower staleness risk. | `core/AGENTS.md`, `apple-universal/AGENTS.md`, `AGENTS.md` | Component AGENTS retain durable constraints and boundaries; volatile runtime status sections are removed or explicitly minimized with freshness rule. | April 10, 2026 |
| Missing explicit demotion lifecycle language | `align_now` | V8/V9 both require simplifying/removing stale durable docs; current Beluna docs should codify this lifecycle explicitly. | `docs/00-meta/promotion-rules.md`, `docs/00-meta/doc-system.md` | Demotion/removal rule is present with trigger criteria and ownership for cleanup. | April 3, 2026 |

## Notes
- If choosing `keep_intentional_drift`, include concrete risk prevented and maintenance cost accepted.
- `align_later` must include a concrete date milestone, not only a generic backlog statement.

## Phase-0 Recommendation
- Proceed with `align_now` items first in Phase 1.
- Start Unit TDD admission audit immediately; do not change Unit TDD policy text until audit evidence is recorded.

## Execution Update (March 27, 2026)
- `align_now` implementation started and completed for:
  - `00-meta` posture language.
  - read-path de-ritualization.
  - canonical semantics ownership routing to PRD.
  - AGENTS snapshot slimming and restatement protocol.
  - demotion/removal lifecycle rules.
  - Unit TDD admission policy moved from universal full-template to hard-unit-first profiles.
