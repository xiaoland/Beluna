<script setup lang="ts">
import { formatWhen, prettyCount } from '@/presentation/format'
import type { TickSummary } from '@/projection/lachesis/models'
import { computed } from 'vue'

const props = defineProps<{
  hiddenTickCount: number
  runId: string | null
  ticks: TickSummary[]
  selectedTick: number | null
  loading: boolean
  cortexView: boolean
}>()

defineEmits<{
  select: [tick: number]
}>()

const subtitle = computed(() => {
  if (!props.runId) {
    return 'Choose a wake session to see its tick cadence.'
  }

  return props.cortexView
    ? `Wake ${props.runId} · Cortex-handled ticks only`
    : `Wake ${props.runId} · all projected ticks`
})
</script>

<template>
  <section class="panel-shell">
    <div class="panel-head">
      <div>
        <p class="panel-kicker">Cortex Anchor</p>
        <h2 class="panel-title">Tick Timeline</h2>
        <p class="panel-subtitle">
          {{ subtitle }}
        </p>
      </div>
    </div>

    <div v-if="!runId" class="empty-state">No wake selected yet.</div>
    <div v-else-if="loading && !ticks.length" class="empty-state">Loading ticks…</div>
    <div v-else-if="cortexView && !ticks.length && hiddenTickCount > 0" class="empty-state">
      This wake has no Cortex-handled ticks. Switch to Stem, Spine, or Raw to inspect the hidden unhandled ticks.
    </div>
    <div v-else-if="!ticks.length" class="empty-state">
      This wake has no projected ticks yet. Lachesis will populate them after ingest.
    </div>

    <div v-else class="timeline">
      <p v-if="cortexView && hiddenTickCount > 0" class="timeline-note">
        {{ hiddenTickCount }} unhandled tick{{ hiddenTickCount === 1 ? '' : 's' }} hidden in Cortex View.
      </p>

      <button
        v-for="tick in ticks"
        :key="tick.tick"
        type="button"
        class="timeline-row"
        :class="{ selected: tick.tick === selectedTick }"
        @click="$emit('select', tick.tick)"
      >
        <div class="rail">
          <span class="dot"></span>
        </div>

        <div class="timeline-copy">
          <div class="timeline-head">
            <strong>Tick {{ tick.tick }}</strong>
            <span>{{ formatWhen(tick.lastSeenAt) }}</span>
          </div>

          <div class="timeline-grid">
            <span>Events {{ prettyCount(tick.eventCount) }}</span>
            <span>Warnings {{ prettyCount(tick.warningCount) }}</span>
            <span>Errors {{ prettyCount(tick.errorCount) }}</span>
          </div>
        </div>
      </button>
    </div>
  </section>
</template>

<style scoped>
.timeline {
  display: grid;
  gap: 0.3rem;
  padding: 0 0.9rem 0.9rem;
}

.timeline-note {
  margin: 0 0 0.35rem;
  color: var(--text-muted);
  font-size: 0.84rem;
  line-height: 1.5;
}

.timeline-row {
  width: 100%;
  display: grid;
  grid-template-columns: 1.6rem minmax(0, 1fr);
  gap: 0.8rem;
  text-align: left;
  padding: 0.68rem 0.75rem;
  border: 1px solid transparent;
  background: transparent;
  transition:
    background-color 140ms ease,
    border-color 140ms ease;
}

.timeline-row:hover {
  background: var(--panel);
  border-color: var(--line-soft);
}

.timeline-row.selected {
  background: color-mix(in srgb, var(--accent) 6%, white);
  border-color: color-mix(in srgb, var(--accent) 28%, var(--line-strong));
}

.rail {
  position: relative;
  display: flex;
  justify-content: center;
}

.rail::before {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  width: 2px;
  background: var(--line-soft);
}

.dot {
  position: relative;
  z-index: 1;
  width: 0.82rem;
  height: 0.82rem;
  margin-top: 0.12rem;
  border: 2px solid var(--accent);
  background: var(--panel-strong);
}

.timeline-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.8rem;
  margin-bottom: 0.45rem;
  color: var(--text-muted);
  font-size: 0.86rem;
}

.timeline-head strong {
  color: var(--text-strong);
  font-size: 1rem;
}

.timeline-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 0.6rem 1rem;
  color: var(--text-muted);
  font-size: 0.84rem;
}
</style>
