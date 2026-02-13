# Body Module

Body is Beluna's world-facing endpoint layer.

Current runtime Body Endpoints:
1. Embedded standard endpoints in core:
- Shell endpoint: `tool.shell.exec` / `cap.std.shell`
- Web endpoint: `tool.web.fetch` / `cap.std.web.fetch`
2. External Apple Universal Body Endpoint:
- Chat reply endpoint: `chat.reply.emit` / `cap.apple.universal.chat`
- Implementation: `/Users/lanzhijiang/Development/Beluna/apple-universal/*`

## Boundary

1. Spine is the boundary between Cortex/Continuity and Body Endpoints.
2. Embedded standard endpoints are part of core runtime composition.
3. External Body Endpoints connect over Spine UnixSocket.

## Runtime Flow

1. Body Endpoint connects to Spine UnixSocket (if external).
2. Body Endpoint sends `body_endpoint_register`.
3. Spine sends `body_endpoint_invoke`.
4. Body Endpoint executes action and returns `body_endpoint_result`.
5. Body Endpoint can emit `sense` for execution observations.

## Safety Controls

1. Shell endpoint executes argv directly (no shell interpolation layer).
2. Shell has timeout and stdout/stderr byte caps.
3. Web endpoint allows `http`/`https` only.
4. Web endpoint has timeout and response byte caps.
