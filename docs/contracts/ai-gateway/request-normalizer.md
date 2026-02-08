# Request Normalizer Contract

## Boundary

`RequestNormalizer` accepts `BelunaInferenceRequest` and returns either:

- `CanonicalRequest`
- `GatewayError(kind = InvalidRequest)` for invalid linkage/schema states

## Scenarios

### Scenario: Missing request id is generated

- Given: a valid inference request with `request_id = null`
- When: the request is normalized
- Then: the result is successful
- Then: `canonical.request_id` is non-empty

### Scenario: Tool message without `tool_call_id` is rejected

- Given: a request containing one `tool` role message
- Given: that message has `tool_name` but no `tool_call_id`
- When: the request is normalized
- Then: normalization fails with `InvalidRequest`
- Then: the error message contains `tool_call_id`

### Scenario: Tool message with image content part is rejected

- Given: a request containing one `tool` role message
- Given: that message contains an `image_url` content part
- When: the request is normalized
- Then: normalization fails with `InvalidRequest`
- Then: the error message indicates tool messages only support text/json parts

### Scenario: Non-tool message with tool linkage is rejected

- Given: a request containing a `user`, `assistant`, or `system` message
- Given: that message has a `tool_call_id` or `tool_name`
- When: the request is normalized
- Then: normalization fails with `InvalidRequest`
- Then: the error message contains `non-tool`

### Scenario: Tool schema with unsupported top-level keyword is rejected

- Given: a request containing a tool definition
- Given: `input_schema` has an unsupported top-level keyword
- When: the request is normalized
- Then: normalization fails with `InvalidRequest`
- Then: the error message contains `unsupported keyword`
