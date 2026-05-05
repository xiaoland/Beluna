<script setup lang="ts">
import { formatWhen } from '@/presentation/format'
import JsonSectionGroup from '@/presentation/loom/shared/JsonSectionGroup.vue'
import { jsonSectionsForEvent } from '@/projection/lachesis/json-sections'
import { rawEventHeadline } from '@/projection/lachesis/labels'
import type { RawEvent } from '@/projection/lachesis/models'

defineProps<{
  rawEvents: RawEvent[]
}>()
</script>

<template>
  <div v-if="!rawEvents.length" class="empty-state">
    No raw events were attached to this tick detail response yet.
  </div>

  <div v-else class="event-stack">
    <details
      v-for="event in rawEvents"
      :key="event.rawEventId"
      class="event-card"
    >
      <summary class="event-summary">
        <div>
          <strong>{{ rawEventHeadline(event) }}</strong>
          <p class="event-meta">
            {{ event.scopeName ?? event.subsystem ?? 'unknown scope' }} /
            {{ event.eventName ?? event.family ?? 'unknown event' }}
          </p>
        </div>

        <div class="event-meta right">
          <span class="record-kind">{{ event.recordKind }}</span>
          <span>{{ event.severityText ?? 'INFO' }}</span>
          <span>{{ formatWhen(event.observedAt) }}</span>
        </div>
      </summary>

      <div class="event-detail">
        <div class="kv">
          <span class="label">record_kind</span>
          <span>{{ event.recordKind }}</span>
        </div>
        <div class="kv">
          <span class="label">run_id</span>
          <span class="mono">{{ event.runId ?? '—' }}</span>
        </div>
        <div class="kv">
          <span class="label">trace_id</span>
          <span class="mono">{{ event.traceId ?? '—' }}</span>
        </div>
        <div class="kv">
          <span class="label">span_id</span>
          <span class="mono">{{ event.spanId ?? '—' }}</span>
        </div>
        <div class="kv">
          <span class="label">tick</span>
          <span>{{ event.tick ?? '—' }}</span>
        </div>
        <div class="kv">
          <span class="label">schema</span>
          <span>{{
            event.scopeName && event.eventName
              ? `${event.scopeName} / ${event.eventName}`
              : (event.target ?? '—')
          }}</span>
        </div>

        <JsonSectionGroup
          :sections="jsonSectionsForEvent(event, { openPayload: true })"
        />
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

.record-kind {
  color: var(--text);
  font-size: 0.72rem;
  font-weight: 700;
  text-transform: uppercase;
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

.event-detail > :last-child {
  margin-top: 0.85rem;
}
</style>
