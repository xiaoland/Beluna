import type { EventRecordPayload } from '@/bridge/contracts/lachesis'
import {
  compareDateDesc,
  cryptoRandomId,
  parseMaybeJson,
  read,
  readString,
  toRecord,
} from '@/coerce'
import type { RawEvent } from './models'

export function normalizeRawEvent(
  value: EventRecordPayload,
  fallbackRunId: string | null = null,
  fallbackTick: number | null = null,
): RawEvent {
  const attributes = toRecord(parseMaybeJson(value.attributes))
  const body = parseMaybeJson(value.body)
  const tickAttribute = read(attributes, ['tick', 'cycle_id'])

  return {
    rawEventId: value.rawEventId ?? cryptoRandomId(),
    receivedAt: value.receivedAt ?? null,
    observedAt: value.observedAt ?? null,
    severityText: value.severityText ?? null,
    severityNumber: value.severityNumber ?? null,
    target: value.target ?? null,
    family: value.family ?? null,
    subsystem: value.subsystem ?? null,
    runId: value.runId ?? readString(attributes, ['run_id']) ?? fallbackRunId,
    tick:
      value.tick ??
      coerceTick(tickAttribute) ??
      coerceTick(fallbackTick),
    messageText:
      value.messageText ??
      readString(attributes, ['message']) ??
      readString(toRecord(body), ['message']) ??
      (typeof body === 'string' ? body : null),
    payload: parseEventPayload(attributes, body),
    body,
    attributes,
    resource: toRecord(parseMaybeJson(value.resource)),
    scope: toRecord(parseMaybeJson(value.scope)),
  }
}

export function compareChronologyEvents(left: RawEvent, right: RawEvent): number {
  const leftMs = parseObservedMs(left.observedAt) ?? Number.POSITIVE_INFINITY
  const rightMs = parseObservedMs(right.observedAt) ?? Number.POSITIVE_INFINITY
  if (leftMs !== rightMs) {
    return leftMs - rightMs
  }
  return left.rawEventId.localeCompare(right.rawEventId)
}

export function compareTicksByObservedAt(left: string | null, right: string | null): number {
  return compareDateDesc(left, right)
}

export function parseObservedMs(value: string | null): number | null {
  if (!value) {
    return null
  }

  const parsed = Date.parse(value)
  return Number.isFinite(parsed) ? parsed : null
}

export function eventsForSubsystem(explicit: RawEvent[], rawEvents: RawEvent[], subsystem: string): RawEvent[] {
  if (explicit.length > 0) {
    return explicit
  }

  return rawEvents.filter((event) => event.subsystem === subsystem)
}

export function eventsForFamilies(
  explicit: RawEvent[],
  rawEvents: RawEvent[],
  predicate: (family: string | null) => boolean,
): RawEvent[] {
  if (explicit.length > 0) {
    return explicit
  }

  return rawEvents.filter((event) => predicate(event.family))
}

export function firstPayloadValue(events: RawEvent[], family: string, keys: string[]): unknown | null {
  for (const event of events) {
    if (event.family !== family) {
      continue
    }

    const value = read(eventPayloadRecord(event), keys)
    if (value != null) {
      return value
    }
  }

  return null
}

function coerceTick(value: unknown): number | null {
  if (typeof value === 'number' && Number.isFinite(value)) {
    return value
  }

  if (typeof value === 'string') {
    const parsed = Number(value)
    return Number.isFinite(parsed) ? parsed : null
  }

  return null
}

export function collectNarratives(events: RawEvent[], families?: string[]): unknown[] {
  return events
    .filter((event) => !families || families.includes(event.family ?? ''))
    .map(eventNarrative)
}

export function eventNarrative(event: RawEvent): unknown {
  const payload = event.payload
  if (payload && typeof payload === 'object' && !Array.isArray(payload)) {
    return {
      family: event.family,
      observed_at: event.observedAt,
      severity: event.severityText,
      ...payload,
    }
  }

  return {
    family: event.family,
    observed_at: event.observedAt,
    severity: event.severityText,
    target: event.target,
    message_text: event.messageText,
  }
}

export function eventPayloadRecord(event: RawEvent): Record<string, unknown> {
  return toRecord(event.payload)
}

function parseEventPayload(attributes: Record<string, unknown>, body: unknown): unknown | null {
  const payload = parseMaybeJson(attributes.payload)
  if (payload != null) {
    return payload
  }

  if (body && typeof body === 'object') {
    return body
  }

  return null
}
