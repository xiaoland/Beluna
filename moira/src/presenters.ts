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
  const payload = toRecord(event.payload)
  const identity = [
    stringify(payload.request_id),
    stringify(payload.ai_request_id),
    stringify(payload.thread_id),
    stringify(payload.turn_id),
    stringify(payload.sense_id),
    stringify(payload.act_id),
    stringify(payload.endpoint_id),
    stringify(payload.adapter_id),
  ].filter(Boolean)
  const state = [
    stringify(payload.kind),
    stringify(payload.phase),
    stringify(payload.status),
    stringify(payload.change_mode),
    stringify(payload.outcome),
    stringify(payload.terminal_outcome),
  ].filter(Boolean)

  return (
    event.messageText ||
    [event.family, state.join(' / '), identity.join(' · ')].filter(Boolean).join(' · ') ||
    [event.subsystem, event.family, event.target].filter(Boolean).join(' / ') ||
    'Raw event'
  )
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
  const nestedMessage = toRecord(record.message)
  if (Object.keys(nestedMessage).length > 0) {
    return [
      stringify(record.turn_id) ? `turn ${stringify(record.turn_id)}` : null,
      stringify(nestedMessage.kind),
      stringify(nestedMessage.id),
    ]
      .filter(Boolean)
      .join(' · ')
  }

  const candidates = [
    'label',
    'name',
    'message',
    'message_text',
    'messageText',
    'organ',
    'family',
    'descriptor',
    'descriptor_id',
    'descriptorId',
    'catalog_version',
    'change_mode',
    'adapter_id',
    'adapterId',
    'request_id',
    'requestId',
    'ai_request_id',
    'thread_id',
    'threadId',
    'turn_id',
    'turnId',
    'endpoint_id',
    'endpointId',
    'act_id',
    'actId',
    'sense_id',
    'senseId',
    'phase',
    'status',
    'kind',
    'binding_kind',
    'model',
    'backend_id',
    'tick',
    'outcome',
    'terminal_outcome',
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
      {
        title: 'Organ Intervals',
        hint: 'Paired Cortex boundary records with related AI activity when present.',
        items: detail.cortex.organs,
      },
      {
        title: 'Goal Forest Events',
        hint: 'Snapshots and mutation records emitted inside this tick.',
        items: detail.cortex.goalForestEvents,
      },
      {
        title: 'Latest Goal Forest',
        hint: 'Most recent snapshot or mutation result available for this tick.',
        items: [],
        single: detail.cortex.goalForest,
      },
    ]
  }

  if (tab === 'stem') {
    return [
      {
        title: 'Tick Anchor',
        hint: 'Canonical tick-grant records owned by Stem.',
        items: detail.stem.tickAnchor,
      },
      {
        title: 'Afferent Pathway',
        hint: 'Incoming neural signals entering Core.',
        items: detail.stem.afferent,
      },
      {
        title: 'Efferent Pathway',
        hint: 'Outgoing neural signals and terminal results owned by Stem.',
        items: detail.stem.efferent,
      },
      {
        title: 'Neural Signal Catalog',
        hint: 'Descriptor catalog commits visible during this tick.',
        items: detail.stem.nsCatalog,
      },
      {
        title: 'Proprioception',
        hint: 'Physical-state mutations and retained status patches.',
        items: detail.stem.proprioception,
      },
      {
        title: 'Afferent Rules',
        hint: 'Deferral-rule lifecycle observed inside Stem.',
        items: detail.stem.afferentRules,
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
      title: 'Endpoints',
      hint: 'Endpoint lifecycle and registration records.',
      items: detail.spine.endpoints,
    },
    {
      title: 'Sense Ingress',
      hint: 'Senses that Spine accepted from body endpoints.',
      items: detail.spine.senses,
    },
    {
      title: 'Act Routing',
      hint: 'Act bindings and terminal delivery outcomes owned by Spine.',
      items: detail.spine.acts,
    },
  ]
}
