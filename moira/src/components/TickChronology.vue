<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { formatWhen, rawEventHeadline } from '@/presenters'
import type { ChronologyEntry, TickChronology } from '@/types'

const props = defineProps<{
  chronology: TickChronology
}>()

const selectedRawEventId = ref<string | null>(null)

const selectedEntry = computed<ChronologyEntry | null>(() => {
  if (!selectedRawEventId.value) {
    return props.chronology.lanes[0]?.entries[0] ?? null
  }

  for (const lane of props.chronology.lanes) {
    const match = lane.entries.find((entry) => entry.rawEventId === selectedRawEventId.value)
    if (match) {
      return match
    }
  }

  return props.chronology.lanes[0]?.entries[0] ?? null
})

const axisModeLabel = computed(() =>
  props.chronology.usesObservedTime ? 'Observed-time axis' : 'Event-order axis',
)

watch(
  () => props.chronology,
  (chronology) => {
    selectedRawEventId.value = chronology.lanes[0]?.entries[0]?.rawEventId ?? null
  },
  { immediate: true },
)

function selectEntry(entry: ChronologyEntry): void {
  selectedRawEventId.value = entry.rawEventId
}

function pretty(value: unknown): string {
  return JSON.stringify(value, null, 2) ?? 'null'
}

function laneLabel(type: string): string {
  return type.replace(/-/g, ' ')
}

function entryCountLabel(entry: ChronologyEntry): string {
  const relatedLabel = entry.relatedEvents.length
    ? `${entry.relatedEvents.length} related`
    : 'no linked detail'
  return `${entry.sourceEvents.length} source · ${relatedLabel}`
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
  <section class="chronology-shell">
    <div v-if="!chronology.lanes.length" class="section-empty">
      No laneable events were reconstructed for this tick yet.
    </div>

    <template v-else>
      <div class="chronology-head">
        <div>
          <h3>Humane Chronology</h3>
          <p>
            {{ chronology.lanes.length }} lanes · {{ chronology.eventCount }} events · {{ axisModeLabel }}
          </p>
        </div>
        <div class="axis-summary">
          <span>Start {{ formatWhen(chronology.firstObservedAt) }}</span>
          <span>End {{ formatWhen(chronology.lastObservedAt) }}</span>
        </div>
      </div>

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
                :class="[toneClass(entry), { selected: entry.rawEventId === selectedEntry?.rawEventId }]"
                :style="{
                  left: `${entry.position * 100}%`,
                  width: `${Math.max((entry.endPosition - entry.position) * 100, 4)}%`,
                }"
                @click="selectEntry(entry)"
              >
                <span class="entry-kind">{{ entry.entryType === 'interval' ? 'interval' : 'event' }}</span>
                <span class="entry-title">{{ entry.title }}</span>
                <span v-if="entry.subtitle" class="entry-subtitle">{{ entry.subtitle }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>

      <div v-if="selectedEntry" class="chronology-inspector">
        <div class="inspector-head">
          <div>
            <h4>{{ selectedEntry.title }}</h4>
            <p>
              {{ selectedEntry.event.subsystem ?? 'unknown subsystem' }} /
              {{ selectedEntry.family ?? 'unknown family' }} · {{ entryCountLabel(selectedEntry) }}
            </p>
          </div>
          <div class="inspector-meta">
            <span>{{ selectedEntry.event.severityText ?? 'INFO' }}</span>
            <span>{{ formatWhen(selectedEntry.event.observedAt) }}</span>
          </div>
        </div>

        <div class="inspector-grid inspector-grid-stack">
          <article class="inspector-card">
            <h5>Selected Entry</h5>
            <pre>{{ pretty({
              title: selectedEntry.title,
              subtitle: selectedEntry.subtitle,
              entry_type: selectedEntry.entryType,
              source_event_ids: selectedEntry.sourceEvents.map((event) => event.rawEventId),
              related_event_ids: selectedEntry.relatedEvents.map((event) => event.rawEventId),
            }) }}</pre>
          </article>

          <article class="inspector-card">
            <h5>Source Events</h5>
            <div class="event-stack">
              <section
                v-for="sourceEvent in selectedEntry.sourceEvents"
                :key="sourceEvent.rawEventId"
                class="event-card"
              >
                <header>{{ rawEventHeadline(sourceEvent) }}</header>
                <pre>{{ pretty(sourceEvent.payload) }}</pre>
              </section>
            </div>
          </article>

          <article v-if="selectedEntry.relatedEvents.length" class="inspector-card">
            <h5>Related Activity</h5>
            <div class="event-stack">
              <section
                v-for="relatedEvent in selectedEntry.relatedEvents"
                :key="relatedEvent.rawEventId"
                class="event-card"
              >
                <header>{{ rawEventHeadline(relatedEvent) }}</header>
                <pre>{{ pretty(relatedEvent.payload) }}</pre>
              </section>
            </div>
          </article>

          <article class="inspector-card">
            <h5>Attributes</h5>
            <pre>{{ pretty(selectedEntry.event.attributes) }}</pre>
          </article>
        </div>
      </div>
    </template>
  </section>
</template>

<style scoped>
.chronology-shell {
  display: grid;
  gap: 0.8rem;
}

.section-empty {
  color: var(--text-muted);
}

.chronology-head,
.inspector-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.chronology-head h3,
.inspector-head h4 {
  margin: 0;
  font-size: 1rem;
}

.chronology-head p,
.inspector-head p,
.axis-summary,
.inspector-meta {
  margin: 0.28rem 0 0;
  color: var(--text-muted);
  font-size: 0.83rem;
}

.axis-summary,
.inspector-meta {
  display: grid;
  gap: 0.22rem;
  text-align: right;
}

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
  color: var(--accent-soft);
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
  outline: 1px solid var(--accent);
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

.chronology-inspector {
  border: 1px solid var(--line-soft);
  padding: 0.75rem;
}

.inspector-grid {
  display: grid;
  gap: 0.6rem;
  margin-top: 0.7rem;
}

.inspector-grid-stack {
  grid-template-columns: 1fr;
}

.inspector-card {
  border: 1px solid var(--line-soft);
  padding: 0.7rem;
  background: var(--panel-strong);
}

.inspector-card h5 {
  margin: 0 0 0.45rem;
  font-size: 0.78rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.event-stack {
  display: grid;
  gap: 0.65rem;
}

.event-card {
  padding: 0.68rem;
  border: 1px solid var(--line-soft);
  background: var(--panel);
}

.event-card header {
  margin-bottom: 0.45rem;
  font-weight: 600;
}

@media (max-width: 780px) {
  .chronology-head,
  .inspector-head {
    flex-direction: column;
  }

  .axis-summary,
  .inspector-meta {
    text-align: left;
  }

  .axis-row,
  .lane-row {
    grid-template-columns: 1fr;
  }
}
</style>
