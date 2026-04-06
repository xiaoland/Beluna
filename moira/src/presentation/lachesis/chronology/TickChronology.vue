<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import ChronologyEntryDialog from '@/presentation/lachesis/chronology/ChronologyEntryDialog.vue'
import ChronologyLaneTrack from '@/presentation/lachesis/chronology/ChronologyLaneTrack.vue'
import { formatWhen } from '@/presentation/format'
import type { ChronologyEntry, TickChronology } from '@/projection/lachesis/models'

const props = defineProps<{
  chronology: TickChronology
}>()

const selectedRawEventId = ref<string | null>(null)

const selectedEntry = computed<ChronologyEntry | null>(() => {
  if (!selectedRawEventId.value) {
    return null
  }

  for (const lane of props.chronology.lanes) {
    const match = lane.entries.find((entry) => entry.rawEventId === selectedRawEventId.value)
    if (match) {
      return match
    }
  }

  return null
})

const axisModeLabel = computed(() =>
  props.chronology.usesObservedTime ? 'Observed-time axis' : 'Event-order axis',
)

watch(
  () => props.chronology,
  (chronology) => {
    const activeId = selectedRawEventId.value
    if (!activeId) {
      return
    }

    const stillPresent = chronology.lanes.some((lane) =>
      lane.entries.some((entry) => entry.rawEventId === activeId),
    )

    if (!stillPresent) {
      selectedRawEventId.value = null
    }
  },
)

function selectEntry(entry: ChronologyEntry): void {
  selectedRawEventId.value = entry.rawEventId
}

function clearSelection(): void {
  selectedRawEventId.value = null
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
          <h3>Cortex Timeline</h3>
          <p>
            {{ chronology.lanes.length }} lanes · {{ chronology.eventCount }} events · {{ axisModeLabel }}
          </p>
        </div>
        <div class="axis-summary">
          <span>Start {{ formatWhen(chronology.firstObservedAt) }}</span>
          <span>End {{ formatWhen(chronology.lastObservedAt) }}</span>
        </div>
      </div>

      <p class="chronology-note">
        Click an interval or event to inspect its source events and related activity in a popup.
      </p>

      <ChronologyLaneTrack
        :chronology="chronology"
        :selected-raw-event-id="selectedEntry?.rawEventId ?? null"
        @select="selectEntry"
      />

      <ChronologyEntryDialog :entry="selectedEntry" @close="clearSelection" />
    </template>
  </section>
</template>

<style scoped>
.chronology-shell {
  display: grid;
  gap: 0.8rem;
}

.section-empty,
.chronology-note {
  color: var(--text-muted);
}

.chronology-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.chronology-head h3 {
  margin: 0;
  font-size: 1rem;
}

.chronology-head p,
.axis-summary {
  margin: 0.28rem 0 0;
  color: var(--text-muted);
  font-size: 0.83rem;
}

.axis-summary {
  display: grid;
  gap: 0.22rem;
  text-align: right;
}

.chronology-note {
  margin: 0;
  font-size: 0.82rem;
}

@media (max-width: 780px) {
  .chronology-head {
    flex-direction: column;
  }

  .axis-summary {
    text-align: left;
  }
}
</style>
