# Body Module

Body is Beluna's world-facing endpoint layer.

Current runtime body endpoints:

1. Embedded standard endpoints in core:

- Shell act descriptor: `tool.shell.exec`
- Shell emitted sense descriptor: `body.std.shell.result`
- Web act descriptor: `tool.web.fetch`
- Web emitted sense descriptor: `body.std.web.result`

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

Identifier constraints for Body Endpoints:

1. Stem accepts only ASCII letters, digits, `.`, and `-` in endpoint/ns ids.
2. Dot segmentation must stay valid (`.foo`, `foo.`, `foo..bar` are invalid).
3. Avoid mixed dashed/dotted aliases such as `aa-bb` and `aa.bb` for NS ids in the same endpoint namespace. Reason: act-tool name normalization can collapse those variants into the same tool name and trigger duplicate-tool collisions during tool build.
