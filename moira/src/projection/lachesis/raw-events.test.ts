import { describe, expect, it } from 'vitest'

import type { EventRecordPayload } from '@/bridge/contracts/lachesis'
import { normalizeRawEvent } from './raw-events'

describe('Lachesis raw event normalization', () => {
  it('keeps native OTLP identity as native owner metadata', () => {
    const event = normalizeRawEvent(
      eventRecord({
        rawEventId: 'evt-native',
        recordKind: 'native_owner',
        scopeName: 'beluna.core.stem',
        eventName: 'tick.granted',
        traceId: 'trace-1',
        spanId: 'span-tick',
        body: {
          summary: 'Stem granted tick 1.',
          run_id: 'run-1',
          tick: 1,
        },
      }),
    )

    expect(event).toMatchObject({
      rawEventId: 'evt-native',
      recordKind: 'native_owner',
      scopeName: 'beluna.core.stem',
      eventName: 'tick.granted',
      traceId: 'trace-1',
      spanId: 'span-tick',
      subsystem: 'stem',
      runId: 'run-1',
      tick: 1,
    })
  })

  it('marks legacy contract payloads and preserves their rich payload', () => {
    const event = normalizeRawEvent(
      eventRecord({
        rawEventId: 'evt-legacy',
        recordKind: 'legacy_contract',
        scopeName: 'observability.contract',
        eventName: null,
        target: 'cortex.primary',
        family: 'cortex.primary',
        subsystem: 'cortex',
        runId: 'run-1',
        tick: 1,
        attributes: {
          family: 'cortex.primary',
          payload: JSON.stringify({
            organ: 'primary',
            phase: 'started',
          }),
        },
        body: 'Cortex primary phase started.',
      }),
    )

    expect(event.recordKind).toBe('legacy_contract')
    expect(event.payload).toEqual({
      organ: 'primary',
      phase: 'started',
    })
    expect(event.messageText).toBe('Cortex primary phase started.')
  })
})

function eventRecord(
  overrides: Partial<EventRecordPayload>,
): EventRecordPayload {
  return {
    rawEventId: 'evt',
    receivedAt: '2026-05-05T00:00:00Z',
    observedAt: '2026-05-05T00:00:00Z',
    severityText: 'INFO',
    severityNumber: 9,
    recordKind: 'ordinary_log',
    scopeName: null,
    eventName: null,
    traceId: null,
    spanId: null,
    traceFlags: null,
    target: null,
    family: null,
    subsystem: null,
    runId: null,
    tick: null,
    messageText: null,
    attributes: {},
    body: null,
    resource: {},
    scope: {},
    ...overrides,
  }
}
