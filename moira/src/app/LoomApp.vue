<script setup lang="ts">
import { computed } from 'vue'
import TickDetailPanel from '@/presentation/lachesis/workspace/TickDetailPanel.vue'
import TickTimeline from '@/presentation/lachesis/workspace/TickTimeline.vue'
import WakeSessionList from '@/presentation/lachesis/workspace/WakeSessionList.vue'
import StatusHeader from '@/presentation/loom/chrome/StatusHeader.vue'
import { formatWhen } from '@/presentation/format'
import { useLachesisWorkspace } from '@/query/lachesis/workspace'

const {
  activeTab,
  issue,
  loading,
  refreshVisibleState,
  selectTick,
  selectWake,
  selectedRunId,
  selectedTick,
  selectedTickDetail,
  status,
  wakeSessions,
  tickTimeline,
} = useLachesisWorkspace()

const selectionHint = computed(() => {
  if (!selectedTickDetail.value) {
    return 'Select a wake session and tick to inspect its chronology, intervals, and source-grounded detail.'
  }

  return `Tick ${selectedTickDetail.value.tick} from wake ${selectedTickDetail.value.runId} · ${selectedTickDetail.value.chronology.lanes.length} lanes · ${selectedTickDetail.value.rawEvents.length} raw events · updated ${formatWhen(
    selectedTickDetail.value.rawEvents[selectedTickDetail.value.rawEvents.length - 1]?.observedAt ?? null,
  )}`
})
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
  background: var(--panel-strong);
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
