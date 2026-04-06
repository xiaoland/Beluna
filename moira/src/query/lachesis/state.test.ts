import { describe, expect, it } from 'vitest'

import { defaultDetailTabForTicks, visibleTicksForDetailTab } from './state'

describe('lachesis workspace state', () => {
  const ticks = [
    {
      runId: 'run-1',
      tick: 3,
      firstSeenAt: null,
      lastSeenAt: null,
      eventCount: 10,
      warningCount: 0,
      errorCount: 0,
      cortexHandled: false,
    },
    {
      runId: 'run-1',
      tick: 2,
      firstSeenAt: null,
      lastSeenAt: null,
      eventCount: 14,
      warningCount: 0,
      errorCount: 0,
      cortexHandled: true,
    },
  ]

  it('defaults to cortex when a wake has handled ticks', () => {
    expect(defaultDetailTabForTicks(ticks)).toBe('cortex')
  })

  it('defaults to raw when a wake has no handled ticks', () => {
    expect(defaultDetailTabForTicks(ticks.filter((tick) => !tick.cortexHandled))).toBe('raw')
  })

  it('filters visible ticks for cortex view only', () => {
    expect(visibleTicksForDetailTab(ticks, 'cortex').map((tick) => tick.tick)).toEqual([2])
    expect(visibleTicksForDetailTab(ticks, 'raw').map((tick) => tick.tick)).toEqual([3, 2])
  })
})
