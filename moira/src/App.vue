<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import StatusHeader from '@/components/StatusHeader.vue'
import TickDetailPanel from '@/components/TickDetailPanel.vue'
import TickTimeline from '@/components/TickTimeline.vue'
import WakeSessionList from '@/components/WakeSessionList.vue'
import {
  LACHESIS_UPDATED_EVENT,
  getReceiverStatus,
  getTickDetail,
  getTicks,
  getWakeSessions,
  hasTauriBridge,
} from '@/api'
import { formatWhen } from '@/presenters'
import type { DetailTab, ReceiverStatus, TickDetail, TickSummary, WakeSessionSummary } from '@/types'

const status = ref<ReceiverStatus | null>(null)
const wakeSessions = ref<WakeSessionSummary[]>([])
const tickTimeline = ref<TickSummary[]>([])
const selectedRunId = ref<string | null>(null)
const selectedTick = ref<string | null>(null)
const selectedTickDetail = ref<TickDetail | null>(null)
const activeTab = ref<DetailTab>('chronology')
const issue = ref<string | null>(null)
const usingTauri = hasTauriBridge()

const loading = reactive({
  status: false,
  wakes: false,
  ticks: false,
  detail: false,
})

let unlisten: UnlistenFn | null = null
let refreshTimer: number | null = null

const selectionHint = computed(() => {
  if (!selectedTickDetail.value) {
    return 'Select a wake session and tick to inspect its chronology, intervals, and source-grounded detail.'
  }

  return `Tick ${selectedTickDetail.value.tick} from wake ${selectedTickDetail.value.runId} · ${selectedTickDetail.value.chronology.lanes.length} lanes · ${selectedTickDetail.value.rawEvents.length} raw events · updated ${formatWhen(
    selectedTickDetail.value.rawEvents[selectedTickDetail.value.rawEvents.length - 1]?.observedAt ?? null,
  )}`
})

onMounted(async () => {
  if (!usingTauri) {
    issue.value =
      'Loom is running without the Tauri bridge. Start Moira through the desktop shell to query live Lachesis state.'
    return
  }

  await refreshVisibleState()

  unlisten = await listen(LACHESIS_UPDATED_EVENT, () => {
    if (refreshTimer) {
      window.clearTimeout(refreshTimer)
    }

    refreshTimer = window.setTimeout(() => {
      void refreshVisibleState()
    }, 180)
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

async function refreshVisibleState(): Promise<void> {
  issue.value = null

  await Promise.all([loadStatus(), loadWakeSessions(true)])

  if (!selectedRunId.value) {
    tickTimeline.value = []
    selectedTick.value = null
    selectedTickDetail.value = null
    return
  }

  await loadTicksForRun(selectedRunId.value, true)

  if (!selectedTick.value) {
    selectedTickDetail.value = null
    return
  }

  await loadTickDetailForSelection(selectedRunId.value, selectedTick.value)
}

async function loadStatus(): Promise<void> {
  loading.status = true

  try {
    status.value = await getReceiverStatus()
  } catch (error) {
    issue.value = `Unable to load receiver status: ${errorMessage(error)}`
  } finally {
    loading.status = false
  }
}

async function loadWakeSessions(preserveSelection: boolean): Promise<void> {
  loading.wakes = true

  try {
    const sessions = await getWakeSessions()
    wakeSessions.value = sessions

    const nextRunId =
      preserveSelection && selectedRunId.value && sessions.some((session) => session.runId === selectedRunId.value)
        ? selectedRunId.value
        : sessions[0]?.runId ?? null

    if (nextRunId !== selectedRunId.value) {
      selectedRunId.value = nextRunId
      selectedTick.value = null
      selectedTickDetail.value = null
      activeTab.value = 'chronology'
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
    const ticks = await getTicks(runId)
    if (selectedRunId.value !== runId) {
      return
    }

    tickTimeline.value = ticks

    const nextTick =
      preserveSelection && selectedTick.value && ticks.some((entry) => entry.tick === selectedTick.value)
        ? selectedTick.value
        : ticks[0]?.tick ?? null

    if (nextTick !== selectedTick.value) {
      selectedTick.value = nextTick
      selectedTickDetail.value = null
      activeTab.value = 'chronology'
    }
  } catch (error) {
    issue.value = `Unable to load tick timeline: ${errorMessage(error)}`
  } finally {
    loading.ticks = false
  }
}

async function loadTickDetailForSelection(runId: string, tick: string): Promise<void> {
  loading.detail = true

  try {
    const detail = await getTickDetail(runId, tick)
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
  activeTab.value = 'chronology'
  await loadTicksForRun(runId, false)

  if (selectedTick.value) {
    await loadTickDetailForSelection(runId, selectedTick.value)
  }
}

async function selectTick(tick: string): Promise<void> {
  if (!selectedRunId.value || tick === selectedTick.value) {
    return
  }

  selectedTick.value = tick
  selectedTickDetail.value = null
  activeTab.value = 'chronology'
  await loadTickDetailForSelection(selectedRunId.value, tick)
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}
</script>

<template>
  <div class="app-shell">
    <div class="bg-orb orb-a"></div>
    <div class="bg-orb orb-b"></div>

    <StatusHeader
      :status="status"
      :loading="loading.status"
      :issue="issue"
      @refresh="refreshVisibleState"
    />

    <p v-if="selectionHint" class="selection-hint">{{ selectionHint }}</p>

    <main class="workspace-shell">
      <section class="workspace-row">
        <WakeSessionList
          class="pane wake-pane"
          :runs="wakeSessions"
          :selected-run-id="selectedRunId"
          :loading="loading.wakes"
          @select="selectWake"
        />
      </section>

      <section class="workspace-row workspace-focus">
        <TickTimeline
          class="pane timeline-pane"
          :run-id="selectedRunId"
          :ticks="tickTimeline"
          :selected-tick="selectedTick"
          :loading="loading.ticks"
          @select="selectTick"
        />

        <TickDetailPanel
          class="pane detail-pane"
          v-model:tab="activeTab"
          :detail="selectedTickDetail"
          :loading="loading.detail"
        />
      </section>
    </main>
  </div>
</template>

<style scoped>
.selection-hint {
  margin: 0 0 1rem;
  padding: 0.72rem 0.9rem;
  border: 1px solid var(--line-soft);
  background: var(--panel);
  color: var(--text-muted);
}

.workspace-shell {
  position: relative;
  z-index: 1;
  display: grid;
  gap: 1rem;
}

.workspace-row {
  min-width: 0;
}

.workspace-focus {
  display: grid;
  grid-template-columns: minmax(17rem, 2fr) minmax(0, 8fr);
  gap: 1rem;
  align-items: start;
}

.detail-pane {
  min-height: 34rem;
}

@media (max-width: 980px) {
  .workspace-focus {
    grid-template-columns: 1fr;
  }
}
</style>
