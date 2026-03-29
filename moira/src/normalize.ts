import type { RawEvent, ReceiverStatus, TickDetail, TickSummary, WakeSessionSummary } from './types'
import {
  compareDateDesc,
  cryptoRandomId,
  matchesKeywords,
  parseMaybeJson,
  read,
  readArray,
  readNumber,
  readString,
  stringify,
  toArray,
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
  const cortexEvents = readArray(record, ['cortex']).map((item) => normalizeRawEvent(item, runId, tick))
  const stemEvents = readArray(record, ['stem']).map((item) => normalizeRawEvent(item, runId, tick))
  const spineEvents = readArray(record, ['spine']).map((item) => normalizeRawEvent(item, runId, tick))

  return {
    runId,
    tick,
    cortex: {
      senses: collectPayloadArray(cortexEvents, 'cortex.tick', ['senses_summary']),
      proprioception: collectPayloadSingles(cortexEvents, 'cortex.tick', ['proprioception_snapshot_or_ref']),
      primaryMessages: collectNarratives(cortexEvents, ['cortex.organ.request', 'cortex.organ.response']),
      primaryTools: collectPayloadArray(cortexEvents, 'cortex.organ.response', ['tool_summary']),
      acts: [
        ...collectPayloadArray(cortexEvents, 'cortex.tick', ['acts_summary']),
        ...collectPayloadArray(cortexEvents, 'cortex.organ.response', ['act_summary']),
      ],
      goalForest:
        firstPayloadValue(cortexEvents, 'cortex.goal_forest.snapshot', ['snapshot_or_ref']) ??
        firstPayloadValue(cortexEvents, 'cortex.tick', ['goal_forest_ref']),
    },
    stem: {
      afferentPathway: collectNarratives(
        stemEvents.filter(
          (event) =>
            event.family === 'stem.signal.transition' &&
            eventPayloadRecord(event).direction === 'afferent',
        ),
      ),
      efferentPathway: collectNarratives([
        ...stemEvents.filter(
          (event) =>
            event.family === 'stem.signal.transition' &&
            eventPayloadRecord(event).direction === 'efferent',
        ),
        ...stemEvents.filter((event) => event.family === 'stem.dispatch.transition'),
      ]),
      descriptorCatalog: collectNarratives(
        stemEvents.filter((event) => event.family === 'stem.descriptor.catalog'),
      ),
    },
    spine: {
      adapters: collectNarratives(spineEvents.filter((event) => event.family === 'spine.adapter.lifecycle')),
      bodyEndpoints: collectNarratives(
        spineEvents.filter((event) => event.family === 'spine.endpoint.lifecycle'),
      ),
      topologyEvents: collectNarratives(
        spineEvents.filter((event) => event.family === 'spine.dispatch.outcome'),
      ),
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
      (typeof body === 'string' ? body : null),
    payload: parseEventPayload(attributes, body),
    body,
    attributes,
    resource: toRecord(parseMaybeJson(read(record, ['resource', 'resourceJson', 'resource_json']))),
    scope: toRecord(parseMaybeJson(read(record, ['scope', 'scopeJson', 'scope_json']))),
  }
}

function collectPayloadArray(events: RawEvent[], family: string, keys: string[]): unknown[] {
  return events
    .filter((event) => event.family === family)
    .flatMap((event) => {
      const value = read(eventPayloadRecord(event), keys)
      return Array.isArray(value) ? value : []
    })
}

function collectPayloadSingles(events: RawEvent[], family: string, keys: string[]): unknown[] {
  return events
    .filter((event) => event.family === family)
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
