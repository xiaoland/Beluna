import { invoke } from '@tauri-apps/api/core'

import type { RuntimeStatusPayload } from '@/bridge/contracts/atropos'
import type { WakeInputRequestPayload } from '@/bridge/contracts/clotho'

export async function fetchRuntimeStatus(): Promise<RuntimeStatusPayload> {
  return invoke<RuntimeStatusPayload>('runtime_status')
}

export async function wakeCore(request: WakeInputRequestPayload): Promise<RuntimeStatusPayload> {
  return invoke<RuntimeStatusPayload>('wake', { request })
}

export async function stopCore(): Promise<RuntimeStatusPayload> {
  return invoke<RuntimeStatusPayload>('stop')
}

export async function forceKillCore(): Promise<RuntimeStatusPayload> {
  return invoke<RuntimeStatusPayload>('force_kill')
}
