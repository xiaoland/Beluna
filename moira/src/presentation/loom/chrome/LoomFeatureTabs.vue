<script setup lang="ts">
type LoomFeatureTab = 'lachesis' | 'atropos' | 'clotho'

const props = defineProps<{
  activeTab: LoomFeatureTab
  runtimePhase: string | null
  selectedBuildId: string | null
  selectedProfileId: string | null
  wakeCount: number
}>()

const emit = defineEmits<{
  select: [tab: LoomFeatureTab]
}>()

const tabMeta: Array<{
  description: string
  id: LoomFeatureTab
  kicker: string
  name: string
}> = [
  {
    id: 'lachesis',
    kicker: 'Observe',
    name: 'Lachesis',
    description: 'Browse wakes, tick timelines, chronology, and raw drilldown.',
  },
  {
    id: 'atropos',
    kicker: 'Supervise',
    name: 'Atropos',
    description: 'Wake, stop, and force-kill the supervised Core runtime.',
  },
  {
    id: 'clotho',
    kicker: 'Prepare',
    name: 'Clotho',
    description: 'Register local builds and curate reusable profile documents.',
  },
]

function tabSummary(tab: LoomFeatureTab): string {
  if (tab === 'lachesis') {
    return props.wakeCount > 0 ? `${props.wakeCount} wake sessions` : 'No wake captured yet'
  }

  if (tab === 'atropos') {
    return `Runtime ${props.runtimePhase ?? 'idle'}`
  }

  if (!props.selectedBuildId) {
    return 'No build selected yet'
  }

  return props.selectedProfileId
    ? `Build ${props.selectedBuildId} + profile ${props.selectedProfileId}`
    : `Build ${props.selectedBuildId} + profile optional`
}
</script>

<template>
  <nav class="tab-shell panel-shell" aria-label="Loom stations">
    <button
      v-for="tab in tabMeta"
      :key="tab.id"
      type="button"
      class="tab-card"
      :class="{ active: tab.id === activeTab }"
      @click="emit('select', tab.id)"
    >
      <span class="tab-kicker">{{ tab.kicker }}</span>
      <strong class="tab-name">{{ tab.name }}</strong>
      <span class="tab-description">{{ tab.description }}</span>
      <span class="tab-summary mono">{{ tabSummary(tab.id) }}</span>
    </button>
  </nav>
</template>

<style scoped>
.tab-shell {
  position: relative;
  z-index: 1;
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.8rem;
  margin-bottom: 1rem;
  padding: 0.8rem;
}

.tab-card {
  display: grid;
  gap: 0.38rem;
  align-content: start;
  min-height: 9rem;
  padding: 0.95rem;
  border: 1px solid var(--line-soft);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.94), rgba(246, 249, 253, 0.96)),
    var(--panel-strong);
  color: var(--text-strong);
  text-align: left;
  transition:
    transform 160ms ease,
    border-color 160ms ease,
    box-shadow 160ms ease;
}

.tab-card:hover {
  transform: translateY(-1px);
  border-color: color-mix(in srgb, var(--accent) 26%, var(--line-strong));
  box-shadow: var(--shadow-card);
}

.tab-card.active {
  border-color: color-mix(in srgb, var(--accent) 48%, var(--line-strong));
  background:
    linear-gradient(180deg, rgba(232, 241, 249, 0.96), rgba(248, 251, 255, 0.98)),
    var(--panel-strong);
}

.tab-kicker {
  color: var(--text-muted);
  font-size: 0.76rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.tab-name {
  font-family: var(--font-display);
  font-size: 1.28rem;
}

.tab-description {
  color: var(--text-muted);
  line-height: 1.5;
}

.tab-summary {
  margin-top: auto;
  color: color-mix(in srgb, var(--accent) 84%, black);
  font-size: 0.82rem;
  line-height: 1.5;
}

@media (max-width: 980px) {
  .tab-shell {
    grid-template-columns: 1fr;
  }

  .tab-card {
    min-height: auto;
  }
}
</style>
