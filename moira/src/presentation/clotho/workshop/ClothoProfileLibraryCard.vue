<script setup lang="ts">
import type { ProfileDocumentSummary } from '@/projection/clotho'

defineProps<{
  issue: string | null
  loading: {
    list: boolean
    load: boolean
  }
  profiles: ProfileDocumentSummary[]
  selectedProfileId: string | null
}>()

defineEmits<{
  openProfile: [profileId: string]
  refresh: []
  selectNoProfile: []
  startNew: []
}>()
</script>

<template>
  <article class="workshop-card">
    <div class="section-head">
      <div>
        <h3>Profile Library</h3>
        <p class="section-note">Keep multiple JSONC config documents and reopen them by profile id for the next wake.</p>
      </div>

      <div class="section-actions">
        <button class="button-secondary" type="button" :disabled="loading.list" @click="$emit('refresh')">
          Refresh
        </button>
        <button class="button-secondary" type="button" @click="$emit('startNew')">New Profile…</button>
      </div>
    </div>

    <p v-if="issue" class="inline-issue">{{ issue }}</p>

    <div class="profile-grid">
      <button
        type="button"
        class="profile-card profile-card-empty"
        :class="{ selected: selectedProfileId == null }"
        @click="$emit('selectNoProfile')"
      >
        <span class="profile-id">Wake Without Profile</span>
        <span class="profile-path">Omit <code>--config</code> on the next wake.</span>
      </button>

      <button
        v-for="profile in profiles"
        :key="profile.profileId"
        type="button"
        class="profile-card"
        :class="{ selected: profile.profileId === selectedProfileId }"
        :disabled="loading.load"
        @click="$emit('openProfile', profile.profileId)"
      >
        <span class="profile-id mono">{{ profile.profileId }}</span>
        <span class="profile-path mono">{{ profile.profilePath }}</span>
      </button>
    </div>

    <p v-if="!profiles.length" class="field-note empty-note">
      No profile document exists yet. Create one, then click its card later to reopen and edit it.
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

.profile-grid {
  display: grid;
  gap: 0.75rem;
}

.profile-card {
  display: grid;
  gap: 0.35rem;
  padding: 0.8rem;
  border: 1px solid var(--line-soft);
  background: rgba(255, 255, 255, 0.84);
  color: var(--text-strong);
  text-align: left;
}

.profile-card.selected {
  border-color: color-mix(in srgb, var(--accent) 46%, transparent);
  background: var(--accent-soft);
}

.profile-card-empty {
  border-style: dashed;
}

.profile-id {
  font-size: 0.96rem;
  line-height: 1.4;
}

.profile-path {
  color: var(--text-muted);
  font-size: 0.84rem;
  line-height: 1.45;
  word-break: break-word;
}

.field-note {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.88rem;
  line-height: 1.5;
}

.empty-note {
  margin-top: 0.8rem;
}

@media (max-width: 780px) {
  .section-head {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
