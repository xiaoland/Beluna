import type { ReceiverStatusPayload } from '@/bridge/contracts/lachesis'
import type { ReceiverStatus } from './models'

export function normalizeReceiverStatus(value: ReceiverStatusPayload): ReceiverStatus {
  return {
    state: value.wakeState ?? 'unknown',
    storagePath: value.dbPath,
    receiverBind: value.endpoint,
    lastIngestAt: value.lastBatchAt,
    rawEventCount: value.rawEventCount,
    runCount: value.wakeCount,
    tickCount: value.tickCount,
    note: value.lastError,
  }
}
