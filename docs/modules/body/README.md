# Body Module

Body is Beluna's world-facing endpoint layer.

Current runtime body endpoints:
1. Embedded standard endpoints in core:
- Shell act descriptor: `tool.shell.exec`
- Web act descriptor: `tool.web.fetch`
2. External Apple Universal endpoint:
- act descriptor: `present.message.text`
- sense descriptors: `user.message.text`, `present.message.text.success`, `present.message.text.failure`
- implementation: `/Users/lanzhijiang/Development/Beluna/apple-universal/*`

Boundary:
1. Spine is the runtime boundary between cognition runtime and body endpoints.
2. Embedded endpoints are started by core and attached through Spine inline adapter.
3. External endpoints connect via Spine UnixSocket NDJSON.

Runtime flow:
1. Endpoint connects to Spine transport.
2. Endpoint sends `auth` with `endpoint_name` and `ns_descriptors`.
3. Spine sends `act` envelopes.
4. Endpoint sends `act_ack` and may emit correlated senses.
5. Endpoint senses use text payload with `weight` and optional `act_instance_id`.
