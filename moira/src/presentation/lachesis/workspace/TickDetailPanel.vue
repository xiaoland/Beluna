<script setup lang="ts">
import { computed } from 'vue'
import TickChronology from '@/presentation/lachesis/chronology/TickChronology.vue'
import RawEventInspector from '@/presentation/lachesis/inspectors/RawEventInspector.vue'
import JsonSectionGroup from '@/presentation/loom/shared/JsonSectionGroup.vue'
import { formatWhen } from '@/presentation/format'
import { summarizeEntry } from '@/projection/lachesis/labels'
import { narrativeSections } from '@/projection/lachesis/narratives'
import type { TickDetail } from '@/projection/lachesis/models'
import type { LachesisDetailTab } from '@/query/lachesis/state'

const props = defineProps<{
  detail: TickDetail | null
  loading: boolean
  tab: LachesisDetailTab
}>()

const emit = defineEmits<{
  'update:tab': [tab: LachesisDetailTab]
}>()

const tabs: Array<{ key: LachesisDetailTab; label: string }> = [
  { key: 'chronology', label: 'Chronology' },
  { key: 'cortex', label: 'Cortex' },
  { key: 'stem', label: 'Stem' },
  { key: 'spine', label: 'Spine' },
  { key: 'raw', label: 'Raw' },
]

const sections = computed(() => {
  if (!props.detail || props.tab === 'raw' || props.tab === 'chronology') {
    return []
  }

  return narrativeSections(props.detail, props.tab)
})
</script>

<template>
  <section class="panel-shell">
    <div class="panel-head detail-head">
      <div>
        <p class="panel-kicker">Selected Tick</p>
        <h2 class="panel-title">Tick Detail</h2>
        <p class="panel-subtitle">
          {{
            detail
              ? `Wake ${detail.runId} · Tick ${detail.tick} · ${detail.rawEvents.length} raw events`
              : 'Cortex, Stem, Spine, and raw drilldown for one selected tick.'
          }}
        </p>
      </div>
      <div class="detail-meta">
        <span v-if="detail">Last observed {{ formatWhen(detail.rawEvents[detail.rawEvents.length - 1]?.observedAt ?? null) }}</span>
      </div>
    </div>

    <div class="tabs">
      <button
        v-for="item in tabs"
        :key="item.key"
        type="button"
        class="tab"
        :class="{ active: item.key === tab }"
        @click="emit('update:tab', item.key)"
      >
        {{ item.label }}
      </button>
    </div>

    <div v-if="loading && !detail" class="empty-state">Loading tick detail…</div>
    <div v-else-if="!detail" class="empty-state">
      Select a tick from the timeline to inspect its chronology, narrative, and raw events.
    </div>

    <div v-else-if="tab === 'chronology'" class="detail-body">
      <TickChronology :chronology="detail.chronology" />
    </div>

    <div v-else-if="tab === 'raw'" class="detail-body">
      <RawEventInspector :raw-events="detail.rawEvents" />
    </div>

    <div v-else class="detail-body">
      <section v-for="section in sections" :key="section.title" class="narrative-section">
        <div class="section-head">
          <div>
            <h3>{{ section.title }}</h3>
            <p>{{ section.hint }}</p>
          </div>
          <span class="section-count">
            {{ section.single ? '1 snapshot' : `${section.items.length} entr${section.items.length === 1 ? 'y' : 'ies'}` }}
          </span>
        </div>

        <article v-if="section.single" class="entry-card">
          <header>{{ summarizeEntry(section.single) }}</header>
          <JsonSectionGroup
            :sections="[
              {
                key: `${section.title}-single`,
                title: 'Structured payload',
                value: section.single,
                defaultOpen: true,
              },
            ]"
          />
        </article>

        <div v-else-if="section.items.length" class="entry-stack">
          <article v-for="(entry, index) in section.items" :key="index" class="entry-card">
            <header>{{ summarizeEntry(entry) }}</header>
            <JsonSectionGroup
              :sections="[
                {
                  key: `${section.title}-${index}`,
                  title: 'Structured payload',
                  value: entry,
                  defaultOpen: index === 0,
                },
              ]"
            />
          </article>
        </div>

        <div v-else class="section-empty">
          No entries for this section were reconstructed from the current tick detail response.
        </div>
      </section>
    </div>
  </section>
</template>

<style scoped>
.detail-head {
  padding-bottom: 0.5rem;
}

.detail-meta {
  color: var(--text-muted);
  font-size: 0.86rem;
}

.tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  padding: 0 0.9rem 0.65rem;
}

.tab {
  min-height: 2.25rem;
  padding: 0.42rem 0.72rem;
  border: 1px solid var(--line-soft);
  background: var(--panel);
  color: var(--text-muted);
}

.tab.active {
  color: var(--text-strong);
  border-color: color-mix(in srgb, var(--accent) 34%, var(--line-strong));
  background: color-mix(in srgb, var(--accent) 6%, white);
}

.detail-body {
  display: grid;
  gap: 0.7rem;
  padding: 0 0.9rem 0.9rem;
}

.narrative-section {
  padding: 0.78rem;
  background: var(--surface-subtle);
  border: 1px solid var(--line-soft);
}

.section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.9rem;
  margin-bottom: 0.8rem;
}

.section-head h3 {
  margin: 0;
  font-size: 1rem;
}

.section-head p {
  margin: 0.3rem 0 0;
  color: var(--text-muted);
  line-height: 1.45;
}

.section-count {
  color: var(--text-muted);
  font-size: 0.82rem;
  white-space: nowrap;
}

.entry-stack {
  display: grid;
  gap: 0.75rem;
}

.entry-card {
  display: grid;
  gap: 0.6rem;
  padding: 0.72rem;
  border: 1px solid var(--line-soft);
  background: var(--panel);
}

.entry-card header {
  margin-bottom: 0.55rem;
  font-weight: 600;
}

.section-empty {
  color: var(--text-muted);
}
</style>
