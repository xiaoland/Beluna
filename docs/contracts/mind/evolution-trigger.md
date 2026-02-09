# Evolution Trigger Contract

## Scope

Defines proposal-only evolution decisions for Mind.

## Contract Cases

1. Given a single isolated failure signal
- When evolution decision runs
- Then output is `no_change`.

2. Given repeated reliability failures over threshold
- When evolution decision runs
- Then output is `change_proposal`.

3. Given repeated signal-faithfulness failures
- When evolution decision runs
- Then proposal target prefers perception pipeline.

4. Given low-confidence failure evidence
- When evolution decision runs
- Then output is `no_change`.

## Proposal-Only Rule

- Mind may emit `ChangeProposal` only.
- Mind does not execute replace/retrain/reconfigure in MVP.
