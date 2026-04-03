<script setup lang="ts">
import { computed } from 'vue'

import { formatWhen } from '@/presentation/format'
import type { TickDetail, TickSummary, WakeSessionSummary } from '@/projection/lachesis/models'
import TickDetailPanel from './TickDetailPanel.vue'
import TickTimeline from './TickTimeline.vue'
import WakeSessionList from './WakeSessionList.vue'

type LachesisDetailTab = 'chronology' | 'cortex' | 'stem' | 'spine' | 'raw'

const props = defineProps<{
  activeTab: LachesisDetailTab
  loading: {
    detail: boolean
    ticks: boolean
    wakes: boolean
  }
  selectedRunId: string | null
  selectedTick: number | null
  selectedTickDetail: TickDetail | null
  tickTimeline: TickSummary[]
  wakeSessions: WakeSessionSummary[]
}>()

const emit = defineEmits<{
  selectTick: [tick: number]
  selectWake: [runId: string]
  'update:tab': [tab: LachesisDetailTab]
}>()

const selectionHint = computed(() => {
  if (!props.selectedTickDetail) {
    return 'Select a wake session and tick to inspect chronology, subsystem intervals, and source-grounded raw detail.'
  }

  return `Tick ${props.selectedTickDetail.tick} from wake ${props.selectedTickDetail.runId} · ${props.selectedTickDetail.chronology.lanes.length} lanes · ${props.selectedTickDetail.rawEvents.length} raw events · updated ${formatWhen(
    props.selectedTickDetail.rawEvents[props.selectedTickDetail.rawEvents.length - 1]?.observedAt ?? null,
  )}`
})
</script>

<template>
  <section class="panel-shell workspace-panel">
    <div class="panel-head workspace-head">
      <div>
        <p class="panel-kicker">Lachesis</p>
        <h2 class="panel-title">Wake Browse Surface</h2>
        <p class="panel-subtitle">
          Inspect wake sessions, follow the tick timeline, and open the selected tick through chronology, cortex, stem,
          spine, or raw detail.
        </p>
      </div>
    </div>

    <p class="selection-hint">{{ selectionHint }}</p>

    <main class="workspace-shell">
      <section class="workspace-row">
        <WakeSessionList
          class="pane wake-pane"
          :runs="wakeSessions"
          :selected-run-id="selectedRunId"
          :loading="loading.wakes"
          @select="emit('selectWake', $event)"
        />
      </section>

      <section class="workspace-row workspace-focus">
        <TickTimeline
          class="pane timeline-pane"
          :run-id="selectedRunId"
          :ticks="tickTimeline"
          :selected-tick="selectedTick"
          :loading="loading.ticks"
          @select="emit('selectTick', $event)"
        />

        <TickDetailPanel
          class="pane detail-pane"
          :tab="activeTab"
          :detail="selectedTickDetail"
          :loading="loading.detail"
          @update:tab="emit('update:tab', $event)"
        />
      </section>
    </main>
  </section>
</template>

<style scoped>
.workspace-panel {
  position: relative;
  z-index: 1;
  overflow: hidden;
}

.workspace-head {
  padding-bottom: 0.9rem;
}

.selection-hint {
  margin: 0 1rem 1rem;
  padding: 0.72rem 0.9rem;
  border: 1px solid var(--line-soft);
  background: var(--panel-strong);
  color: var(--text-muted);
}

.workspace-shell {
  display: grid;
  gap: 1rem;
  padding: 0 1rem 1rem;
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
