<script setup lang="ts">
import { formatWhen, prettyCount, stateTone } from '@/presenters'
import type { WakeSessionSummary } from '@/types'

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
          <span class="status-badge" :class="`status-${stateTone(run.state)}`">
            {{ run.state ?? 'observed' }}
          </span>
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
  border-radius: 999px;
  background: rgba(168, 93, 44, 0.12);
  color: var(--accent);
  border: 1px solid rgba(168, 93, 44, 0.18);
  font-weight: 600;
}

.session-list {
  display: grid;
  gap: 0.75rem;
  padding: 0 1rem 1rem;
}

.session-card {
  width: 100%;
  text-align: left;
  border: 1px solid var(--line-soft);
  background: var(--panel-strong);
  border-radius: 1rem;
  padding: 0.9rem;
  box-shadow: var(--shadow-card);
  transition:
    border-color 140ms ease,
    transform 140ms ease,
    box-shadow 140ms ease;
}

.session-card:hover {
  transform: translateY(-1px);
  border-color: rgba(168, 93, 44, 0.3);
}

.session-card.selected {
  border-color: rgba(168, 93, 44, 0.48);
  box-shadow: 0 16px 28px rgba(168, 93, 44, 0.14);
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
  .card-grid {
    grid-template-columns: 1fr;
  }
}
</style>
