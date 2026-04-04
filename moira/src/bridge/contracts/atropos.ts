export type SupervisionPhasePayload = 'idle' | 'waking' | 'running' | 'stopping' | 'terminated'

export interface RuntimeStatusPayload {
  phase: SupervisionPhasePayload
  targetLabel?: string | null
  executablePath?: string | null
  workingDir?: string | null
  profilePath?: string | null
  pid?: number | null
  terminalReason?: string | null
}
