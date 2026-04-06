import type { TickDetailPayload, TickSummaryPayload } from '@/bridge/contracts/lachesis'

import { buildChronology, buildCortexOrganIntervals } from './chronology'
import { isAiFamily } from './families'
import { organFamilyLabel } from './labels'
import type { TickDetail, TickSummary } from './models'
import { intervalNarrative } from './narratives'
import {
  collectNarratives,
  compareTicksByObservedAt,
  eventsForFamilies,
  eventsForSubsystem,
  firstPayloadValue,
  normalizeRawEvent,
} from './raw-events'

export function normalizeTickSummary(value: TickSummaryPayload): TickSummary {
  return {
    runId: value.runId,
    tick: value.tick,
    firstSeenAt: value.firstSeenAt,
    lastSeenAt: value.lastSeenAt,
    eventCount: value.eventCount,
    warningCount: value.warningCount,
    errorCount: value.errorCount,
    cortexHandled: value.cortexHandled === true,
  }
}

export function compareTicks(left: TickSummary, right: TickSummary): number {
  const leftNumber = Number(left.tick)
  const rightNumber = Number(right.tick)

  if (Number.isFinite(leftNumber) && Number.isFinite(rightNumber) && leftNumber !== rightNumber) {
    return rightNumber - leftNumber
  }

  return compareTicksByObservedAt(left.lastSeenAt, right.lastSeenAt)
}

export function normalizeTickDetail(value: TickDetailPayload): TickDetail {
  const runId = value.summary.runId
  const tick = value.summary.tick
  const rawEvents = value.raw.map((item) => normalizeRawEvent(item, runId, tick))
  const aiEvents = eventsForFamilies(
    [],
    rawEvents,
    isAiFamily,
  )
  const cortexEvents = eventsForSubsystem(
    value.cortex.map((item) => normalizeRawEvent(item, runId, tick)),
    rawEvents,
    'cortex',
  )
  const stemEvents = eventsForSubsystem(
    value.stem.map((item) => normalizeRawEvent(item, runId, tick)),
    rawEvents,
    'stem',
  )
  const spineEvents = eventsForSubsystem(
    value.spine.map((item) => normalizeRawEvent(item, runId, tick)),
    rawEvents,
    'spine',
  )
  const cortexOrganIntervals = buildCortexOrganIntervals(rawEvents, cortexEvents, aiEvents, organFamilyLabel)

  return {
    runId,
    tick,
    cortexHandled: value.summary.cortexHandled === true,
    chronology: buildChronology(rawEvents, cortexOrganIntervals),
    cortex: {
      organs: cortexOrganIntervals.map(intervalNarrative),
      goalForestEvents: collectNarratives(cortexEvents.filter((event) => event.family === 'cortex.goal-forest')),
      goalForest:
        firstPayloadValue(cortexEvents, 'cortex.goal-forest', ['snapshot']) ??
        firstPayloadValue(cortexEvents, 'cortex.goal-forest', ['mutation_result']),
    },
    stem: {
      tickAnchor: collectNarratives(stemEvents.filter((event) => event.family === 'stem.tick')),
      afferent: collectNarratives(stemEvents.filter((event) => event.family === 'stem.afferent')),
      efferent: collectNarratives(stemEvents.filter((event) => event.family === 'stem.efferent')),
      nsCatalog: collectNarratives(stemEvents.filter((event) => event.family === 'stem.ns-catalog')),
      proprioception: collectNarratives(stemEvents.filter((event) => event.family === 'stem.proprioception')),
      afferentRules: collectNarratives(stemEvents.filter((event) => event.family === 'stem.afferent.rule')),
    },
    spine: {
      adapters: collectNarratives(spineEvents.filter((event) => event.family === 'spine.adapter')),
      endpoints: collectNarratives(spineEvents.filter((event) => event.family === 'spine.endpoint')),
      senses: collectNarratives(spineEvents.filter((event) => event.family === 'spine.sense')),
      acts: collectNarratives(spineEvents.filter((event) => event.family === 'spine.act')),
    },
    rawEvents,
  }
}
