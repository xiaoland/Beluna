export function prettyCount(value: number | null): string {
  return value == null ? '—' : Intl.NumberFormat().format(value)
}

export function formatWhen(value: string | null): string {
  if (!value) {
    return '—'
  }

  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString()
}

export function stateTone(state: string | null): 'ok' | 'warn' | 'err' | 'idle' {
  const normalized = (state ?? '').toLowerCase()

  if (['awake', 'ready', 'listening', 'healthy', 'running', 'active'].some((token) => normalized.includes(token))) {
    return 'ok'
  }

  if (['awakening', 'warn', 'degraded', 'stopping', 'waiting'].some((token) => normalized.includes(token))) {
    return 'warn'
  }

  if (['error', 'failed', 'fault', 'faulted', 'stopped', 'terminal'].some((token) => normalized.includes(token))) {
    return 'err'
  }

  return 'idle'
}
