import { describe, expect, it } from 'vitest'

import { buildChronology } from './chronology'
import type { RawEvent } from './models'

describe('Lachesis chronology projection', () => {
  it('uses native owner scope as the lane and pairs boundary events by span', () => {
    const chronology = buildChronology(
      [
        rawEvent('tick', 'beluna.core.stem.tick', 'granted', 'span-tick', 0),
        rawEvent('primary-start', 'beluna.core.cortex.primary', 'started', 'span-primary', 1),
        rawEvent('transport', 'beluna.core.ai-gateway.transport', 'request.completed', 'span-request', 2),
        rawEvent('primary-finish', 'beluna.core.cortex.primary', 'finished', 'span-primary', 3),
      ],
      [],
    )

    expect(chronology.lanes.map((lane) => lane.laneKey)).toEqual([
      'beluna.core.stem.tick',
      'beluna.core.cortex.primary',
      'beluna.core.ai-gateway.transport',
    ])

    const primaryLane = chronology.lanes.find((lane) => lane.laneKey === 'beluna.core.cortex.primary')
    expect(primaryLane?.entries).toHaveLength(1)
    expect(primaryLane?.entries[0]).toMatchObject({
      entryType: 'interval',
      title: 'started -> finished',
      laneType: 'owner',
    })
    expect(primaryLane?.entries[0].sourceEvents.map((event) => event.rawEventId)).toEqual([
      'primary-start',
      'primary-finish',
    ])

    const transportLane = chronology.lanes.find((lane) => lane.laneKey === 'beluna.core.ai-gateway.transport')
    expect(transportLane?.entries[0]).toMatchObject({
      entryType: 'point',
      title: 'request.completed',
      laneType: 'owner',
    })
  })
})

function rawEvent(
  rawEventId: string,
  scopeName: string,
  eventName: string,
  spanId: string,
  offsetMs: number,
): RawEvent {
  return {
    rawEventId,
    receivedAt: `2026-05-05T00:00:00.${String(offsetMs).padStart(3, '0')}Z`,
    observedAt: `2026-05-05T00:00:00.${String(offsetMs).padStart(3, '0')}Z`,
    severityText: 'INFO',
    severityNumber: 9,
    recordKind: 'native_owner',
    scopeName,
    eventName,
    traceId: 'trace-1',
    spanId,
    traceFlags: 1,
    target: null,
    family: null,
    subsystem: scopeName.slice('beluna.core.'.length).split('.')[0] ?? null,
    runId: 'run-1',
    tick: 1,
    messageText: null,
    payload: { summary: eventName },
    body: { summary: eventName },
    attributes: {},
    resource: {},
    scope: { name: scopeName },
  }
}
