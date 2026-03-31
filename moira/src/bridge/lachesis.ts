import { invoke } from '@tauri-apps/api/core'
import type {
  ListTicksArgs,
  ReceiverStatusPayload,
  RunSummaryPayload,
  TickDetailArgs,
  TickDetailPayload,
  TickSummaryPayload,
} from '@/bridge/contracts/lachesis'

export async function fetchReceiverStatus(): Promise<ReceiverStatusPayload> {
  return invoke<ReceiverStatusPayload>('receiver_status')
}

export async function fetchWakeSessions(): Promise<RunSummaryPayload[]> {
  return invoke<RunSummaryPayload[]>('list_runs')
}

export async function fetchTicks(runId: string): Promise<TickSummaryPayload[]> {
  return invoke<TickSummaryPayload[]>('list_ticks', toRunArgs(runId))
}

export async function fetchTickDetail(runId: string, tick: number): Promise<TickDetailPayload> {
  return invoke<TickDetailPayload>('tick_detail', toTickDetailArgs(runId, tick))
}

function toRunArgs(runId: string): ListTicksArgs {
  return {
    runId,
    run_id: runId,
  }
}

function toTickDetailArgs(runId: string, tick: number): TickDetailArgs {
  return {
    ...toRunArgs(runId),
    tick,
  }
}
