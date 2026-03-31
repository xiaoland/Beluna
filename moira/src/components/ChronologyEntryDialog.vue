<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, watch } from 'vue'
import JsonSectionGroup from '@/components/JsonSectionGroup.vue'
import { jsonSectionsForEvent } from '@/json-sections'
import { formatWhen, rawEventHeadline } from '@/presenters'
import type { ChronologyEntry, RawEvent } from '@/types'

const props = defineProps<{
  entry: ChronologyEntry | null
}>()

const emit = defineEmits<{
  close: []
}>()

const dialogTitleId = computed(() =>
  props.entry ? `chronology-entry-dialog-${props.entry.rawEventId}` : undefined,
)

const entryProjectionSections = computed(() => {
  if (!props.entry) {
    return []
  }

  return [
    {
      key: 'projection',
      title: 'Entry Projection',
      value: {
        title: props.entry.title,
        subtitle: props.entry.subtitle,
        entry_type: props.entry.entryType,
        source_event_ids: props.entry.sourceEvents.map((event) => event.rawEventId),
        related_event_ids: props.entry.relatedEvents.map((event) => event.rawEventId),
      },
      defaultOpen: true,
    },
  ]
})

watch(
  () => props.entry,
  (entry) => {
    if (typeof document === 'undefined') {
      return
    }

    document.body.style.overflow = entry ? 'hidden' : ''
  },
)

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  if (typeof document !== 'undefined') {
    document.body.style.overflow = ''
  }

  window.removeEventListener('keydown', handleKeydown)
})

function handleKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && props.entry) {
    emit('close')
  }
}

function eventMeta(event: RawEvent): string {
  return [
    event.subsystem ?? 'unknown subsystem',
    event.family ?? 'unknown family',
    formatWhen(event.observedAt),
  ].join(' · ')
}
</script>

<template>
  <Teleport to="body">
    <div v-if="entry" class="dialog-overlay" @click.self="emit('close')">
      <section
        class="dialog-shell"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="dialogTitleId"
      >
        <header class="dialog-head">
          <div>
            <p class="dialog-kicker">
              {{ entry.event.subsystem ?? 'unknown subsystem' }} / {{ entry.family ?? 'unknown family' }}
            </p>
            <h3 :id="dialogTitleId">{{ entry.title }}</h3>
            <p class="dialog-subtitle">
              {{ entry.subtitle ?? 'Selected chronology entry' }} ·
              {{ entry.sourceEvents.length }} source ·
              {{ entry.relatedEvents.length }} related
            </p>
          </div>

          <button type="button" class="button-secondary dialog-close" @click="emit('close')">
            Close
          </button>
        </header>

        <div class="dialog-body">
          <section class="dialog-card">
            <h4>Entry Context</h4>
            <JsonSectionGroup :sections="entryProjectionSections" />
          </section>

          <section class="dialog-card">
            <div class="dialog-section-head">
              <h4>Source Events</h4>
              <span>{{ entry.sourceEvents.length }}</span>
            </div>

            <div class="event-stack">
              <article v-for="sourceEvent in entry.sourceEvents" :key="sourceEvent.rawEventId" class="event-card">
                <header class="event-head">
                  <div>
                    <strong>{{ rawEventHeadline(sourceEvent) }}</strong>
                    <p>{{ eventMeta(sourceEvent) }}</p>
                  </div>
                  <span class="event-severity">{{ sourceEvent.severityText ?? 'INFO' }}</span>
                </header>

                <JsonSectionGroup :sections="jsonSectionsForEvent(sourceEvent, { openPayload: true })" />
              </article>
            </div>
          </section>

          <section v-if="entry.relatedEvents.length" class="dialog-card">
            <div class="dialog-section-head">
              <h4>Related Activity</h4>
              <span>{{ entry.relatedEvents.length }}</span>
            </div>

            <div class="event-stack">
              <article
                v-for="relatedEvent in entry.relatedEvents"
                :key="relatedEvent.rawEventId"
                class="event-card"
              >
                <header class="event-head">
                  <div>
                    <strong>{{ rawEventHeadline(relatedEvent) }}</strong>
                    <p>{{ eventMeta(relatedEvent) }}</p>
                  </div>
                  <span class="event-severity">{{ relatedEvent.severityText ?? 'INFO' }}</span>
                </header>

                <JsonSectionGroup :sections="jsonSectionsForEvent(relatedEvent, { openPayload: true })" />
              </article>
            </div>
          </section>
        </div>
      </section>
    </div>
  </Teleport>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 40;
  display: grid;
  place-items: center;
  padding: 1.2rem;
  background: var(--overlay);
}

.dialog-shell {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  width: min(76rem, 100%);
  height: min(88vh, 64rem);
  max-height: min(88vh, 64rem);
  overflow: hidden;
  border: 1px solid var(--line-strong);
  background: var(--panel-strong);
}

.dialog-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem 1rem 0.85rem;
  border-bottom: 1px solid var(--line-soft);
}

.dialog-kicker,
.dialog-subtitle,
.event-head p {
  margin: 0.24rem 0 0;
  color: var(--text-muted);
  font-size: 0.84rem;
}

.dialog-head h3,
.dialog-card h4 {
  margin: 0;
}

.dialog-close {
  min-width: 5.4rem;
}

.dialog-body {
  display: grid;
  min-height: 0;
  align-content: start;
  gap: 0.8rem;
  padding: 1rem;
  overflow: auto;
  overscroll-behavior: contain;
}

.dialog-card {
  display: grid;
  gap: 0.75rem;
  padding: 0.85rem;
  border: 1px solid var(--line-soft);
  background: var(--surface-subtle);
}

.dialog-section-head,
.event-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.dialog-section-head span,
.event-severity {
  color: var(--text-muted);
  font-size: 0.78rem;
  white-space: nowrap;
}

.event-stack {
  display: grid;
  gap: 0.7rem;
}

.event-card {
  display: grid;
  gap: 0.65rem;
  padding: 0.75rem;
  border: 1px solid var(--line-soft);
  background: var(--panel-strong);
}

@media (max-width: 780px) {
  .dialog-overlay {
    padding: 0.6rem;
  }

  .dialog-head,
  .dialog-section-head,
  .event-head {
    flex-direction: column;
  }
}
</style>
