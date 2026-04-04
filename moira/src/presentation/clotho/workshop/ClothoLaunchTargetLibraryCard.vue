<script setup lang="ts">
import { computed } from 'vue'

import type { LaunchTargetSummary } from '@/projection/clotho'

const props = defineProps<{
  issue: string | null
  launchTargets: LaunchTargetSummary[]
  loading: {
    forge: boolean
    install: boolean
    listReleases: boolean
    listTargets: boolean
    register: boolean
  }
  selectedTargetKey: string | null
}>()

const emit = defineEmits<{
  openForge: []
  openInstall: []
  openRegister: []
  refreshTargets: []
  selectTarget: [target: LaunchTargetSummary['target']]
}>()

const selectedTarget = computed<LaunchTargetSummary | null>(
  () => props.launchTargets.find((target) => target.key === props.selectedTargetKey) ?? null,
)

const selectionSummary = computed(() => {
  if (!selectedTarget.value) {
    return 'Choose exactly one launch target here. Atropos wakes whichever target Clotho currently has selected.'
  }

  return `${selectedTarget.value.label} is the single target armed for the next wake.`
})

function statusCopy(target: LaunchTargetSummary): string {
  const parts: string[] = [target.provenance]
  parts.push(target.readiness)
  if (target.checksumVerified) {
    parts.push('checksum verified')
  }
  return parts.join(' · ')
}
</script>

<template>
  <article class="workshop-card">
    <div class="section-head">
      <div>
        <h3>Launch Target Library</h3>
        <p class="section-note">
          Keep registered builds, forged local builds, and installed releases inside one Clotho-owned target library.
        </p>
      </div>

      <div class="section-actions">
        <button class="button-secondary" type="button" :disabled="loading.listTargets" @click="$emit('refreshTargets')">
          Refresh
        </button>
        <button class="button-secondary" type="button" :disabled="loading.register" @click="$emit('openRegister')">
          Register…
        </button>
        <button class="button-secondary" type="button" :disabled="loading.forge" @click="$emit('openForge')">
          Forge…
        </button>
        <button class="button-secondary" type="button" :disabled="loading.install || loading.listReleases" @click="$emit('openInstall')">
          Install Release…
        </button>
      </div>
    </div>

    <p v-if="issue" class="inline-issue">{{ issue }}</p>

    <div class="selection-banner" :class="{ empty: !selectedTarget }">
      <span class="selection-kicker">Single Select</span>
      <strong class="mono">{{ selectedTarget?.label ?? 'No launch target selected yet' }}</strong>
      <span class="selection-copy">{{ selectionSummary }}</span>
    </div>

    <div v-if="launchTargets.length" class="target-grid" role="radiogroup" aria-label="Launch target library">
      <button
        v-for="target in launchTargets"
        :key="target.key"
        type="button"
        class="target-card"
        :class="{ selected: target.key === selectedTargetKey, stale: target.readiness === 'stale' }"
        role="radio"
        :aria-checked="target.key === selectedTargetKey"
        @click="emit('selectTarget', target.target)"
      >
        <div class="target-head">
          <span class="target-label mono">{{ target.label }}</span>
          <span class="target-status">{{ statusCopy(target) }}</span>
        </div>

        <span class="target-selection" :class="{ active: target.key === selectedTargetKey }">
          {{ target.key === selectedTargetKey ? 'Selected for next wake' : 'Click to select' }}
        </span>

        <span class="target-path mono">{{ target.executablePath ?? target.issue ?? 'Executable pending' }}</span>
        <span v-if="target.sourceDir" class="target-meta mono">Source {{ target.sourceDir }}</span>
        <span v-else-if="target.installDir" class="target-meta mono">Install {{ target.installDir }}</span>
        <span v-if="target.issue" class="target-issue">{{ target.issue }}</span>
      </button>
    </div>

    <div v-else class="empty-state build-empty">
      No launch target is ready yet. Register a local executable, forge from source, or install a published release.
    </div>

    <p class="field-note build-note">
      Target manifests are durable app-local Clotho truth, but the currently selected launch target remains session-local Loom
      state until a later persistence slice lands.
    </p>
  </article>
</template>

<style scoped>
.workshop-card {
  min-width: 0;
  padding: 1rem;
  border: 1px solid var(--line-soft);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.9), rgba(247, 249, 252, 0.94)),
    var(--panel-strong);
}

.section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.95rem;
}

.section-head h3 {
  margin: 0;
  font-family: var(--font-display);
  font-size: 1.08rem;
}

.section-note {
  margin: 0.24rem 0 0;
  color: var(--text-muted);
  font-size: 0.88rem;
  line-height: 1.45;
}

.section-actions {
  display: flex;
  gap: 0.55rem;
  flex-wrap: wrap;
}

.inline-issue {
  margin: 0 0 0.8rem;
  padding: 0.72rem 0.85rem;
  border: 1px solid rgba(162, 77, 68, 0.2);
  background: rgba(162, 77, 68, 0.08);
  color: var(--err);
}

.selection-banner {
  display: grid;
  gap: 0.18rem;
  margin-bottom: 0.85rem;
  padding: 0.8rem 0.88rem;
  border: 1px solid color-mix(in srgb, var(--accent) 22%, transparent);
  background: color-mix(in srgb, var(--accent-soft) 65%, white);
}

.selection-banner.empty {
  border-style: dashed;
  background: rgba(255, 255, 255, 0.72);
}

.selection-kicker {
  color: var(--text-muted);
  font-size: 0.74rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.selection-copy {
  color: var(--text-muted);
  font-size: 0.86rem;
  line-height: 1.45;
}

.target-grid {
  display: grid;
  gap: 0.75rem;
}

.target-card {
  display: grid;
  gap: 0.38rem;
  padding: 0.85rem;
  border: 1px solid var(--line-soft);
  background: rgba(255, 255, 255, 0.84);
  color: var(--text-strong);
  box-shadow: var(--shadow-card);
  text-align: left;
  transition:
    border-color 140ms ease,
    background-color 140ms ease,
    transform 140ms ease,
    box-shadow 140ms ease;
}

.target-card.selected {
  border-color: color-mix(in srgb, var(--accent) 46%, transparent);
  background: var(--accent-soft);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 20%, transparent);
  transform: translateY(-1px);
}

.target-card.stale {
  border-style: dashed;
}

.target-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.75rem;
}

.target-label {
  font-size: 0.95rem;
  line-height: 1.4;
}

.target-status {
  color: var(--text-muted);
  font-size: 0.78rem;
  line-height: 1.45;
}

.target-selection {
  color: var(--text-muted);
  font-size: 0.78rem;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.target-selection.active {
  color: var(--accent);
  font-weight: 700;
}

.target-path,
.target-meta {
  color: var(--text-muted);
  font-size: 0.84rem;
  line-height: 1.45;
  word-break: break-word;
}

.target-issue {
  color: var(--err);
  font-size: 0.82rem;
  line-height: 1.45;
}

.build-empty {
  padding: 1rem;
  border: 1px dashed var(--line-strong);
  background: rgba(255, 255, 255, 0.72);
}

.build-note {
  margin-top: 0.8rem;
}

.field-note {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.88rem;
  line-height: 1.5;
}

@media (max-width: 780px) {
  .section-head,
  .target-head {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
