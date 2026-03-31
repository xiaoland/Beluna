import { stringify, toRecord } from '@/coerce'

import { isCortexOrganFamily } from './families'
import type { ChronologyLaneType, RawEvent } from './models'

export function rawEventHeadline(event: RawEvent): string {
  const payload = toRecord(event.payload)
  const identity = [
    stringify(payload.request_id),
    stringify(payload.ai_request_id),
    stringify(payload.capability),
    stringify(payload.thread_id),
    stringify(payload.turn_id),
    stringify(payload.attempt),
    stringify(payload.sense_id),
    stringify(payload.act_id),
    stringify(payload.endpoint_id),
    stringify(payload.adapter_id),
  ].filter(Boolean)
  const state = [
    stringify(payload.kind),
    stringify(payload.phase),
    stringify(payload.status),
    stringify(payload.change_mode),
    stringify(payload.outcome),
    stringify(payload.terminal_outcome),
  ].filter(Boolean)

  return (
    event.messageText ||
    [event.family, state.join(' / '), identity.join(' · ')].filter(Boolean).join(' · ') ||
    [event.subsystem, event.family, event.target].filter(Boolean).join(' / ') ||
    'Raw event'
  )
}

export function summarizeEntry(value: unknown): string {
  if (typeof value === 'string') {
    return value
  }

  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value)
  }

  if (Array.isArray(value)) {
    return `${value.length} item${value.length === 1 ? '' : 's'}`
  }

  const record = toRecord(value)
  const nestedMessage = toRecord(record.message)
  if (Object.keys(nestedMessage).length > 0) {
    return [
      stringify(record.turn_id) ? `turn ${stringify(record.turn_id)}` : null,
      stringify(nestedMessage.kind),
      stringify(nestedMessage.id),
    ]
      .filter(Boolean)
      .join(' · ')
  }

  const candidates = [
    'label',
    'name',
    'message',
    'message_text',
    'messageText',
    'organ',
    'family',
    'descriptor',
    'descriptor_id',
    'descriptorId',
    'catalog_version',
    'change_mode',
    'adapter_id',
    'adapterId',
    'request_id',
    'requestId',
    'ai_request_id',
    'thread_id',
    'threadId',
    'turn_id',
    'turnId',
    'endpoint_id',
    'endpointId',
    'act_id',
    'actId',
    'sense_id',
    'senseId',
    'phase',
    'status',
    'kind',
    'binding_kind',
    'model',
    'backend_id',
    'capability',
    'attempt',
    'tick',
    'outcome',
    'terminal_outcome',
    'id',
  ]

  for (const key of candidates) {
    const candidate = stringify(record[key])
    if (candidate) {
      return candidate
    }
  }

  const keys = Object.keys(record).slice(0, 3)
  return keys.length ? keys.join(' · ') : 'Structured entry'
}

export function organFamilyLabel(family: string | null): string {
  switch (family) {
    case 'cortex.primary':
      return 'Cortex Primary'
    case 'cortex.sense-helper':
      return 'Sense Helper'
    case 'cortex.goal-forest-helper':
      return 'Goal Forest Helper'
    case 'cortex.acts-helper':
      return 'Acts Helper'
    default:
      return family ?? 'Cortex'
  }
}

export function resolveLaneLabel(
  laneType: ChronologyLaneType,
  laneKey: string,
  payload: Record<string, unknown>,
  event: RawEvent,
): string {
  switch (laneType) {
    case 'tick':
      return `Tick ${event.tick ?? laneKey}`
    case 'cortex':
      return event.family === 'cortex.goal-forest' ? 'Goal Forest' : organFamilyLabel(event.family)
    case 'afferent':
      return stringify(payload.sense_id) ?? stringify(payload.descriptor_id) ?? abbreviateId(laneKey)
    case 'efferent':
      return stringify(payload.act_id) ?? stringify(payload.descriptor_id) ?? abbreviateId(laneKey)
    case 'spine':
      if (event.family === 'spine.adapter') {
        return stringify(payload.adapter_id) ?? 'Adapter'
      }
      return stringify(payload.endpoint_id) ?? stringify(payload.act_id) ?? stringify(payload.sense_id) ?? abbreviateId(laneKey)
    case 'misc':
      return event.family ?? abbreviateId(laneKey)
  }
}

export function resolveLaneSubtitle(
  laneType: ChronologyLaneType,
  payload: Record<string, unknown>,
  event: RawEvent,
): string | null {
  switch (laneType) {
    case 'tick':
      return stringify(payload.status)
    case 'cortex':
      return [stringify(payload.route_or_backend), stringify(payload.request_id)].filter(Boolean).join(' · ') || null
    case 'afferent':
      return [stringify(payload.descriptor_id), stringify(payload.endpoint_id), stringify(payload.kind)]
        .filter(Boolean)
        .join(' · ') || null
    case 'efferent':
      return [stringify(payload.descriptor_id), stringify(payload.endpoint_id), stringify(payload.kind)]
        .filter(Boolean)
        .join(' · ') || null
    case 'spine':
      return [
        stringify(payload.binding_kind),
        stringify(payload.channel_or_session),
        stringify(payload.outcome),
        stringify(payload.kind),
      ]
        .filter(Boolean)
        .join(' · ') || event.subsystem
    case 'misc':
      return event.subsystem
  }
}

export function chronologyTitle(event: RawEvent, payload: Record<string, unknown>): string {
  if (isCortexOrganFamily(event.family)) {
    return organFamilyLabel(event.family)
  }

  return (
    stringify(payload.kind) ??
    stringify(payload.phase) ??
    stringify(payload.status) ??
    stringify(payload.change_mode) ??
    stringify(payload.descriptor_id) ??
    stringify(payload.sense_id) ??
    stringify(payload.act_id) ??
    stringify(payload.endpoint_id) ??
    event.family ??
    'event'
  )
}

export function chronologySubtitle(event: RawEvent, payload: Record<string, unknown>): string | null {
  const fragments = [
    event.family,
    stringify(payload.descriptor_id),
    stringify(payload.request_id),
    stringify(payload.endpoint_id),
    stringify(payload.outcome) ?? stringify(payload.terminal_outcome),
  ].filter(Boolean)

  return fragments.length ? fragments.join(' · ') : null
}

export function chronologySubtitleForInterval(requestId: string | null, relatedEventCount: number): string | null {
  return [
    requestId ? abbreviateId(requestId) : null,
    relatedEventCount ? `${relatedEventCount} linked AI entr${relatedEventCount === 1 ? 'y' : 'ies'}` : null,
  ]
    .filter(Boolean)
    .join(' · ') || null
}

export function intervalLaneSubtitle(routeOrBackend: string | null, requestId: string | null): string | null {
  return [routeOrBackend, requestId ? abbreviateId(requestId) : null].filter(Boolean).join(' · ') || null
}

function abbreviateId(value: string): string {
  if (value.length <= 22) {
    return value
  }

  return `${value.slice(0, 10)}…${value.slice(-8)}`
}
