# Motor Internal Model

> Last Updated: 2026-06-13

## Current Position

`routine` is acceptable vocabulary, but it should stay grounded in Motor's needs.

A routine is a Cortex-authored DSL function that Motor can activate as a
Sense-to-Act reflex.

Routine execution is not merely "expand one Act into more Acts".

A routine can keep control of procedural work by reacting mechanically to
matched Afferent Senses, updating explicit activation state, and emitting
Efferent Acts.

## Routine

Working definition:

```text
routine: DSL transition function selected by Motor from an active routine
registry when an incoming Sense matches the routine selector.
```

Current assumptions:

- routines are written in a DSL.
- Cortex writes routines.
- a routine is function-shaped.
- routine registry is owned by Motor.
- routine definitions are persisted through Continuity.
- routine activation state is owned by Motor and non-persisted for MVP.
- routine lifecycle is affected by built-in Motor Acts.
- an activated routine is driven by Sense input and returns next state plus Act output.
- hidden mutable DSL runtime state is not desired for MVP.

## Routine Input

The minimum routine input is likely:

- one matched `Sense`
- current activation state

Potentially available later, but not assumed for MVP:

- activation id
- routine id
- descriptor metadata for the incoming Sense
- previous child Act lineage
- bounded execution budget
- cancellation / termination marker

Do not add context unless a concrete routine requires it.

## Routine Output

A routine may produce:

- next activation state
- `Vec<Act>` for downstream efferent pipeline stages after Motor.
- `Vec<Sense>` written to the afferent pathway.

The key point:

- Active routine execution is driven by Motor observing matched Senses on the Afferent Pathway.
- Motor stores explicit activation state returned by routines.
- routine-produced Senses are afferent feedback.
- generic accepted/rejected/failure dispatch payloads remain efferent pathway authority.

## Registry

Motor owns the routine registry.

Continuity owns persistence of learned routines.

Communication mode:

- Act.
- Motor must request routine persistence from Continuity by emitting a Continuity-owned Act.
- Motor should not directly call Continuity storage APIs.

Open details:

- how routines are loaded from Cortex-authored create Acts
- whether routine ids map to Neural Signal descriptors at all in MVP
- whether Stem only registers built-in Motor lifecycle descriptors or also routine-produced Sense descriptors
- exact Continuity Act descriptor for persisting routine definitions
- how Motor rehydrates registry from Continuity after startup / restore
- whether DSL validation happens at create time or activation time

## Built-In Lifecycle Signals

Because Cortex writes and manages routines, Motor needs built-in Neural Signals
for routine lifecycle changes.

Working descriptor proposal:

- Act: `motor.create-routine`
- Act: `motor.delete-routine`
- Act: `motor.activate-routine`
- Act: `motor.terminate-routine`

The old `motor.register-routine` shape can be treated as an alias for
`motor.create-routine` only if we want the softer name.

Create flow:

1. Cortex emits `motor.create-routine` with DSL source, routine id, selector metadata, and declared outputs.
2. Motor validates the DSL routine.
3. Motor requests Continuity persistence through an Act.
4. Continuity persists the routine definition.
5. Motor stores the routine definition in its registry.
6. Motor emits success or failure Sense.

Activate flow:

1. Cortex emits `motor.activate-routine` with routine id and activation scope.
2. Motor installs the routine selector into its active routine registry.
3. Matched future Senses can now trigger the routine.

Open detail:

- whether lifecycle result Senses are one generic descriptor or one descriptor per lifecycle Act.
- whether `terminate-routine` should be renamed to `deactivate-routine`.
- whether deleting an active routine implicitly terminates it first.
- whether Continuity persistence must complete before routine activation.
- exact success/failure payload shape.

## DSL Decision

The DSL has not been selected.

See [ROUTINE-DSL.md](./ROUTINE-DSL.md).

Current requirement:

- can express function-shaped routines.
- can be authored by Cortex.
- can take a Sense-shaped input.
- can take and return explicit activation state.
- can emit downstream Acts.
- can emit routine-specific Senses.
- can be called repeatedly by Motor as matched Senses arrive.

## Non-Goals For MVP

- persisted routine activation frames
- learned mutable state across invocations beyond the routine definition itself
- hidden mutable DSL runtime state
- a private Cortex-to-Motor API
- Motor-level generic accepted/rejected sense descriptors
- forcing every routine to become a routine-specific Motor Act descriptor
