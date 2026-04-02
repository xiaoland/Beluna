import type { RuntimeStatusPayload } from '@/bridge/contracts/atropos'
import type { RuntimeStatus, SupervisionPhase } from './models'

const VALID_PHASES: SupervisionPhase[] = ['idle', 'waking', 'running', 'stopping', 'terminated']

export function normalizeRuntimeStatus(payload: RuntimeStatusPayload): RuntimeStatus {
  const phase = VALID_PHASES.includes(payload.phase) ? payload.phase : 'idle'

  return {
    phase,
    buildId: normalizeString(payload.buildId),
    executablePath: normalizeString(payload.executablePath),
    workingDir: normalizeString(payload.workingDir),
    profilePath: normalizeString(payload.profilePath),
    pid: typeof payload.pid === 'number' ? payload.pid : null,
    terminalReason: normalizeString(payload.terminalReason),
  }
}

function normalizeString(value: string | null | undefined): string | null {
  return typeof value === 'string' && value.trim().length > 0 ? value : null
}
