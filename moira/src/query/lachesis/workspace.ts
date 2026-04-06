import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue'

import { listenLachesisUpdated } from '@/bridge/events'
import { hasTauriBridge } from '@/bridge/env'
import {
  fetchReceiverStatus,
  fetchTickDetail,
  fetchTicks,
  fetchWakeSessions,
} from '@/bridge/lachesis'
import {
  compareTicks,
  compareWakeSessions,
  normalizeReceiverStatus,
  normalizeTickDetail,
  normalizeTickSummary,
  normalizeWakeSession,
} from '@/projection/lachesis'
import type {
  ReceiverStatus,
  TickDetail,
  TickSummary,
  WakeSessionSummary,
} from '@/projection/lachesis/models'
import {
  defaultDetailTabForTicks,
  visibleTicksForDetailTab,
  type CortexViewMode,
  type LachesisDetailTab,
} from './state'

type StopListening = () => void | Promise<void>

export function useLachesisWorkspace() {
  const status = ref<ReceiverStatus | null>(null)
  const wakeSessions = ref<WakeSessionSummary[]>([])
  const allTicks = ref<TickSummary[]>([])
  const selectedRunId = ref<string | null>(null)
  const selectedTick = ref<number | null>(null)
  const selectedTickDetail = ref<TickDetail | null>(null)
  const activeTab = ref<LachesisDetailTab>('raw')
  const cortexMode = ref<CortexViewMode>('timeline')
  const issue = ref<string | null>(null)
  const usingTauri = hasTauriBridge()
  const tickTimeline = computed(() => visibleTicksForDetailTab(allTicks.value, activeTab.value))
  const hiddenTickCount = computed(() => allTicks.value.length - tickTimeline.value.length)

  const loading = reactive({
    status: false,
    wakes: false,
    ticks: false,
    detail: false,
  })

  let unlisten: StopListening | null = null
  let refreshTimer: number | null = null

  onMounted(async () => {
    if (!usingTauri) {
      issue.value =
        'Loom is running without the Tauri bridge. Start Moira through the desktop shell to query live Lachesis state.'
      return
    }

    await refreshVisibleState()
    unlisten = await listenLachesisUpdated(() => {
      scheduleRefresh()
    })
  })

  onBeforeUnmount(() => {
    if (refreshTimer) {
      window.clearTimeout(refreshTimer)
    }

    if (unlisten) {
      void unlisten()
    }
  })

  function scheduleRefresh() {
    if (refreshTimer) {
      window.clearTimeout(refreshTimer)
    }

    refreshTimer = window.setTimeout(() => {
      void refreshVisibleState()
    }, 180)
  }

  async function refreshVisibleState(): Promise<void> {
    issue.value = null

    await Promise.all([loadStatus(), loadWakeSessions(true)])

    if (!selectedRunId.value) {
      allTicks.value = []
      selectedTick.value = null
      selectedTickDetail.value = null
      return
    }

    await loadTicksForRun(selectedRunId.value, true)

    if (selectedTick.value == null) {
      selectedTickDetail.value = null
      return
    }

    await loadTickDetailForSelection(selectedRunId.value, selectedTick.value)
  }

  async function loadStatus(): Promise<void> {
    loading.status = true

    try {
      const payload = await fetchReceiverStatus()
      status.value = normalizeReceiverStatus(payload)
    } catch (error) {
      issue.value = `Unable to load receiver status: ${errorMessage(error)}`
    } finally {
      loading.status = false
    }
  }

  async function loadWakeSessions(preserveSelection: boolean): Promise<void> {
    loading.wakes = true

    try {
      const payload = await fetchWakeSessions()
      const sessions = payload.map(normalizeWakeSession).sort(compareWakeSessions)
      wakeSessions.value = sessions

      const nextRunId =
        preserveSelection && selectedRunId.value && sessions.some((session) => session.runId === selectedRunId.value)
          ? selectedRunId.value
          : sessions[0]?.runId ?? null

      if (nextRunId !== selectedRunId.value) {
        selectedRunId.value = nextRunId
        selectedTick.value = null
        selectedTickDetail.value = null
      }
    } catch (error) {
      issue.value = `Unable to load wake sessions: ${errorMessage(error)}`
    } finally {
      loading.wakes = false
    }
  }

  async function loadTicksForRun(runId: string, preserveSelection: boolean): Promise<void> {
    loading.ticks = true

    try {
      const payload = await fetchTicks(runId)
      const ticks = payload.map(normalizeTickSummary).sort(compareTicks)
      if (selectedRunId.value !== runId) {
        return
      }

      allTicks.value = ticks
      if (!preserveSelection && selectedTick.value == null) {
        activeTab.value = defaultDetailTabForTicks(ticks)
      } else if (activeTab.value === 'cortex' && !ticks.some((entry) => entry.cortexHandled)) {
        activeTab.value = 'raw'
      }
      const nextTick = nextSelectedTick(ticks, activeTab.value, selectedTick.value, preserveSelection)

      if (nextTick !== selectedTick.value) {
        selectedTick.value = nextTick
        selectedTickDetail.value = null
      }
    } catch (error) {
      issue.value = `Unable to load tick timeline: ${errorMessage(error)}`
    } finally {
      loading.ticks = false
    }
  }

  async function loadTickDetailForSelection(runId: string, tick: number): Promise<void> {
    loading.detail = true

    try {
      const payload = await fetchTickDetail(runId, tick)
      const detail = normalizeTickDetail(payload)
      if (selectedRunId.value !== runId || selectedTick.value !== tick) {
        return
      }

      selectedTickDetail.value = detail
    } catch (error) {
      issue.value = `Unable to load tick detail: ${errorMessage(error)}`
    } finally {
      loading.detail = false
    }
  }

  async function selectWake(runId: string): Promise<void> {
    if (runId === selectedRunId.value) {
      return
    }

    selectedRunId.value = runId
    selectedTick.value = null
    selectedTickDetail.value = null
    await loadTicksForRun(runId, false)

    if (selectedTick.value != null) {
      await loadTickDetailForSelection(runId, selectedTick.value)
    }
  }

  async function selectTick(tick: number): Promise<void> {
    if (!selectedRunId.value || tick === selectedTick.value) {
      return
    }

    selectedTick.value = tick
    selectedTickDetail.value = null
    await loadTickDetailForSelection(selectedRunId.value, tick)
  }

  async function selectDetailTab(tab: LachesisDetailTab): Promise<void> {
    if (tab === activeTab.value) {
      return
    }

    activeTab.value = tab
    if (!selectedRunId.value) {
      return
    }

    const nextTick = nextSelectedTick(allTicks.value, activeTab.value, selectedTick.value, true)
    if (nextTick === selectedTick.value) {
      return
    }

    selectedTick.value = nextTick
    if (nextTick == null) {
      selectedTickDetail.value = null
      return
    }

    selectedTickDetail.value = null
    await loadTickDetailForSelection(selectedRunId.value, nextTick)
  }

  function selectCortexMode(mode: CortexViewMode): void {
    cortexMode.value = mode
  }

  return {
    activeTab,
    cortexMode,
    hiddenTickCount,
    issue,
    loading,
    refreshVisibleState,
    selectCortexMode,
    selectDetailTab,
    selectTick,
    selectWake,
    selectedRunId,
    selectedTick,
    selectedTickDetail,
    status,
    tickTimeline,
    usingTauri,
    wakeSessions,
  }
}

function nextSelectedTick(
  ticks: TickSummary[],
  tab: LachesisDetailTab,
  currentTick: number | null,
  preserveSelection: boolean,
): number | null {
  const visibleTicks = visibleTicksForDetailTab(ticks, tab)

  if (
    preserveSelection &&
    currentTick != null &&
    visibleTicks.some((entry) => entry.tick === currentTick)
  ) {
    return currentTick
  }

  return visibleTicks[0]?.tick ?? null
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}
