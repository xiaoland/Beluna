import type { TickSummary } from '@/projection/lachesis/models'

export type LachesisDetailTab = 'cortex' | 'stem' | 'spine' | 'raw'
export type CortexViewMode = 'timeline' | 'narrative'

export function defaultDetailTabForTicks(ticks: TickSummary[]): LachesisDetailTab {
  return ticks.some((tick) => tick.cortexHandled) ? 'cortex' : 'raw'
}

export function visibleTicksForDetailTab(
  ticks: TickSummary[],
  tab: LachesisDetailTab,
): TickSummary[] {
  if (tab !== 'cortex') {
    return ticks
  }

  return ticks.filter((tick) => tick.cortexHandled)
}
