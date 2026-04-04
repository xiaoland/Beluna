<script setup lang="ts">
import type { PublishedReleaseSummary } from '@/projection/clotho'
import LoomDialogShell from '@/presentation/loom/chrome/LoomDialogShell.vue'

const props = defineProps<{
  canInstall: boolean
  issue: string | null
  loading: {
    install: boolean
    list: boolean
  }
  open: boolean
  releases: PublishedReleaseSummary[]
  selectedReleaseKey: string | null
}>()

const emit = defineEmits<{
  close: []
  install: []
  refresh: []
  selectRelease: [releaseKey: string]
}>()

const titleId = 'clotho-install-release-dialog'
</script>

<template>
  <LoomDialogShell :open="open" :title-id="titleId" :dismissible="!loading.install" close-label="Cancel" @close="emit('close')">
    <template #header>
      <p class="dialog-kicker">Clotho / Release Intake</p>
      <h3 :id="titleId">Install published release</h3>
      <p class="dialog-subtitle">
        Discover the current supported release target, verify it against <code>SHA256SUMS</code>, and install it into an
        isolated local directory.
      </p>
    </template>

    <div class="dialog-stack">
      <div class="dialog-actions top-actions">
        <p class="field-note">Current first supported consumer target: <code>aarch64-apple-darwin</code>.</p>
        <button class="button-secondary" type="button" :disabled="loading.list" @click="emit('refresh')">
          Refresh Releases
        </button>
      </div>

      <p v-if="issue" class="inline-issue">{{ issue }}</p>

      <div v-if="releases.length" class="release-grid">
        <button
          v-for="release in releases"
          :key="release.key"
          type="button"
          class="release-card"
          :class="{ selected: release.key === selectedReleaseKey }"
          @click="emit('selectRelease', release.key)"
        >
          <div class="release-head">
            <span class="release-title">{{ release.displayName }}</span>
            <span class="release-meta">{{ release.prerelease ? 'prerelease' : 'release' }}</span>
          </div>
          <span class="release-line mono">{{ release.releaseTag }} / {{ release.rustTargetTriple }}</span>
          <span class="release-line mono">{{ release.archiveAssetName }}</span>
          <span class="release-line mono">{{ release.checksumAssetName }}</span>
          <span class="release-line">{{ release.alreadyInstalled ? 'Already installed locally' : 'Not installed yet' }}</span>
        </button>
      </div>

      <p v-else class="field-note empty-note">
        No compatible published release is visible yet. This usually means #8 has not produced the required asset + checksum pair.
      </p>

      <div class="dialog-actions">
        <p class="field-note">Install performs download, checksum verification, extraction, and manifest write in one Clotho-owned step.</p>
        <button class="button-primary" type="button" :disabled="!canInstall" @click="emit('install')">
          {{ loading.install ? 'Installing…' : 'Install Selected Release' }}
        </button>
      </div>
    </div>
  </LoomDialogShell>
</template>

<style scoped>
.dialog-stack {
  display: grid;
  gap: 0.8rem;
}

.inline-issue {
  margin: 0;
  padding: 0.72rem 0.85rem;
  border: 1px solid rgba(162, 77, 68, 0.2);
  background: rgba(162, 77, 68, 0.08);
  color: var(--err);
}

.release-grid {
  display: grid;
  gap: 0.75rem;
}

.release-card {
  display: grid;
  gap: 0.32rem;
  padding: 0.85rem;
  border: 1px solid var(--line-soft);
  background: rgba(255, 255, 255, 0.84);
  color: var(--text-strong);
  text-align: left;
}

.release-card.selected {
  border-color: color-mix(in srgb, var(--accent) 46%, transparent);
  background: var(--accent-soft);
}

.release-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.75rem;
}

.release-title {
  font-size: 0.96rem;
  line-height: 1.4;
}

.release-meta,
.release-line,
.field-note {
  color: var(--text-muted);
  font-size: 0.84rem;
  line-height: 1.45;
}

.dialog-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.85rem;
  flex-wrap: wrap;
}

.top-actions {
  margin-bottom: 0.1rem;
}

.empty-note {
  margin: 0;
}

@media (max-width: 780px) {
  .dialog-actions,
  .release-head {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
