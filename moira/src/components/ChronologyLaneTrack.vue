<script setup lang="ts">
import { formatWhen } from '@/presenters'
import type { ChronologyEntry, TickChronology } from '@/types'

defineProps<{
  chronology: TickChronology
  selectedRawEventId: string | null
}>()

defineEmits<{
  select: [entry: ChronologyEntry]
}>()

function laneLabel(type: string): string {
  return type.replace(/-/g, ' ')
}

function toneClass(entry: ChronologyEntry): string {
  const severity = (entry.severityText ?? '').toLowerCase()
  if (severity.includes('error') || severity.includes('fatal')) {
    return 'tone-err'
  }
  if (severity.includes('warn')) {
    return 'tone-warn'
  }
  return 'tone-ok'
}
</script>

<template>
  <div class="chronology-frame">
    <div class="axis-row">
      <div class="axis-spacer"></div>
      <div class="axis-track">
        <span class="axis-mark start">Start</span>
        <span class="axis-mark mid">Flow</span>
        <span class="axis-mark end">Settle</span>
      </div>
    </div>

    <div class="lane-stack">
      <div v-for="lane in chronology.lanes" :key="lane.id" class="lane-row">
        <div class="lane-meta">
          <span class="lane-type">{{ laneLabel(lane.laneType) }}</span>
          <strong>{{ lane.label }}</strong>
          <p>{{ lane.subtitle ?? `${lane.eventCount} events` }}</p>
        </div>

        <div class="lane-track">
          <div class="lane-grid"></div>

          <button
            v-for="entry in lane.entries"
            :key="entry.rawEventId"
            type="button"
            class="entry-pill"
            :class="[toneClass(entry), { selected: entry.rawEventId === selectedRawEventId }]"
            :style="{
              left: `${entry.position * 100}%`,
              width: `${Math.max((entry.endPosition - entry.position) * 100, 4)}%`,
            }"
            :title="`${entry.title} · ${formatWhen(entry.event.observedAt)}`"
            @click="$emit('select', entry)"
          >
            <span class="entry-kind">{{ entry.entryType === 'interval' ? 'interval' : 'event' }}</span>
            <span class="entry-title">{{ entry.title }}</span>
            <span v-if="entry.subtitle" class="entry-subtitle">{{ entry.subtitle }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.chronology-frame {
  overflow-x: auto;
}

.axis-row,
.lane-row {
  display: grid;
  grid-template-columns: 12rem minmax(44rem, 1fr);
  gap: 0.8rem;
}

.axis-row {
  margin-bottom: 0.4rem;
}

.axis-track,
.lane-track {
  position: relative;
}

.axis-track {
  min-height: 1.2rem;
  color: var(--text-muted);
  font-size: 0.72rem;
}

.axis-mark {
  position: absolute;
  top: 0;
}

.axis-mark.start { left: 0; }
.axis-mark.mid { left: 48%; }
.axis-mark.end { right: 0; }

.lane-stack {
  display: grid;
  gap: 0.55rem;
}

.lane-meta {
  padding-top: 0.25rem;
}

.lane-type {
  display: inline-block;
  margin-bottom: 0.18rem;
  color: var(--accent);
  font-size: 0.68rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.lane-meta strong {
  display: block;
  font-size: 0.9rem;
}

.lane-meta p {
  margin: 0.18rem 0 0;
  color: var(--text-muted);
  font-size: 0.77rem;
}

.lane-track {
  min-height: 2.9rem;
}

.lane-grid {
  position: absolute;
  inset: 0;
  border: 1px solid var(--line-soft);
  background:
    linear-gradient(90deg, transparent 24.8%, var(--line-soft) 25%, transparent 25.2%),
    linear-gradient(90deg, transparent 49.8%, var(--line-soft) 50%, transparent 50.2%),
    linear-gradient(90deg, transparent 74.8%, var(--line-soft) 75%, transparent 75.2%);
}

.entry-pill {
  position: absolute;
  top: 0.35rem;
  height: 2.2rem;
  min-width: 3.2rem;
  padding: 0.26rem 0.5rem;
  border: 1px solid var(--line-strong);
  border-left-width: 2px;
  background: var(--panel-strong);
  color: var(--text-strong);
  text-align: left;
  overflow: hidden;
}

.entry-pill.selected {
  outline: 2px solid color-mix(in srgb, var(--accent) 55%, transparent);
  outline-offset: 1px;
}

.entry-title,
.entry-subtitle {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.entry-kind {
  display: block;
  margin-bottom: 0.08rem;
  color: var(--text-muted);
  font-size: 0.62rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.entry-title {
  font-size: 0.77rem;
}

.entry-subtitle {
  margin-top: 0.08rem;
  color: var(--text-muted);
  font-size: 0.68rem;
}

.tone-ok { border-left-color: var(--accent); }
.tone-warn { border-left-color: var(--warn); }
.tone-err { border-left-color: var(--err); }

@media (max-width: 780px) {
  .axis-row,
  .lane-row {
    grid-template-columns: 1fr;
  }
}
</style>
