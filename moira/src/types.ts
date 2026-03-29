export type DetailTab = 'cortex' | 'stem' | 'spine' | 'raw'

export interface ReceiverStatus {
  state: string
  storagePath: string | null
  receiverBind: string | null
  lastIngestAt: string | null
  rawEventCount: number | null
  runCount: number | null
  tickCount: number | null
  note: string | null
}

export interface WakeSessionSummary {
  runId: string
  firstSeenAt: string | null
  lastSeenAt: string | null
  eventCount: number
  warningCount: number
  errorCount: number
  latestTick: string | null
  state: string | null
}

export interface TickSummary {
  runId: string
  tick: string
  firstSeenAt: string | null
  lastSeenAt: string | null
  eventCount: number
  warningCount: number
  errorCount: number
}

export interface RawEvent {
  rawEventId: string
  receivedAt: string | null
  observedAt: string | null
  severityText: string | null
  severityNumber: number | null
  target: string | null
  family: string | null
  subsystem: string | null
  runId: string | null
  tick: string | null
  messageText: string | null
  payload: unknown | null
  body: unknown
  attributes: Record<string, unknown>
  resource: Record<string, unknown>
  scope: Record<string, unknown>
}

export interface CortexDetail {
  senses: unknown[]
  proprioception: unknown[]
  primaryMessages: unknown[]
  primaryTools: unknown[]
  acts: unknown[]
  goalForest: unknown | null
}

export interface StemDetail {
  afferentPathway: unknown[]
  efferentPathway: unknown[]
  descriptorCatalog: unknown[]
}

export interface SpineDetail {
  adapters: unknown[]
  bodyEndpoints: unknown[]
  topologyEvents: unknown[]
}

export interface TickDetail {
  runId: string
  tick: string
  cortex: CortexDetail
  stem: StemDetail
  spine: SpineDetail
  rawEvents: RawEvent[]
}
