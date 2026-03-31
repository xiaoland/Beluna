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
      key: 'payload',
      title: 'Payload',
      value: event.payload,
      defaultOpen: options.openPayload ?? true,
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
