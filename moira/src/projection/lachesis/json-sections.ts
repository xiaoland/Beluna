import type { RawEvent } from './models'
import type { JsonSectionInput } from '@/presentation/loom/shared/json'

export function jsonSectionsForEvent(
  event: RawEvent,
  options: {
    openPayload?: boolean
  } = {},
): JsonSectionInput[] {
  return [
    {
      key: 'otel',
      title: 'OTLP',
      value: {
        record_kind: event.recordKind,
        scope_name: event.scopeName,
        event_name: event.eventName,
        trace_id: event.traceId,
        span_id: event.spanId,
        trace_flags: event.traceFlags,
      },
      defaultOpen: true,
    },
    {
      key: 'payload',
      title: 'Payload',
      value: event.payload,
      defaultOpen: options.openPayload ?? false,
    },
    {
      key: 'body',
      title: 'Body',
      value: event.body,
    },
    {
      key: 'attributes',
      title: 'Attributes',
      value: event.attributes,
    },
    {
      key: 'resource',
      title: 'Resource',
      value: event.resource,
    },
    {
      key: 'scope',
      title: 'Scope',
      value: event.scope,
    },
  ]
}
