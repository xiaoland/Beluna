export type SupervisionPhase = 'idle' | 'waking' | 'running' | 'stopping' | 'terminated'

export interface RuntimeStatus {
  phase: SupervisionPhase
  buildId: string | null
  executablePath: string | null
  workingDir: string | null
  profilePath: string | null
  pid: number | null
  terminalReason: string | null
}
