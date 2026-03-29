export type DetailTab = 'chronology' | 'cortex' | 'stem' | 'spine' | 'raw'

export type ChronologyLaneType = 'tick' | 'organ' | 'thread' | 'sense' | 'act' | 'endpoint' | 'adapter' | 'misc'

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

export interface ChronologyEntry {
  rawEventId: string
  laneType: ChronologyLaneType
  laneKey: string
  title: string
  subtitle: string | null
  family: string | null
  observedAt: string | null
  severityText: string | null
  eventIndex: number
  position: number
  endPosition: number
  event: RawEvent
}

export interface ChronologyLane {
  id: string
  laneType: ChronologyLaneType
  laneKey: string
  label: string
  subtitle: string | null
  eventCount: number
  entries: ChronologyEntry[]
}

export interface TickChronology {
  lanes: ChronologyLane[]
  eventCount: number
  firstObservedAt: string | null
  lastObservedAt: string | null
  usesObservedTime: boolean
}

export interface CortexDetail {
  senses: unknown[]
  proprioception: unknown[]
  primaryMessages: unknown[]
  primaryTools: unknown[]
  gatewayRequests: unknown[]
  gatewayTurns: unknown[]
  gatewayThreads: unknown[]
  acts: unknown[]
  goalForest: unknown | null
}

export interface StemDetail {
  afferentPathway: unknown[]
  efferentPathway: unknown[]
  descriptorCatalog: unknown[]
  proprioception: unknown[]
  afferentRules: unknown[]
}

export interface SpineDetail {
  adapters: unknown[]
  bodyEndpoints: unknown[]
  topologyEvents: unknown[]
}

export interface TickDetail {
  runId: string
  tick: string
  chronology: TickChronology
  cortex: CortexDetail
  stem: StemDetail
  spine: SpineDetail
  rawEvents: RawEvent[]
}
