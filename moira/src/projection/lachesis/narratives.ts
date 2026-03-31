import { read, readString } from '@/coerce'

import type { CortexOrganInterval } from './chronology'
import type { NarrativeSection, TickDetail } from './models'
import { collectNarratives, eventPayloadRecord } from './raw-events'

export function intervalNarrative(interval: CortexOrganInterval): unknown {
  const startPayload = eventPayloadRecord(interval.startEvent)
  const endPayload = interval.endEvent ? eventPayloadRecord(interval.endEvent) : {}

  return {
    family: interval.family,
    organ: interval.label,
    request_id: interval.requestId,
    started_at: interval.startEvent.observedAt,
    ended_at: interval.endEvent?.observedAt ?? null,
    status:
      readString(endPayload, ['status']) ??
      readString(startPayload, ['status']) ??
      (interval.endEvent ? 'ok' : 'open'),
    route_or_backend: readString(startPayload, ['route_or_backend']),
    input_payload: read(startPayload, ['input_payload']),
    output_payload: read(endPayload, ['output_payload']),
    error: read(endPayload, ['error']),
    ai_request_id:
      readString(endPayload, ['ai_request_id']) ??
      readString(startPayload, ['ai_request_id']),
    thread_id:
      readString(endPayload, ['thread_id']) ??
      readString(startPayload, ['thread_id']),
    turn_id: read(endPayload, ['turn_id']) ?? read(startPayload, ['turn_id']),
    related_ai: collectNarratives(interval.relatedEvents),
  }
}

export function narrativeSections(
  detail: TickDetail,
  tab: 'cortex' | 'stem' | 'spine',
): NarrativeSection[] {
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
