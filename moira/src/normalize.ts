import type {
  ChronologyEntry,
  ChronologyLane,
  ChronologyLaneType,
  RawEvent,
  ReceiverStatus,
  TickDetail,
  TickSummary,
  WakeSessionSummary,
} from './types'
import {
  compareDateDesc,
  cryptoRandomId,
  parseMaybeJson,
  read,
  readArray,
  readNumber,
  readString,
  stringify,
  toRecord,
} from './coerce'

const CORTEX_ORGAN_FAMILIES = [
  'cortex.primary',
  'cortex.sense-helper',
  'cortex.goal-forest-helper',
  'cortex.acts-helper',
] as const

const LANE_TYPE_ORDER: Record<ChronologyLaneType, number> = {
  tick: 0,
  cortex: 1,
  afferent: 2,
  efferent: 3,
  spine: 4,
  misc: 5,
}

interface TimelineMetrics {
  positions: Map<string, number>
  firstObservedAt: string | null
  lastObservedAt: string | null
  usesObservedTime: boolean
}

interface CortexOrganInterval {
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

export function normalizeReceiverStatus(value: unknown): ReceiverStatus {
  const record = toRecord(value)

  return {
    state: readString(record, ['wakeState', 'wake_state', 'state', 'receiver_state']) ?? 'unknown',
    storagePath: readString(record, ['dbPath', 'db_path', 'storagePath', 'storage_path']),
    receiverBind: readString(record, ['endpoint', 'receiverBind', 'receiver_bind', 'listen_addr', 'listenAddr']),
    lastIngestAt: readString(record, ['lastBatchAt', 'last_batch_at', 'lastIngestAt', 'last_ingest_at']),
    rawEventCount: readNumber(record, ['rawEventCount', 'raw_event_count']),
    runCount: readNumber(record, ['wakeCount', 'wake_count', 'runCount', 'run_count']),
    tickCount: readNumber(record, ['tickCount', 'tick_count']),
    note: readString(record, ['lastError', 'last_error', 'note', 'message']),
  }
}

export function normalizeWakeSession(value: unknown): WakeSessionSummary {
  const record = toRecord(value)

  return {
    runId: stringify(read(record, ['runId', 'run_id'])) ?? 'unknown-run',
    firstSeenAt: readString(record, ['firstSeenAt', 'first_seen_at']),
    lastSeenAt: readString(record, ['lastSeenAt', 'last_seen_at']),
    eventCount: readNumber(record, ['eventCount', 'event_count']) ?? 0,
    warningCount: readNumber(record, ['warningCount', 'warning_count']) ?? 0,
    errorCount: readNumber(record, ['errorCount', 'error_count']) ?? 0,
    latestTick: stringify(read(record, ['latestTick', 'latest_tick'])),
    state: readString(record, ['state', 'run_state']),
  }
}

export function normalizeTickSummary(value: unknown): TickSummary {
  const record = toRecord(value)

  return {
    runId: stringify(read(record, ['runId', 'run_id'])) ?? '',
    tick: stringify(read(record, ['tick', 'cycle_id'])) ?? 'unknown-tick',
    firstSeenAt: readString(record, ['firstSeenAt', 'first_seen_at']),
    lastSeenAt: readString(record, ['lastSeenAt', 'last_seen_at']),
    eventCount: readNumber(record, ['eventCount', 'event_count']) ?? 0,
    warningCount: readNumber(record, ['warningCount', 'warning_count']) ?? 0,
    errorCount: readNumber(record, ['errorCount', 'error_count']) ?? 0,
  }
}

export function normalizeTickDetail(value: unknown): TickDetail {
  const record = toRecord(value)
  const summaryRecord = toRecord(read(record, ['summary']))
  const runId =
    stringify(read(summaryRecord, ['runId', 'run_id'])) ??
    stringify(read(record, ['runId', 'run_id'])) ??
    ''
  const tick =
    stringify(read(summaryRecord, ['tick', 'cycle_id'])) ??
    stringify(read(record, ['tick', 'cycle_id'])) ??
    ''
  const rawEvents = readArray(record, ['raw', 'rawEvents', 'raw_events', 'events']).map((item) =>
    normalizeRawEvent(item, runId, tick),
  )
  const aiGatewayEvents = eventsForSubsystem(
    readArray(record, ['aiGateway', 'ai_gateway']).map((item) => normalizeRawEvent(item, runId, tick)),
    rawEvents,
    'ai-gateway',
  )
  const cortexEvents = eventsForSubsystem(
    readArray(record, ['cortex']).map((item) => normalizeRawEvent(item, runId, tick)),
    rawEvents,
    'cortex',
  )
  const stemEvents = eventsForSubsystem(
    readArray(record, ['stem']).map((item) => normalizeRawEvent(item, runId, tick)),
    rawEvents,
    'stem',
  )
  const spineEvents = eventsForSubsystem(
    readArray(record, ['spine']).map((item) => normalizeRawEvent(item, runId, tick)),
    rawEvents,
    'spine',
  )
  const cortexOrganIntervals = buildCortexOrganIntervals(rawEvents, cortexEvents, aiGatewayEvents)

  return {
    runId,
    tick,
    chronology: buildChronology(rawEvents, cortexOrganIntervals),
    cortex: {
      organs: cortexOrganIntervals.map(intervalNarrative),
      goalForestEvents: collectNarratives(cortexEvents.filter((event) => event.family === 'cortex.goal-forest')),
      goalForest:
        firstPayloadValue(cortexEvents, 'cortex.goal-forest', ['snapshot']) ??
        firstPayloadValue(cortexEvents, 'cortex.goal-forest', ['mutation_result']),
    },
    stem: {
      tickAnchor: collectNarratives(stemEvents.filter((event) => event.family === 'stem.tick')),
      afferent: collectNarratives(stemEvents.filter((event) => event.family === 'stem.afferent')),
      efferent: collectNarratives(stemEvents.filter((event) => event.family === 'stem.efferent')),
      nsCatalog: collectNarratives(stemEvents.filter((event) => event.family === 'stem.ns-catalog')),
      proprioception: collectNarratives(stemEvents.filter((event) => event.family === 'stem.proprioception')),
      afferentRules: collectNarratives(stemEvents.filter((event) => event.family === 'stem.afferent.rule')),
    },
    spine: {
      adapters: collectNarratives(spineEvents.filter((event) => event.family === 'spine.adapter')),
      endpoints: collectNarratives(spineEvents.filter((event) => event.family === 'spine.endpoint')),
      senses: collectNarratives(spineEvents.filter((event) => event.family === 'spine.sense')),
      acts: collectNarratives(spineEvents.filter((event) => event.family === 'spine.act')),
    },
    rawEvents,
  }
}

export function compareWakeSessions(left: WakeSessionSummary, right: WakeSessionSummary): number {
  return compareDateDesc(left.lastSeenAt, right.lastSeenAt)
}

export function compareTicks(left: TickSummary, right: TickSummary): number {
  const leftNumber = Number(left.tick)
  const rightNumber = Number(right.tick)

  if (Number.isFinite(leftNumber) && Number.isFinite(rightNumber) && leftNumber !== rightNumber) {
    return rightNumber - leftNumber
  }

  return compareDateDesc(left.lastSeenAt, right.lastSeenAt)
}

function normalizeRawEvent(
  value: unknown,
  fallbackRunId: string | null = null,
  fallbackTick: string | null = null,
): RawEvent {
  const record = toRecord(value)
  const attributes = toRecord(parseMaybeJson(read(record, ['attributes', 'attributesJson', 'attributes_json'])))
  const body = parseMaybeJson(read(record, ['body', 'bodyJson', 'body_json']))

  return {
    rawEventId: stringify(read(record, ['rawEventId', 'raw_event_id'])) ?? cryptoRandomId(),
    receivedAt: readString(record, ['receivedAt', 'received_at']),
    observedAt: readString(record, ['observedAt', 'observed_at', 'timestamp']),
    severityText: readString(record, ['severityText', 'severity_text', 'level']),
    severityNumber: readNumber(record, ['severityNumber', 'severity_number']),
    target: readString(record, ['target']),
    family: readString(record, ['family']),
    subsystem: readString(record, ['subsystem']),
    runId:
      stringify(read(record, ['runId', 'run_id'])) ??
      stringify(read(attributes, ['run_id'])) ??
      fallbackRunId,
    tick:
      stringify(read(record, ['tick', 'cycle_id'])) ??
      stringify(read(attributes, ['tick', 'cycle_id'])) ??
      fallbackTick,
    messageText:
      readString(record, ['messageText', 'message_text', 'message']) ??
      readString(attributes, ['message']) ??
      readString(toRecord(body), ['message']) ??
      (typeof body === 'string' ? body : null),
    payload: parseEventPayload(attributes, body),
    body,
    attributes,
    resource: toRecord(parseMaybeJson(read(record, ['resource', 'resourceJson', 'resource_json']))),
    scope: toRecord(parseMaybeJson(read(record, ['scope', 'scopeJson', 'scope_json']))),
  }
}

function buildChronology(rawEvents: RawEvent[], cortexIntervals: CortexOrganInterval[]) {
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
  const laneMap = new Map<
    string,
    Omit<ChronologyLane, 'entries' | 'eventCount'> & { entries: ChronologyEntry[]; firstEventIndex: number }
  >()
  const hiddenRawEventIds = new Set<string>()

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

    appendChronologyEntry(laneMap, {
      laneId: `cortex:${interval.laneKey}`,
      laneType: 'cortex',
      laneKey: interval.laneKey,
      laneLabel: interval.label,
      laneSubtitle: intervalLaneSubtitle(interval),
      entry: {
        rawEventId: interval.startEvent.rawEventId,
        laneType: 'cortex',
        laneKey: interval.laneKey,
        entryType: 'interval',
        title: interval.label,
        subtitle: chronologySubtitleForInterval(interval),
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

function buildCortexOrganIntervals(
  rawEvents: RawEvent[],
  cortexEvents: RawEvent[],
  aiGatewayEvents: RawEvent[],
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
    const phase = readString(payload, ['phase'])
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
        createCortexInterval(startEvent, event, aiGatewayEvents, rawEventIndex, claimedAiEventIds),
      )
    }
  }

  for (const event of openIntervals.values()) {
    intervals.push(createCortexInterval(event, null, aiGatewayEvents, rawEventIndex, claimedAiEventIds))
  }

  return intervals.sort((left, right) => left.firstEventIndex - right.firstEventIndex)
}

function createCortexInterval(
  startEvent: RawEvent,
  endEvent: RawEvent | null,
  aiGatewayEvents: RawEvent[],
  rawEventIndex: Map<string, number>,
  claimedAiEventIds: Set<string>,
): CortexOrganInterval {
  const family = startEvent.family ?? endEvent?.family ?? 'cortex.primary'
  const startPayload = eventPayloadRecord(startEvent)
  const endPayload = endEvent ? eventPayloadRecord(endEvent) : {}
  const requestId =
    firstString(startPayload, ['request_id']) ??
    firstString(endPayload, ['request_id'])
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

      if (!event.family?.startsWith('ai-gateway.')) {
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
  const usesObservedTime =
    minObservedMs != null && maxObservedMs != null && maxObservedMs > minObservedMs
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

function compareChronologyEvents(left: RawEvent, right: RawEvent): number {
  const leftMs = parseObservedMs(left.observedAt) ?? Number.POSITIVE_INFINITY
  const rightMs = parseObservedMs(right.observedAt) ?? Number.POSITIVE_INFINITY
  if (leftMs !== rightMs) {
    return leftMs - rightMs
  }
  return left.rawEventId.localeCompare(right.rawEventId)
}

function parseObservedMs(value: string | null): number | null {
  if (!value) {
    return null
  }

  const parsed = Date.parse(value)
  return Number.isFinite(parsed) ? parsed : null
}

function resolveLaneType(
  event: RawEvent,
  payload: Record<string, unknown>,
): ChronologyLaneType {
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

  if (family.startsWith('ai-gateway.')) {
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
    case 'tick':
      return `tick:${event.tick ?? '0'}`
    case 'cortex':
      return firstString(payload, ['request_id']) ?? event.family ?? event.rawEventId
    case 'afferent':
      return firstString(payload, ['sense_id', 'descriptor_id', 'endpoint_id']) ?? event.rawEventId
    case 'efferent':
      return firstString(payload, ['act_id', 'descriptor_id', 'endpoint_id']) ?? event.rawEventId
    case 'spine':
      return firstString(payload, ['endpoint_id', 'adapter_id', 'act_id', 'sense_id']) ?? event.rawEventId
    case 'misc':
      return firstString(payload, ['span_id', 'request_id']) ?? event.rawEventId
  }
}

function resolveLaneLabel(
  laneType: ChronologyLaneType,
  laneKey: string,
  payload: Record<string, unknown>,
  event: RawEvent,
): string {
  switch (laneType) {
    case 'tick':
      return `Tick ${event.tick ?? laneKey}`
    case 'cortex':
      return event.family === 'cortex.goal-forest'
        ? 'Goal Forest'
        : organFamilyLabel(event.family)
    case 'afferent':
      return firstString(payload, ['sense_id']) ?? firstString(payload, ['descriptor_id']) ?? abbreviateId(laneKey)
    case 'efferent':
      return firstString(payload, ['act_id']) ?? firstString(payload, ['descriptor_id']) ?? abbreviateId(laneKey)
    case 'spine':
      if (event.family === 'spine.adapter') {
        return firstString(payload, ['adapter_id']) ?? 'Adapter'
      }
      return firstString(payload, ['endpoint_id', 'act_id', 'sense_id']) ?? abbreviateId(laneKey)
    case 'misc':
      return event.family ?? abbreviateId(laneKey)
  }
}

function resolveLaneSubtitle(
  laneType: ChronologyLaneType,
  payload: Record<string, unknown>,
  event: RawEvent,
): string | null {
  switch (laneType) {
    case 'tick':
      return firstString(payload, ['status'])
    case 'cortex':
      return [
        firstString(payload, ['route_or_backend']),
        firstString(payload, ['request_id']),
      ]
        .filter(Boolean)
        .join(' · ') || null
    case 'afferent':
      return [
        firstString(payload, ['descriptor_id']),
        firstString(payload, ['endpoint_id']),
        firstString(payload, ['kind']),
      ]
        .filter(Boolean)
        .join(' · ') || null
    case 'efferent':
      return [
        firstString(payload, ['descriptor_id']),
        firstString(payload, ['endpoint_id']),
        firstString(payload, ['kind']),
      ]
        .filter(Boolean)
        .join(' · ') || null
    case 'spine':
      return [
        firstString(payload, ['binding_kind']),
        firstString(payload, ['channel_or_session']),
        firstString(payload, ['outcome']),
        firstString(payload, ['kind']),
      ]
        .filter(Boolean)
        .join(' · ') || event.subsystem
    case 'misc':
      return event.subsystem
  }
}

function chronologyTitle(event: RawEvent, payload: Record<string, unknown>): string {
  if (isCortexOrganFamily(event.family)) {
    return organFamilyLabel(event.family)
  }

  return (
    firstString(
      payload,
      [
        'kind',
        'phase',
        'status',
        'change_mode',
        'descriptor_id',
        'sense_id',
        'act_id',
        'endpoint_id',
      ],
    ) ??
    event.family ??
    'event'
  )
}

function chronologySubtitle(event: RawEvent, payload: Record<string, unknown>): string | null {
  const fragments = [
    event.family,
    firstString(payload, ['descriptor_id']),
    firstString(payload, ['request_id']),
    firstString(payload, ['endpoint_id']),
    firstString(payload, ['outcome', 'terminal_outcome']),
  ].filter(Boolean)

  return fragments.length ? fragments.join(' · ') : null
}

function chronologySubtitleForInterval(interval: CortexOrganInterval): string | null {
  return [
    interval.requestId ? abbreviateId(interval.requestId) : null,
    interval.relatedEvents.length
      ? `${interval.relatedEvents.length} linked AI entr${interval.relatedEvents.length === 1 ? 'y' : 'ies'}`
      : null,
  ]
    .filter(Boolean)
    .join(' · ') || null
}

function intervalLaneSubtitle(interval: CortexOrganInterval): string | null {
  const startPayload = eventPayloadRecord(interval.startEvent)
  return [
    firstString(startPayload, ['route_or_backend']),
    interval.requestId ? abbreviateId(interval.requestId) : null,
  ]
    .filter(Boolean)
    .join(' · ') || null
}

function organFamilyLabel(family: string | null): string {
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

function intervalNarrative(interval: CortexOrganInterval): unknown {
  const startPayload = eventPayloadRecord(interval.startEvent)
  const endPayload = interval.endEvent ? eventPayloadRecord(interval.endEvent) : {}

  return {
    family: interval.family,
    organ: interval.label,
    request_id: interval.requestId,
    started_at: interval.startEvent.observedAt,
    ended_at: interval.endEvent?.observedAt ?? null,
    status:
      firstString(endPayload, ['status']) ??
      firstString(startPayload, ['status']) ??
      (interval.endEvent ? 'ok' : 'open'),
    route_or_backend: firstString(startPayload, ['route_or_backend']),
    input_payload: read(startPayload, ['input_payload']),
    output_payload: read(endPayload, ['output_payload']),
    error: read(endPayload, ['error']),
    ai_request_id:
      firstString(endPayload, ['ai_request_id']) ??
      firstString(startPayload, ['ai_request_id']),
    thread_id:
      firstString(endPayload, ['thread_id']) ??
      firstString(startPayload, ['thread_id']),
    turn_id: read(endPayload, ['turn_id']) ?? read(startPayload, ['turn_id']),
    related_ai: collectNarratives(interval.relatedEvents),
  }
}

function isCortexOrganFamily(family: string | null): boolean {
  return !!family && (CORTEX_ORGAN_FAMILIES as readonly string[]).includes(family)
}

function collectPayloadSingles(events: RawEvent[], family: string | null, keys: string[]): unknown[] {
  return events
    .filter((event) => family == null || event.family === family)
    .map((event) => read(eventPayloadRecord(event), keys))
    .filter((value) => value != null)
}

function firstPayloadValue(events: RawEvent[], family: string, keys: string[]): unknown | null {
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

function collectNarratives(events: RawEvent[], families?: string[]): unknown[] {
  return events
    .filter((event) => !families || families.includes(event.family ?? ''))
    .map(eventNarrative)
}

function eventNarrative(event: RawEvent): unknown {
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

function eventPayloadRecord(event: RawEvent): Record<string, unknown> {
  return toRecord(event.payload)
}

function firstString(record: Record<string, unknown>, keys: string[]): string | null {
  return readString(record, keys)
}

function abbreviateId(value: string): string {
  if (value.length <= 22) {
    return value
  }

  return `${value.slice(0, 10)}…${value.slice(-8)}`
}

function clamp01(value: number): number {
  return Math.min(0.98, Math.max(0.02, value))
}

function eventsForSubsystem(explicit: RawEvent[], rawEvents: RawEvent[], subsystem: string): RawEvent[] {
  if (explicit.length > 0) {
    return explicit
  }

  return rawEvents.filter((event) => event.subsystem === subsystem)
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
