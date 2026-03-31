import type { RunSummaryPayload } from '@/bridge/contracts/lachesis'
import { compareDateDesc } from '@/coerce'
import type { WakeSessionSummary } from './models'

export function normalizeWakeSession(value: RunSummaryPayload): WakeSessionSummary {
  return {
    runId: value.runId,
    firstSeenAt: value.firstSeenAt,
    lastSeenAt: value.lastSeenAt,
    eventCount: value.eventCount,
    warningCount: value.warningCount,
    errorCount: value.errorCount,
    latestTick: value.latestTick,
  }
}

export function compareWakeSessions(left: WakeSessionSummary, right: WakeSessionSummary): number {
  return compareDateDesc(left.lastSeenAt, right.lastSeenAt)
}
