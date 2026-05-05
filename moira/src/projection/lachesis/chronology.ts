import { readString } from '@/coerce'

import { isAiChatFamily, isAiFamily, isAiTransportFamily, isCortexOrganFamily } from './families'
import {
  chronologySubtitle,
  chronologySubtitleForInterval,
  chronologyTitle,
  intervalLaneSubtitle,
  resolveLaneLabel,
  resolveLaneSubtitle,
} from './labels'
import type { ChronologyEntry, ChronologyLane, ChronologyLaneType, RawEvent } from './models'
import { compareChronologyEvents, eventPayloadRecord, parseObservedMs } from './raw-events'

const LANE_TYPE_ORDER: Record<ChronologyLaneType, number> = {
  owner: 0,
  tick: 1,
  cortex: 2,
  afferent: 3,
  efferent: 4,
  spine: 5,
  misc: 6,
}

const CORE_OWNER_SCOPE_ORDER = [
  'beluna.core.main.runtime',
  'beluna.core.stem.tick',
  'beluna.core.stem.afferent-pathway',
  'beluna.core.stem.afferent-rules',
  'beluna.core.stem.proprioception',
  'beluna.core.stem.descriptor-catalog',
  'beluna.core.cortex.primary',
  'beluna.core.cortex.attention',
  'beluna.core.cortex.cleanup',
  'beluna.core.cortex.sense-helper',
  'beluna.core.cortex.acts-helper',
  'beluna.core.cortex.goal-forest',
  'beluna.core.ai-gateway.chat.turn',
  'beluna.core.ai-gateway.chat.thread',
  'beluna.core.ai-gateway.transport',
  'beluna.core.stem.efferent-pathway',
  'beluna.core.spine.sense-ingress',
  'beluna.core.spine.act-routing',
  'beluna.core.spine.endpoint',
  'beluna.core.spine.adapter',
] as const

const CORE_OWNER_SCOPE_ORDER_INDEX: ReadonlyMap<string, number> = new Map(
  CORE_OWNER_SCOPE_ORDER.map((scope, index) => [scope, index]),
)

interface TimelineMetrics {
  positions: Map<string, number>
  firstObservedAt: string | null
  lastObservedAt: string | null
  usesObservedTime: boolean
}

export interface CortexOrganInterval {
  family: string
  label: string
  laneKey: string
  requestId: string | null
  startEvent: RawEvent
  endEvent: RawEvent | null
  sourceEvents: RawEvent[]
  relatedEvents: RawEvent[]
  firstEventIndex: number
}

interface OwnerInterval {
  scopeName: string
  laneKey: string
  startEvent: RawEvent
  endEvent: RawEvent | null
  sourceEvents: RawEvent[]
  firstEventIndex: number
}

export function buildChronology(rawEvents: RawEvent[], cortexIntervals: CortexOrganInterval[]) {
  const ordered = [...rawEvents].sort(compareChronologyEvents)
  if (!ordered.length) {
    return {
      lanes: [],
      eventCount: 0,
      firstObservedAt: null,
      lastObservedAt: null,
      usesObservedTime: false,
    }
  }

  const timeline = buildTimelineMetrics(ordered)
  const rawEventIndex = new Map(ordered.map((event, index) => [event.rawEventId, index]))
  const ownerIntervals = buildOwnerIntervals(ordered, rawEventIndex)
  const laneMap = new Map<
    string,
    Omit<ChronologyLane, 'entries' | 'eventCount'> & { entries: ChronologyEntry[]; firstEventIndex: number }
  >()
  const hiddenRawEventIds = new Set<string>()

  for (const interval of ownerIntervals) {
    for (const event of interval.sourceEvents) {
      hiddenRawEventIds.add(event.rawEventId)
    }

    const startPosition = timeline.positions.get(interval.startEvent.rawEventId) ?? 0.05
    const endPosition = interval.endEvent
      ? timeline.positions.get(interval.endEvent.rawEventId) ?? clamp01(startPosition + 0.12)
      : clamp01(startPosition + 0.1)
    const startPayload = eventPayloadRecord(interval.startEvent)

    appendChronologyEntry(laneMap, {
      laneId: `owner:${interval.laneKey}`,
      laneType: 'owner',
      laneKey: interval.laneKey,
      laneLabel: resolveLaneLabel('owner', interval.laneKey, startPayload, interval.startEvent),
      laneSubtitle: resolveLaneSubtitle('owner', startPayload, interval.startEvent),
      entry: {
        rawEventId: interval.startEvent.rawEventId,
        laneType: 'owner',
        laneKey: interval.laneKey,
        entryType: 'interval',
        title: ownerIntervalTitle(interval),
        subtitle: ownerIntervalSubtitle(interval),
        family: interval.startEvent.family ?? interval.endEvent?.family ?? null,
        observedAt: interval.startEvent.observedAt,
        severityText: interval.endEvent?.severityText ?? interval.startEvent.severityText,
        eventIndex: interval.firstEventIndex,
        position: startPosition,
        endPosition: Math.max(endPosition, Math.min(0.98, startPosition + 0.04)),
        event: interval.endEvent ?? interval.startEvent,
        sourceEvents: interval.sourceEvents,
        relatedEvents: [],
      },
    })
  }

  for (const interval of cortexIntervals) {
    for (const event of interval.sourceEvents) {
      hiddenRawEventIds.add(event.rawEventId)
    }
    for (const event of interval.relatedEvents) {
      hiddenRawEventIds.add(event.rawEventId)
    }

    const startPosition = timeline.positions.get(interval.startEvent.rawEventId) ?? 0.05
    const endPosition = interval.endEvent
      ? timeline.positions.get(interval.endEvent.rawEventId) ?? clamp01(startPosition + 0.12)
      : clamp01(startPosition + 0.1)
    const startPayload = eventPayloadRecord(interval.startEvent)

    appendChronologyEntry(laneMap, {
      laneId: `cortex:${interval.laneKey}`,
      laneType: 'cortex',
      laneKey: interval.laneKey,
      laneLabel: interval.label,
      laneSubtitle: intervalLaneSubtitle(firstString(startPayload, ['route_or_backend']), interval.requestId),
      entry: {
        rawEventId: interval.startEvent.rawEventId,
        laneType: 'cortex',
        laneKey: interval.laneKey,
        entryType: 'interval',
        title: interval.label,
        subtitle: chronologySubtitleForInterval(interval.requestId, interval.relatedEvents.length),
        family: interval.family,
        observedAt: interval.startEvent.observedAt,
        severityText: interval.endEvent?.severityText ?? interval.startEvent.severityText,
        eventIndex: interval.firstEventIndex,
        position: startPosition,
        endPosition: Math.max(endPosition, Math.min(0.98, startPosition + 0.04)),
        event: interval.endEvent ?? interval.startEvent,
        sourceEvents: interval.sourceEvents,
        relatedEvents: interval.relatedEvents,
      },
    })
  }

  ordered.forEach((event, eventIndex) => {
    if (hiddenRawEventIds.has(event.rawEventId)) {
      return
    }

    const payload = eventPayloadRecord(event)
    const laneType = resolveLaneType(event, payload)
    const laneKey = resolveLaneKey(laneType, event, payload)

    appendChronologyEntry(laneMap, {
      laneId: `${laneType}:${laneKey}`,
      laneType,
      laneKey,
      laneLabel: resolveLaneLabel(laneType, laneKey, payload, event),
      laneSubtitle: resolveLaneSubtitle(laneType, payload, event),
      entry: {
        rawEventId: event.rawEventId,
        laneType,
        laneKey,
        entryType: 'point',
        title: chronologyTitle(event, payload),
        subtitle: chronologySubtitle(event, payload),
        family: event.family,
        observedAt: event.observedAt,
        severityText: event.severityText,
        eventIndex,
        position: timeline.positions.get(event.rawEventId) ?? 0.05,
        endPosition: clamp01((timeline.positions.get(event.rawEventId) ?? 0.05) + 0.035),
        event,
        sourceEvents: [event],
        relatedEvents: [],
      },
    })
  })

  const lanes = [...laneMap.values()]
    .map((lane) => {
      const entries = [...lane.entries].sort((left, right) => {
        if (left.position !== right.position) {
          return left.position - right.position
        }
        return left.eventIndex - right.eventIndex
      })

      return {
        id: lane.id,
        laneType: lane.laneType,
        laneKey: lane.laneKey,
        label: lane.label,
        subtitle: lane.subtitle,
        eventCount: entries.length,
        entries,
        firstEventIndex: lane.firstEventIndex,
      }
    })
    .sort((left, right) => {
      const laneTypeOrder = LANE_TYPE_ORDER[left.laneType] - LANE_TYPE_ORDER[right.laneType]
      if (laneTypeOrder !== 0) {
        return laneTypeOrder
      }
      if (left.laneType === 'owner' && right.laneType === 'owner') {
        const ownerOrder = ownerLaneOrder(left.laneKey) - ownerLaneOrder(right.laneKey)
        if (ownerOrder !== 0) {
          return ownerOrder
        }
      }
      if (left.firstEventIndex !== right.firstEventIndex) {
        return left.firstEventIndex - right.firstEventIndex
      }
      return left.label.localeCompare(right.label)
    })
    .map(({ firstEventIndex: _firstEventIndex, ...lane }) => lane)

  return {
    lanes,
    eventCount: ordered.length,
    firstObservedAt: timeline.firstObservedAt,
    lastObservedAt: timeline.lastObservedAt,
    usesObservedTime: timeline.usesObservedTime,
  }
}

export function buildCortexOrganIntervals(
  rawEvents: RawEvent[],
  cortexEvents: RawEvent[],
  aiGatewayEvents: RawEvent[],
  organFamilyLabel: (family: string | null) => string,
): CortexOrganInterval[] {
  const orderedRaw = [...rawEvents].sort(compareChronologyEvents)
  const rawEventIndex = new Map(orderedRaw.map((event, index) => [event.rawEventId, index]))
  const orderedCortex = cortexEvents
    .filter((event) => isCortexOrganFamily(event.family))
    .sort(compareChronologyEvents)
  const openIntervals = new Map<string, RawEvent>()
  const claimedAiEventIds = new Set<string>()
  const intervals: CortexOrganInterval[] = []

  for (const event of orderedCortex) {
    const payload = eventPayloadRecord(event)
    const phase = firstString(payload, ['phase'])
    const requestId = firstString(payload, ['request_id'])
    const key = `${event.family ?? 'cortex'}:${requestId ?? event.rawEventId}`

    if (phase === 'start') {
      openIntervals.set(key, event)
      continue
    }

    if (phase === 'end') {
      const startEvent = openIntervals.get(key)
      if (!startEvent) {
        continue
      }

      openIntervals.delete(key)
      intervals.push(
        createCortexInterval(startEvent, event, aiGatewayEvents, rawEventIndex, claimedAiEventIds, organFamilyLabel),
      )
    }
  }

  for (const event of openIntervals.values()) {
    intervals.push(createCortexInterval(event, null, aiGatewayEvents, rawEventIndex, claimedAiEventIds, organFamilyLabel))
  }

  return intervals.sort((left, right) => left.firstEventIndex - right.firstEventIndex)
}

function buildOwnerIntervals(
  orderedEvents: RawEvent[],
  rawEventIndex: Map<string, number>,
): OwnerInterval[] {
  const openIntervals = new Map<string, RawEvent>()
  const intervals: OwnerInterval[] = []

  for (const event of orderedEvents) {
    const scopeName = event.scopeName
    const eventName = event.eventName
    if (!scopeName?.startsWith('beluna.core.') || !eventName || !event.spanId) {
      continue
    }

    const role = ownerBoundaryRole(eventName)
    if (!role) {
      continue
    }

    const key = `${scopeName}:${event.spanId}`
    if (role === 'start') {
      openIntervals.set(key, event)
      continue
    }

    const startEvent = openIntervals.get(key)
    if (!startEvent) {
      continue
    }

    openIntervals.delete(key)
    intervals.push({
      scopeName,
      laneKey: scopeName,
      startEvent,
      endEvent: event,
      sourceEvents: [startEvent, event],
      firstEventIndex: rawEventIndex.get(startEvent.rawEventId) ?? 0,
    })
  }

  for (const startEvent of openIntervals.values()) {
    const scopeName = startEvent.scopeName
    if (!scopeName) {
      continue
    }
    intervals.push({
      scopeName,
      laneKey: scopeName,
      startEvent,
      endEvent: null,
      sourceEvents: [startEvent],
      firstEventIndex: rawEventIndex.get(startEvent.rawEventId) ?? 0,
    })
  }

  return intervals.sort((left, right) => left.firstEventIndex - right.firstEventIndex)
}

function ownerBoundaryRole(eventName: string): 'start' | 'end' | null {
  switch (eventName) {
    case 'started':
    case 'request.started':
    case 'dispatch.started':
    case 'dispatched':
      return 'start'
    case 'finished':
    case 'request.completed':
    case 'dispatch.finished':
    case 'committed':
    case 'failed':
      return 'end'
    default:
      return null
  }
}

function ownerIntervalTitle(interval: OwnerInterval): string {
  const start = interval.startEvent.eventName ?? 'started'
  const end = interval.endEvent?.eventName
  return end ? `${start} -> ${end}` : start
}

function ownerIntervalSubtitle(interval: OwnerInterval): string | null {
  return [
    interval.startEvent.spanId ? `span ${abbreviateId(interval.startEvent.spanId)}` : null,
    interval.sourceEvents.length > 1 ? `${interval.sourceEvents.length} boundary events` : null,
  ]
    .filter(Boolean)
    .join(' · ') || null
}

function createCortexInterval(
  startEvent: RawEvent,
  endEvent: RawEvent | null,
  aiGatewayEvents: RawEvent[],
  rawEventIndex: Map<string, number>,
  claimedAiEventIds: Set<string>,
  organFamilyLabel: (family: string | null) => string,
): CortexOrganInterval {
  const family = startEvent.family ?? endEvent?.family ?? 'cortex.primary'
  const startPayload = eventPayloadRecord(startEvent)
  const endPayload = endEvent ? eventPayloadRecord(endEvent) : {}
  const requestId = firstString(startPayload, ['request_id']) ?? firstString(endPayload, ['request_id'])
  const aiRequestId =
    firstString(endPayload, ['ai_request_id', 'ai_request_id_when_present']) ??
    firstString(startPayload, ['ai_request_id', 'ai_request_id_when_present'])
  const threadId =
    firstString(endPayload, ['thread_id', 'thread_id_when_present']) ??
    firstString(startPayload, ['thread_id', 'thread_id_when_present'])
  const turnId =
    firstString(endPayload, ['turn_id', 'turn_id_when_present']) ??
    firstString(startPayload, ['turn_id', 'turn_id_when_present'])
  const sourceEvents = endEvent ? [startEvent, endEvent] : [startEvent]
  const relatedEvents = aiGatewayEvents
    .filter((event) => {
      if (claimedAiEventIds.has(event.rawEventId)) {
        return false
      }

      if (!isAiFamily(event.family)) {
        return false
      }

      return relatesAiEventToInterval(event, requestId, aiRequestId, threadId, turnId)
    })
    .sort(compareChronologyEvents)

  relatedEvents.forEach((event) => claimedAiEventIds.add(event.rawEventId))

  return {
    family,
    label: organFamilyLabel(family),
    laneKey: requestId ?? startEvent.rawEventId,
    requestId,
    startEvent,
    endEvent,
    sourceEvents,
    relatedEvents,
    firstEventIndex: rawEventIndex.get(startEvent.rawEventId) ?? 0,
  }
}

function relatesAiEventToInterval(
  event: RawEvent,
  requestId: string | null,
  aiRequestId: string | null,
  threadId: string | null,
  turnId: string | null,
): boolean {
  const payload = eventPayloadRecord(event)
  const parentSpanId = firstString(payload, ['parent_span_id', 'parent_span_id_when_present'])
  const eventRequestId = firstString(payload, ['request_id', 'request_id_when_present'])
  const eventThreadId = firstString(payload, ['thread_id', 'thread_id_when_present'])
  const eventTurnId = firstString(payload, ['turn_id', 'turn_id_when_present'])

  if (requestId && parentSpanId === requestId) {
    return true
  }

  if (isAiTransportFamily(event.family)) {
    return !!aiRequestId && eventRequestId === aiRequestId
  }

  if (!isAiChatFamily(event.family)) {
    return false
  }

  if (aiRequestId && eventRequestId === aiRequestId) {
    return true
  }

  if (threadId && eventThreadId === threadId) {
    return true
  }

  if (turnId && eventTurnId === turnId) {
    return true
  }

  return false
}

function buildTimelineMetrics(ordered: RawEvent[]): TimelineMetrics {
  const firstObservedAt = ordered[0]?.observedAt ?? null
  const lastObservedAt = ordered[ordered.length - 1]?.observedAt ?? null
  const observedMs = ordered
    .map((event) => parseObservedMs(event.observedAt))
    .filter((value): value is number => value != null)
  const minObservedMs = observedMs.length ? Math.min(...observedMs) : null
  const maxObservedMs = observedMs.length ? Math.max(...observedMs) : null
  const usesObservedTime = minObservedMs != null && maxObservedMs != null && maxObservedMs > minObservedMs
  const sequenceDenominator = Math.max(ordered.length - 1, 1)
  const positions = new Map<string, number>()

  ordered.forEach((event, eventIndex) => {
    const timeRatio = usesObservedTime
      ? ((parseObservedMs(event.observedAt) ?? minObservedMs ?? 0) - (minObservedMs ?? 0)) /
        ((maxObservedMs ?? 1) - (minObservedMs ?? 0))
      : null
    const sequenceRatio = eventIndex / sequenceDenominator
    const position = clamp01(0.03 + ((timeRatio ?? sequenceRatio) * 0.88 + sequenceRatio * 0.09))
    positions.set(event.rawEventId, position)
  })

  return {
    positions,
    firstObservedAt,
    lastObservedAt,
    usesObservedTime,
  }
}

function appendChronologyEntry(
  laneMap: Map<
    string,
    Omit<ChronologyLane, 'entries' | 'eventCount'> & { entries: ChronologyEntry[]; firstEventIndex: number }
  >,
  input: {
    laneId: string
    laneType: ChronologyLaneType
    laneKey: string
    laneLabel: string
    laneSubtitle: string | null
    entry: ChronologyEntry
  },
): void {
  const existing = laneMap.get(input.laneId)
  if (existing) {
    existing.entries.push(input.entry)
    return
  }

  laneMap.set(input.laneId, {
    id: input.laneId,
    laneType: input.laneType,
    laneKey: input.laneKey,
    label: input.laneLabel,
    subtitle: input.laneSubtitle,
    entries: [input.entry],
    firstEventIndex: input.entry.eventIndex,
  })
}

function resolveLaneType(event: RawEvent, payload: Record<string, unknown>): ChronologyLaneType {
  if (event.scopeName?.startsWith('beluna.core.')) {
    return 'owner'
  }

  const family = event.family ?? ''

  if (family === 'stem.tick') {
    return 'tick'
  }

  if (isCortexOrganFamily(family) || family === 'cortex.goal-forest') {
    return 'cortex'
  }

  if (family === 'stem.afferent') {
    return 'afferent'
  }

  if (family === 'stem.efferent') {
    return 'efferent'
  }

  if (family.startsWith('spine.')) {
    return 'spine'
  }

  if (isAiFamily(family)) {
    return 'misc'
  }

  if (payload.sense_id || payload.endpoint_id) {
    return 'afferent'
  }

  if (payload.act_id) {
    return 'efferent'
  }

  return 'misc'
}

function resolveLaneKey(
  laneType: ChronologyLaneType,
  event: RawEvent,
  payload: Record<string, unknown>,
): string {
  switch (laneType) {
    case 'owner':
      return event.scopeName ?? event.rawEventId
    case 'tick':
      return event.traceId ? `trace:${event.traceId}` : `tick:${event.tick ?? '0'}`
    case 'cortex':
      return event.spanId ?? firstString(payload, ['request_id']) ?? event.eventName ?? event.family ?? event.rawEventId
    case 'afferent':
      return firstString(payload, ['sense_id', 'descriptor_id', 'endpoint_id']) ?? event.rawEventId
    case 'efferent':
      return firstString(payload, ['act_id', 'descriptor_id', 'endpoint_id']) ?? event.rawEventId
    case 'spine':
      return event.spanId ?? firstString(payload, ['endpoint_id', 'adapter_id', 'act_id', 'sense_id']) ?? event.rawEventId
    case 'misc':
      return event.spanId ?? firstString(payload, ['span_id', 'request_id']) ?? event.eventName ?? event.rawEventId
  }
}

function ownerLaneOrder(laneKey: string): number {
  return CORE_OWNER_SCOPE_ORDER_INDEX.get(laneKey) ?? Number.MAX_SAFE_INTEGER
}

function abbreviateId(value: string): string {
  if (value.length <= 22) {
    return value
  }

  return `${value.slice(0, 10)}...${value.slice(-8)}`
}

function firstString(record: Record<string, unknown>, keys: string[]): string | null {
  return readString(record, keys)
}

function clamp01(value: number): number {
  return Math.min(0.98, Math.max(0.02, value))
}
