<script setup lang="ts">
import { formatWhen, prettyCount } from '@/presentation/format'
import type { WakeSessionSummary } from '@/projection/lachesis/models'

defineProps<{
  runs: WakeSessionSummary[]
  selectedRunId: string | null
  loading: boolean
}>()

defineEmits<{
  select: [runId: string]
}>()
</script>

<template>
  <section class="panel-shell">
    <div class="panel-head">
      <div>
        <p class="panel-kicker">Lachesis Surface</p>
        <h2 class="panel-title">Wake Sessions</h2>
        <p class="panel-subtitle">Operator-facing run list backed by the `runs` projection.</p>
      </div>
      <span class="count-pill">{{ runs.length }}</span>
    </div>

    <div v-if="loading && !runs.length" class="empty-state">Loading wake sessions…</div>
    <div v-else-if="!runs.length" class="empty-state">
      No wake sessions yet. Once Core OTLP logs arrive, Lachesis will materialize them here.
    </div>

    <div v-else class="session-list">
      <button
        v-for="run in runs"
        :key="run.runId"
        type="button"
        class="session-card"
        :class="{ selected: run.runId === selectedRunId }"
        @click="$emit('select', run.runId)"
      >
        <div class="card-head">
          <span class="status-badge status-idle">observed</span>
          <span class="mono run-id">{{ run.runId }}</span>
        </div>

        <div class="card-grid">
          <div>
            <span class="label">Last seen</span>
            <strong>{{ formatWhen(run.lastSeenAt) }}</strong>
          </div>
          <div>
            <span class="label">Latest tick</span>
            <strong>{{ run.latestTick ?? '—' }}</strong>
          </div>
          <div>
            <span class="label">Events</span>
            <strong>{{ prettyCount(run.eventCount) }}</strong>
          </div>
          <div>
            <span class="label">Warnings / Errors</span>
            <strong>{{ prettyCount(run.warningCount) }} / {{ prettyCount(run.errorCount) }}</strong>
          </div>
        </div>
      </button>
    </div>
  </section>
</template>

<style scoped>
.count-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2rem;
  height: 2rem;
  padding: 0 0.65rem;
  background: color-mix(in srgb, var(--accent) 10%, white);
  color: var(--accent);
  border: 1px solid color-mix(in srgb, var(--accent) 22%, var(--line-strong));
  font-weight: 600;
}

.session-list {
  display: grid;
  gap: 0.55rem;
  grid-template-columns: repeat(auto-fit, minmax(20rem, 1fr));
  padding: 0 0.9rem 0.9rem;
}

.session-card {
  width: 100%;
  text-align: left;
  border: 1px solid var(--line-soft);
  background: var(--panel-strong);
  padding: 0.78rem;
  transition:
    border-color 140ms ease,
    background-color 140ms ease;
}

.session-card:hover {
  border-color: color-mix(in srgb, var(--accent) 40%, transparent);
}

.session-card.selected {
  border-color: color-mix(in srgb, var(--accent) 34%, var(--line-strong));
  background: color-mix(in srgb, var(--accent) 5%, white);
}

.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.8rem;
  margin-bottom: 0.9rem;
}

.run-id {
  color: var(--text-muted);
  font-size: 0.78rem;
}

.card-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.8rem;
}

.label {
  display: block;
  margin-bottom: 0.3rem;
  color: var(--text-muted);
  font-size: 0.76rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

strong {
  line-height: 1.4;
}

@media (max-width: 780px) {
  .session-list {
    grid-template-columns: 1fr;
  }

  .card-grid {
    grid-template-columns: 1fr;
  }
}
</style>
