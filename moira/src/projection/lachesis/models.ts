export type ChronologyLaneType = 'tick' | 'cortex' | 'afferent' | 'efferent' | 'spine' | 'misc'
export type ChronologyEntryType = 'point' | 'interval'

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
  latestTick: number | null
}

export interface TickSummary {
  runId: string
  tick: number
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
  tick: number | null
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
  entryType: ChronologyEntryType
  title: string
  subtitle: string | null
  family: string | null
  observedAt: string | null
  severityText: string | null
  eventIndex: number
  position: number
  endPosition: number
  event: RawEvent
  sourceEvents: RawEvent[]
  relatedEvents: RawEvent[]
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
  organs: unknown[]
  goalForestEvents: unknown[]
  goalForest: unknown | null
}

export interface StemDetail {
  tickAnchor: unknown[]
  afferent: unknown[]
  efferent: unknown[]
  nsCatalog: unknown[]
  proprioception: unknown[]
  afferentRules: unknown[]
}

export interface SpineDetail {
  adapters: unknown[]
  endpoints: unknown[]
  senses: unknown[]
  acts: unknown[]
}

export interface TickDetail {
  runId: string
  tick: number
  chronology: TickChronology
  cortex: CortexDetail
  stem: StemDetail
  spine: SpineDetail
  rawEvents: RawEvent[]
}

export interface NarrativeSection {
  title: string
  hint: string
  items: unknown[]
  single?: unknown | null
}
