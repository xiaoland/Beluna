import { invoke } from '@tauri-apps/api/core'
import type { ReceiverStatus, TickDetail, TickSummary, WakeSessionSummary } from './types'
import { toArray } from './coerce'
import {
  compareTicks,
  compareWakeSessions,
  normalizeReceiverStatus,
  normalizeTickDetail,
  normalizeTickSummary,
  normalizeWakeSession,
} from './normalize'

export const LACHESIS_UPDATED_EVENT = 'lachesis-updated'

export function hasTauriBridge(): boolean {
  return typeof window !== 'undefined' && Boolean(window.__TAURI__ ?? window.__TAURI_INTERNALS__)
}

export async function getReceiverStatus(): Promise<ReceiverStatus> {
  const payload = await invoke<unknown>('receiver_status')
  return normalizeReceiverStatus(payload)
}

export async function getWakeSessions(): Promise<WakeSessionSummary[]> {
  const payload = await invoke<unknown>('list_runs')
  return toArray(payload).map(normalizeWakeSession).sort(compareWakeSessions)
}

export async function getTicks(runId: string): Promise<TickSummary[]> {
  const payload = await invoke<unknown>('list_ticks', {
    runId,
    run_id: runId,
  })

  return toArray(payload).map(normalizeTickSummary).sort(compareTicks)
}

export async function getTickDetail(runId: string, tick: string): Promise<TickDetail> {
  const tickNumber = Number(tick)
  if (!Number.isFinite(tickNumber)) {
    throw new Error(`Invalid tick: ${tick}`)
  }

  const payload = await invoke<unknown>('tick_detail', {
    runId,
    run_id: runId,
    tick: tickNumber,
  })

  return normalizeTickDetail(payload)
}
