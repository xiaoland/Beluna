import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { LachesisUpdatedPayload } from '@/bridge/contracts/lachesis'

export const LACHESIS_UPDATED_EVENT = 'lachesis-updated'

export function listenLachesisUpdated(
  handler: (pulse: LachesisUpdatedPayload) => void,
): Promise<UnlistenFn> {
  return listen<LachesisUpdatedPayload>(LACHESIS_UPDATED_EVENT, (event) => handler(event.payload))
}
