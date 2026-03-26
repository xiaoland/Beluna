# Unit Boundary Rules

This file defines light-normative decomposition governance.

## Keep As Internal Module (Inside Existing Unit)

Keep responsibility as an internal module when most conditions hold:

1. Release cadence is tightly coupled to existing unit changes.
2. Runtime isolation provides limited operational benefit.
3. Cross-boundary contract surface would be narrow or unstable.
4. Ownership and cognitive load remain manageable for the current team.

## Split To New Technical Unit

Split responsibility into a new unit when two or more conditions persist:

1. Independent release cadence is repeatedly blocked by current coupling.
2. Strong runtime isolation materially improves safety or operability.
3. Cross-boundary contracts are stable enough to justify explicit coordination cost.
4. Ownership boundaries are clearer with separate unit accountability.

## Keep Multiple Units In One Code Container

Prefer same container when:

1. Contract churn is high and rapid co-change is expected.
2. Build/release workflow benefits from single-repo coordination.
3. Team size and ownership model do not justify extra repo overhead.

## Split Into Separate Code Containers

Split containers when:

1. Release/process isolation provides clear operational benefit.
2. Contract stability is high enough to tolerate asynchronous evolution.
3. Security/compliance/runtime boundary requirements require independent packaging.

## Merge Previously Split Boundaries

Merge units or containers when:

1. Cross-boundary changes dominate routine work.
2. Coordination overhead consistently exceeds isolation benefit.
3. Independent deployment no longer provides meaningful value.

## Default Bias For Beluna

1. Prefer fewer code containers.
2. Prefer stronger internal boundaries before unit/container splits.
3. Split only when operational benefit clearly exceeds coordination cost.
