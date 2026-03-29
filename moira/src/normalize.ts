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

  return {
    runId,
    tick,
    chronology: buildChronology(rawEvents),
    cortex: {
      senses: collectPayloadArray(cortexEvents, 'cortex.tick', ['drained_senses']),
      proprioception: [
        ...collectPayloadSingles(cortexEvents, 'cortex.tick', ['physical_state_snapshot']),
        ...collectPayloadSingles(stemEvents, 'stem.proprioception', ['entries_or_keys']),
      ],
      primaryMessages: collectNarratives(cortexEvents, ['cortex.organ']),
      primaryTools: collectPathArray(cortexEvents, 'cortex.organ', ['output_payload_when_present', 'tool_calls']),
      gatewayRequests: collectNarratives(aiGatewayEvents.filter((event) => event.family === 'ai-gateway.request')),
      gatewayTurns: collectNarratives(aiGatewayEvents.filter((event) => event.family === 'ai-gateway.turn')),
      gatewayThreads: collectNarratives(aiGatewayEvents.filter((event) => event.family === 'ai-gateway.thread')),
      acts: [
        ...collectPayloadSingles(cortexEvents, 'cortex.tick', ['acts_payload_or_summary_when_present']),
        ...collectPayloadSingles(stemEvents, 'stem.dispatch', ['act_payload_when_present']),
        ...collectPayloadSingles(
          stemEvents.filter(
            (event) =>
              event.family === 'stem.signal' &&
              readString(eventPayloadRecord(event), ['direction']) === 'efferent',
          ),
          null,
          ['act_payload_when_present'],
        ),
      ],
      goalForest:
        firstPayloadValue(cortexEvents, 'cortex.goal-forest', ['snapshot_when_present']) ??
        firstPayloadValue(cortexEvents, 'cortex.goal-forest', ['patch_result_when_present']) ??
        firstPayloadValue(cortexEvents, 'cortex.tick', ['goal_forest_snapshot_ref_or_payload_when_present']),
    },
    stem: {
      afferentPathway: collectNarratives(
        stemEvents.filter(
          (event) =>
            (event.family === 'stem.signal' &&
              readString(eventPayloadRecord(event), ['direction']) === 'afferent') ||
            event.family === 'stem.afferent.rule',
        ),
      ),
      efferentPathway: collectNarratives([
        ...stemEvents.filter(
          (event) =>
            event.family === 'stem.signal' &&
            readString(eventPayloadRecord(event), ['direction']) === 'efferent',
        ),
        ...stemEvents.filter((event) => event.family === 'stem.dispatch'),
      ]),
      descriptorCatalog: collectNarratives(
        stemEvents.filter(
          (event) => event.family === 'stem.tick' || event.family === 'stem.descriptor.catalog',
        ),
      ),
      proprioception: collectNarratives(stemEvents.filter((event) => event.family === 'stem.proprioception')),
      afferentRules: collectNarratives(stemEvents.filter((event) => event.family === 'stem.afferent.rule')),
    },
    spine: {
      adapters: collectNarratives(spineEvents.filter((event) => event.family === 'spine.adapter')),
      bodyEndpoints: collectNarratives(spineEvents.filter((event) => event.family === 'spine.endpoint')),
      topologyEvents: collectNarratives(spineEvents.filter((event) => event.family === 'spine.dispatch')),
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

const LANE_TYPE_ORDER: Record<ChronologyLaneType, number> = {
  tick: 0,
  organ: 1,
  thread: 2,
  sense: 3,
  act: 4,
  endpoint: 5,
  adapter: 6,
  misc: 7,
}

function buildChronology(rawEvents: RawEvent[]) {
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
  const laneMap = new Map<string, Omit<ChronologyLane, 'entries' | 'eventCount'> & { entries: ChronologyEntry[]; firstEventIndex: number }>()

  ordered.forEach((event, eventIndex) => {
    const payload = eventPayloadRecord(event)
    const laneType = resolveLaneType(event, payload)
    const laneKey = resolveLaneKey(laneType, event, payload)
    const laneId = `${laneType}:${laneKey}`
    const laneLabel = resolveLaneLabel(laneType, laneKey, payload, event)
    const laneSubtitle = resolveLaneSubtitle(laneType, payload, event)
    const timeRatio = usesObservedTime
      ? ((parseObservedMs(event.observedAt) ?? minObservedMs ?? 0) - (minObservedMs ?? 0)) /
        ((maxObservedMs ?? 1) - (minObservedMs ?? 0))
      : null
    const sequenceRatio = eventIndex / sequenceDenominator
    const position = clamp01(0.03 + ((timeRatio ?? sequenceRatio) * 0.88 + sequenceRatio * 0.09))

    const entry: ChronologyEntry = {
      rawEventId: event.rawEventId,
      laneType,
      laneKey,
      title: chronologyTitle(event, payload),
      subtitle: chronologySubtitle(event, payload),
      family: event.family,
      observedAt: event.observedAt,
      severityText: event.severityText,
      eventIndex,
      position,
      endPosition: position,
      event,
    }

    const existing = laneMap.get(laneId)
    if (existing) {
      existing.entries.push(entry)
      return
    }

    laneMap.set(laneId, {
      id: laneId,
      laneType,
      laneKey,
      label: laneLabel,
      subtitle: laneSubtitle,
      entries: [entry],
      firstEventIndex: eventIndex,
    })
  })

  const lanes = [...laneMap.values()]
    .map((lane) => {
      const entries = lane.entries.sort((left, right) => {
        if (left.position !== right.position) {
          return left.position - right.position
        }
        return left.eventIndex - right.eventIndex
      })

      const positioned = entries.map((entry, index) => {
        const next = entries[index + 1]
        const fallbackSpan = next ? 0.018 : 0.07
        const softGap = next ? 0.006 : 0
        const endPosition = next
          ? clamp01(Math.max(entry.position + fallbackSpan, next.position - softGap))
          : clamp01(entry.position + fallbackSpan)
        return {
          ...entry,
          endPosition: Math.max(endPosition, Math.min(0.98, entry.position + 0.028)),
        }
      })

      return {
        id: lane.id,
        laneType: lane.laneType,
        laneKey: lane.laneKey,
        label: lane.label,
        subtitle: lane.subtitle,
        eventCount: positioned.length,
        entries: positioned,
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
    firstObservedAt,
    lastObservedAt,
    usesObservedTime,
  }
}

function collectPayloadArray(events: RawEvent[], family: string | null, keys: string[]): unknown[] {
  return events
    .filter((event) => family == null || event.family === family)
    .flatMap((event) => {
      const value = read(eventPayloadRecord(event), keys)
      if (value == null) {
        return []
      }
      return Array.isArray(value) ? value : [value]
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
  if (family === 'cortex.tick' || family === 'stem.tick') {
    return 'tick'
  }
  if (family === 'cortex.organ') {
    return 'organ'
  }
  if (family.startsWith('ai-gateway.')) {
    return 'thread'
  }
  if (family === 'stem.signal') {
    return readString(payload, ['direction']) === 'afferent' ? 'sense' : 'act'
  }
  if (family === 'stem.dispatch' || family === 'spine.dispatch') {
    return 'act'
  }
  if (family === 'spine.endpoint') {
    return 'endpoint'
  }
  if (family === 'spine.adapter') {
    return 'adapter'
  }
  return 'misc'
}

function resolveLaneKey(
  laneType: ChronologyLaneType,
  event: RawEvent,
  payload: Record<string, unknown>,
): string {
  const family = event.family ?? 'unknown'

  switch (laneType) {
    case 'tick':
      return firstString(payload, ['span_id']) ?? `${family}:${event.tick ?? '0'}`
    case 'organ':
      return firstString(payload, ['organ_id', 'request_id', 'span_id']) ?? event.rawEventId
    case 'thread':
      return firstString(
        payload,
        ['thread_id', 'turn_id', 'request_id', 'span_id'],
      ) ?? event.rawEventId
    case 'sense':
      return firstString(
        payload,
        ['sense_id_when_present', 'sense_id', 'endpoint_id_when_present', 'endpoint_id', 'span_id'],
      ) ?? event.rawEventId
    case 'act':
      return firstString(
        payload,
        ['act_id_when_present', 'act_id', 'endpoint_id_when_present', 'endpoint_id', 'span_id'],
      ) ?? event.rawEventId
    case 'endpoint':
      return firstString(
        payload,
        ['endpoint_id', 'endpoint_id_when_present', 'adapter_id_when_present', 'adapter_id', 'span_id'],
      ) ?? event.rawEventId
    case 'adapter':
      return firstString(payload, ['adapter_id', 'span_id']) ?? event.rawEventId
    case 'misc':
      return firstString(payload, ['span_id']) ?? event.rawEventId
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
      return event.family === 'stem.tick' ? 'Stem Rhythm' : `Tick ${event.tick ?? laneKey}`
    case 'organ':
      return firstString(payload, ['organ_id']) ?? abbreviateId(laneKey)
    case 'thread':
      return firstString(payload, ['thread_id']) ?? `turn ${firstString(payload, ['turn_id']) ?? abbreviateId(laneKey)}`
    case 'sense':
      return firstString(payload, ['sense_id_when_present', 'sense_id']) ?? abbreviateId(laneKey)
    case 'act':
      return firstString(payload, ['act_id_when_present', 'act_id']) ?? abbreviateId(laneKey)
    case 'endpoint':
      return firstString(payload, ['endpoint_id', 'endpoint_id_when_present']) ?? abbreviateId(laneKey)
    case 'adapter':
      return firstString(payload, ['adapter_id']) ?? abbreviateId(laneKey)
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
      return firstString(payload, ['kind_or_status', 'status'])
    case 'organ':
      return firstString(payload, ['route_or_backend_when_present', 'request_id'])
    case 'thread':
      return [
        firstString(payload, ['backend_id']),
        firstString(payload, ['model']),
      ]
        .filter(Boolean)
        .join(' · ') || firstString(payload, ['request_id_when_present', 'request_id'])
    case 'sense':
    case 'act':
      return [
        firstString(payload, ['descriptor_id', 'descriptor_id_when_present']),
        firstString(payload, ['endpoint_id', 'endpoint_id_when_present']),
      ]
        .filter(Boolean)
        .join(' · ')
    case 'endpoint':
      return firstString(payload, ['adapter_id_when_present', 'channel_or_session_when_present'])
    case 'adapter':
      return firstString(payload, ['adapter_type'])
    case 'misc':
      return event.subsystem
  }
}

function chronologyTitle(event: RawEvent, payload: Record<string, unknown>): string {
  return (
    firstString(
      payload,
      [
        'phase',
        'kind',
        'kind_or_status',
        'kind_or_transition',
        'kind_or_state',
        'transition_kind',
        'status',
      ],
    ) ??
    event.family ??
    'event'
  )
}

function chronologySubtitle(event: RawEvent, payload: Record<string, unknown>): string | null {
  const fragments = [
    event.family,
    firstString(payload, ['descriptor_id', 'descriptor_id_when_present']),
    firstString(payload, ['backend_id']),
    firstString(payload, ['model']),
    firstString(payload, ['request_id_when_present', 'request_id']),
  ].filter(Boolean)

  return fragments.length ? fragments.join(' · ') : null
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

function collectPathArray(events: RawEvent[], family: string | null, path: string[]): unknown[] {
  return events
    .filter((event) => family == null || event.family === family)
    .flatMap((event) => {
      const value = readPath(event.payload, path)
      if (value == null) {
        return []
      }
      return Array.isArray(value) ? value : [value]
    })
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

function readPath(value: unknown, path: string[]): unknown | null {
  let current: unknown = value

  for (const segment of path) {
    const record = toRecord(current)
    if (!(segment in record)) {
      return null
    }
    current = record[segment]
  }

  return current ?? null
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
