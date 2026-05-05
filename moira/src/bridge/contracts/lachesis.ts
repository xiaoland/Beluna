export interface ReceiverStatusPayload {
  endpoint: string
  wakeState: string
  dbPath: string
  lastBatchAt: string | null
  lastError: string | null
  rawEventCount: number
  wakeCount: number
  tickCount: number
}

export interface RunSummaryPayload {
  runId: string
  firstSeenAt: string
  lastSeenAt: string
  eventCount: number
  warningCount: number
  errorCount: number
  latestTick: number | null
}

export interface TickSummaryPayload {
  runId: string
  tick: number
  traceId: string | null
  firstSeenAt: string
  lastSeenAt: string
  eventCount: number
  warningCount: number
  errorCount: number
  cortexHandled: boolean
}

export interface EventRecordPayload {
  rawEventId: string
  receivedAt: string
  observedAt: string
  severityText: string
  severityNumber?: number | null
  recordKind?: string | null
  scopeName: string | null
  eventName: string | null
  traceId: string | null
  spanId: string | null
  traceFlags: number | null
  target: string | null
  family: string | null
  subsystem: string | null
  runId: string | null
  tick: number | null
  messageText: string | null
  attributes: unknown
  body: unknown
  resource: unknown
  scope: unknown
}

export interface TickDetailPayload {
  summary: TickSummaryPayload
  cortex: EventRecordPayload[]
  stem: EventRecordPayload[]
  spine: EventRecordPayload[]
  raw: EventRecordPayload[]
}

export interface LachesisUpdatedPayload {
  touchedRunIds: string[]
  lastBatchAt: string
}

export type ListTicksArgs = Record<string, unknown> & {
  runId: string
  run_id: string
}

export type TickDetailArgs = ListTicksArgs & {
  tick: number
}
