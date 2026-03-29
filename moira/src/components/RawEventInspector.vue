<script setup lang="ts">
import { formatWhen, rawEventHeadline } from '@/presenters'
import type { RawEvent } from '@/types'

defineProps<{
  rawEvents: RawEvent[]
}>()

function pretty(value: unknown): string {
  return JSON.stringify(value, null, 2) ?? 'null'
}
</script>

<template>
  <div v-if="!rawEvents.length" class="empty-state">
    No raw events were attached to this tick detail response yet.
  </div>

  <div v-else class="event-stack">
    <details v-for="event in rawEvents" :key="event.rawEventId" class="event-card">
      <summary class="event-summary">
        <div>
          <strong>{{ rawEventHeadline(event) }}</strong>
          <p class="event-meta">
            {{ event.subsystem ?? 'unknown subsystem' }} / {{ event.family ?? 'unknown family' }}
          </p>
        </div>

        <div class="event-meta right">
          <span>{{ event.severityText ?? 'INFO' }}</span>
          <span>{{ formatWhen(event.observedAt) }}</span>
        </div>
      </summary>

      <div class="event-detail">
        <div class="kv">
          <span class="label">run_id</span>
          <span class="mono">{{ event.runId ?? '—' }}</span>
        </div>
        <div class="kv">
          <span class="label">tick</span>
          <span>{{ event.tick ?? '—' }}</span>
        </div>
        <div class="kv">
          <span class="label">target</span>
          <span>{{ event.target ?? '—' }}</span>
        </div>

        <div class="payload-grid">
          <article class="payload-card">
            <h4>Payload</h4>
            <pre>{{ pretty(event.payload) }}</pre>
          </article>

          <article class="payload-card">
            <h4>Body</h4>
            <pre>{{ pretty(event.body) }}</pre>
          </article>

          <article class="payload-card">
            <h4>Attributes</h4>
            <pre>{{ pretty(event.attributes) }}</pre>
          </article>

          <article class="payload-card">
            <h4>Resource</h4>
            <pre>{{ pretty(event.resource) }}</pre>
          </article>

          <article class="payload-card">
            <h4>Scope</h4>
            <pre>{{ pretty(event.scope) }}</pre>
          </article>
        </div>
      </div>
    </details>
  </div>
</template>

<style scoped>
.event-stack {
  display: grid;
  gap: 0.75rem;
}

.event-card {
  border: 1px solid var(--line-soft);
  background: var(--panel-strong);
  overflow: hidden;
}

.event-summary {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.8rem;
  padding: 0.95rem 1rem;
  list-style: none;
  cursor: pointer;
}

.event-summary::-webkit-details-marker {
  display: none;
}

.event-meta {
  margin: 0.3rem 0 0;
  color: var(--text-muted);
  font-size: 0.82rem;
}

.event-meta.right {
  display: grid;
  justify-items: end;
  gap: 0.2rem;
}

.event-detail {
  padding: 0 1rem 1rem;
}

.kv {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.5rem 0;
  border-top: 1px solid var(--line-soft);
}

.label {
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-size: 0.74rem;
}

.payload-grid {
  display: grid;
  gap: 0.75rem;
  margin-top: 0.85rem;
}

.payload-card {
  padding: 0.8rem;
  background: var(--panel);
  border: 1px solid var(--line-soft);
}

h4 {
  margin: 0 0 0.55rem;
  font-size: 0.88rem;
}
</style>
