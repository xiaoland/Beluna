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
    stringify(payload.organ_id),
    stringify(payload.thread_id),
    stringify(payload.turn_id),
    stringify(payload.request_id_when_present ?? payload.request_id),
    stringify(payload.sense_id_when_present ?? payload.sense_id),
    stringify(payload.act_id_when_present ?? payload.act_id),
    stringify(payload.endpoint_id_when_present ?? payload.endpoint_id),
    stringify(payload.adapter_id),
  ].filter(Boolean)
  const state = [
    stringify(payload.kind),
    stringify(payload.phase),
    stringify(payload.status),
    stringify(payload.kind_or_status),
    stringify(payload.transition_kind),
    stringify(payload.kind_or_transition),
    stringify(payload.kind_or_state),
    stringify(payload.outcome_when_present),
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
    'descriptor',
    'descriptor_id',
    'descriptorId',
    'signal_id',
    'signalId',
    'adapter_id',
    'adapterId',
    'organ_id',
    'organId',
    'thread_id',
    'threadId',
    'turn_id',
    'turnId',
    'endpoint_id',
    'endpointId',
    'request_id',
    'request_id_when_present',
    'requestId',
    'act_id',
    'actId',
    'sense_id',
    'senseId',
    'phase',
    'status',
    'kind',
    'kind_or_status',
    'kind_or_transition',
    'kind_or_state',
    'transition_kind',
    'binding_kind',
    'catalog_version',
    'backend_id',
    'model',
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
      {
        title: 'Gateway Requests',
        hint: 'LLM backend requests, retries, and usage for this tick.',
        items: detail.cortex.gatewayRequests,
      },
      {
        title: 'Committed Turns',
        hint: 'Committed AI-gateway turns with finish reasons and message payloads.',
        items: detail.cortex.gatewayTurns,
      },
      {
        title: 'Thread Snapshots',
        hint: 'Authoritative thread snapshots that connect turns without replay heuristics.',
        items: detail.cortex.gatewayThreads,
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
      title: 'Body Endpoints',
      hint: 'Endpoints connected or addressed.',
      items: detail.spine.bodyEndpoints,
    },
    {
      title: 'Dispatch',
      hint: 'Dispatch bindings and outcomes observed around this tick.',
      items: detail.spine.topologyEvents,
    },
  ]
}
