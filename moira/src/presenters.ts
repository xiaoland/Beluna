import type { RawEvent, TickDetail } from './types'
import { stringify, toRecord } from './coerce'

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

export function rawEventHeadline(event: RawEvent): string {
  return event.messageText || [event.subsystem, event.family, event.target].filter(Boolean).join(' / ') || 'Raw event'
}

export function summarizeEntry(value: unknown): string {
  if (typeof value === 'string') {
    return value
  }

  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value)
  }

  if (Array.isArray(value)) {
    return `${value.length} item${value.length === 1 ? '' : 's'}`
  }

  const record = toRecord(value)
  const candidates = [
    'label',
    'name',
    'message',
    'message_text',
    'messageText',
    'descriptor',
    'descriptor_id',
    'descriptorId',
    'signal_id',
    'signalId',
    'adapter_id',
    'adapterId',
    'endpoint_id',
    'endpointId',
    'request_id',
    'requestId',
    'act_id',
    'actId',
    'sense_id',
    'senseId',
    'tick',
    'outcome',
    'state_transition',
    'id',
  ]

  for (const key of candidates) {
    const candidate = stringify(record[key])
    if (candidate) {
      return candidate
    }
  }

  const keys = Object.keys(record).slice(0, 3)
  return keys.length ? keys.join(' · ') : 'Structured entry'
}

export function narrativeSections(
  detail: TickDetail,
  tab: 'cortex' | 'stem' | 'spine',
): Array<{ title: string; hint: string; items: unknown[]; single?: unknown | null }> {
  if (tab === 'cortex') {
    return [
      { title: 'Senses', hint: 'Inputs noticed during this tick.', items: detail.cortex.senses },
      {
        title: 'Proprioception',
        hint: 'Core self-observation and bodily state.',
        items: detail.cortex.proprioception,
      },
      {
        title: 'Primary Messages',
        hint: 'Message-level cognition routed into the tick.',
        items: detail.cortex.primaryMessages,
      },
      {
        title: 'Primary Tools',
        hint: 'Tool selections or calls anchored to this tick.',
        items: detail.cortex.primaryTools,
      },
      { title: 'Acts', hint: 'Actions chosen or attempted.', items: detail.cortex.acts },
      {
        title: 'Goal Forest Snapshot',
        hint: 'Snapshot reference or payload available for later compare.',
        items: [],
        single: detail.cortex.goalForest,
      },
    ]
  }

  if (tab === 'stem') {
    return [
      {
        title: 'Afferent Pathway',
        hint: 'Incoming neural signals entering Core.',
        items: detail.stem.afferentPathway,
      },
      {
        title: 'Efferent Pathway',
        hint: 'Outgoing neural signals leaving Core.',
        items: detail.stem.efferentPathway,
      },
      {
        title: 'Descriptor Catalog',
        hint: 'Descriptor records observed for this tick.',
        items: detail.stem.descriptorCatalog,
      },
    ]
  }

  return [
    {
      title: 'Adapters',
      hint: 'Adapters engaged during the selected tick.',
      items: detail.spine.adapters,
    },
    {
      title: 'Body Endpoints',
      hint: 'Endpoints connected or addressed.',
      items: detail.spine.bodyEndpoints,
    },
    {
      title: 'Dispatch Outcomes',
      hint: 'Terminal dispatch outcomes observed around this tick.',
      items: detail.spine.topologyEvents,
    },
  ]
}
