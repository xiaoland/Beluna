# Mind Purpose

Mind is Beluna's meta-control boundary.

It is responsible for:

- managing layered goals with one active goal at a time,
- running normative evaluation over alignment, reliability, and faithfulness,
- coordinating delegation through helper ports,
- resolving owned conflicts deterministically,
- emitting proposal-only evolution decisions.

Mind is not:

- a transport/runtime layer,
- a socket protocol handler,
- a persistent memory store,
- a direct executor of evolution actions in MVP.
