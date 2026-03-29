<script setup lang="ts">
import { computed } from 'vue'
import { formatWhen, prettyCount, stateTone } from '@/presenters'
import type { ReceiverStatus } from '@/types'

const props = defineProps<{
  status: ReceiverStatus | null
  loading: boolean
  issue: string | null
}>()

defineEmits<{
  refresh: []
}>()

const tone = computed(() => stateTone(props.status?.state ?? null))
</script>

<template>
  <header class="hero panel-shell">
    <div class="hero-copy">
      <p class="eyebrow">Moira / Loom</p>
      <h1>Wake Inspection Control Plane</h1>
      <p class="hero-text">
        Stage 1 Lachesis view over local OTLP ingest, wake sessions, and tick-scoped inspection.
      </p>
      <div class="hero-tags">
        <span class="status-badge" :class="`status-${tone}`">
          {{ status?.state ?? (loading ? 'loading' : 'unknown') }}
        </span>
        <span v-if="issue" class="issue-badge">{{ issue }}</span>
      </div>
    </div>

    <div class="hero-actions">
      <button class="button-primary" type="button" :disabled="loading" @click="$emit('refresh')">
        Refresh Loom
      </button>
    </div>

    <div class="metrics">
      <article class="metric-card">
        <span class="metric-label">Receiver State</span>
        <strong>{{ status?.state ?? 'unknown' }}</strong>
      </article>

      <article class="metric-card wide">
        <span class="metric-label">Storage Path</span>
        <strong class="mono">{{ status?.storagePath ?? 'Not reported yet' }}</strong>
      </article>

      <article class="metric-card">
        <span class="metric-label">Receiver Bind</span>
        <strong class="mono">{{ status?.receiverBind ?? '—' }}</strong>
      </article>

      <article class="metric-card">
        <span class="metric-label">Last Ingest</span>
        <strong>{{ formatWhen(status?.lastIngestAt ?? null) }}</strong>
      </article>

      <article class="metric-card">
        <span class="metric-label">Raw Events</span>
        <strong>{{ prettyCount(status?.rawEventCount ?? null) }}</strong>
      </article>

      <article class="metric-card">
        <span class="metric-label">Wakes / Ticks</span>
        <strong>{{ prettyCount(status?.runCount ?? null) }} / {{ prettyCount(status?.tickCount ?? null) }}</strong>
      </article>
    </div>
  </header>
</template>

<style scoped>
.hero {
  position: relative;
  z-index: 1;
  margin-bottom: 1rem;
  padding: 1.1rem;
  display: grid;
  gap: 1rem;
  grid-template-columns: minmax(0, 1.3fr) auto;
}

.eyebrow {
  margin: 0 0 0.35rem;
  color: var(--text-muted);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  font-size: 0.78rem;
}

h1 {
  margin: 0;
  font-family: var(--font-display);
  font-size: clamp(1.9rem, 3vw, 2.6rem);
}

.hero-text {
  margin: 0.55rem 0 0;
  max-width: 48rem;
  color: var(--text-muted);
  line-height: 1.5;
}

.hero-actions {
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
}

.hero-tags {
  margin-top: 0.85rem;
  display: flex;
  flex-wrap: wrap;
  gap: 0.55rem;
}

.issue-badge {
  display: inline-flex;
  align-items: center;
  min-height: 2rem;
  padding: 0.2rem 0.75rem;
  border-radius: 999px;
  background: rgba(164, 63, 47, 0.12);
  color: var(--err);
  border: 1px solid rgba(164, 63, 47, 0.22);
  font-size: 0.82rem;
}

.metrics {
  grid-column: 1 / -1;
  display: grid;
  gap: 0.75rem;
  grid-template-columns: repeat(6, minmax(0, 1fr));
}

.metric-card {
  min-width: 0;
  padding: 0.9rem;
  border-radius: 0.95rem;
  background: var(--panel-strong);
  border: 1px solid var(--line-soft);
  box-shadow: var(--shadow-card);
}

.metric-card.wide {
  grid-column: span 2;
}

.metric-label {
  display: block;
  margin-bottom: 0.4rem;
  font-size: 0.78rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

strong {
  font-size: 0.98rem;
  line-height: 1.45;
}

@media (max-width: 1180px) {
  .hero {
    grid-template-columns: 1fr;
  }

  .hero-actions {
    justify-content: flex-start;
  }

  .metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .metric-card.wide {
    grid-column: span 2;
  }
}

@media (max-width: 780px) {
  .metrics {
    grid-template-columns: 1fr;
  }

  .metric-card.wide {
    grid-column: span 1;
  }
}
</style>
