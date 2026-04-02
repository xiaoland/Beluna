<script setup lang="ts">
import type { ProfileDocumentSummary } from '@/projection/clotho'

defineProps<{
  canSave: boolean
  draft: {
    profileId: string
    contents: string
  }
  issue: string | null
  loading: {
    list: boolean
    load: boolean
    save: boolean
  }
  pathHint: string | null
  profiles: ProfileDocumentSummary[]
  selectedProfileId: string | null
}>()

defineEmits<{
  openProfile: [profileId: string]
  refresh: []
  save: []
  selectNoProfile: []
  startNew: []
  updateField: [field: 'profileId' | 'contents', value: string]
}>()
</script>

<template>
  <article class="control-card profile-card">
    <div class="section-head">
      <div>
        <h3>Clotho Profiles</h3>
        <p class="section-note">Create and edit JSONC profile documents, then choose one for the next wake.</p>
      </div>

      <div class="section-actions">
        <button class="button-secondary" type="button" :disabled="loading.list" @click="$emit('refresh')">
          Refresh
        </button>
        <button class="button-secondary" type="button" @click="$emit('startNew')">
          New Profile
        </button>
      </div>
    </div>

    <div class="selection-strip">
      <button
        type="button"
        class="selection-chip"
        :class="{ selected: selectedProfileId == null }"
        @click="$emit('selectNoProfile')"
      >
        Wake Without Profile
      </button>

      <button
        v-for="profile in profiles"
        :key="profile.profileId"
        type="button"
        class="selection-chip mono"
        :class="{ selected: profile.profileId === selectedProfileId }"
        @click="$emit('openProfile', profile.profileId)"
      >
        {{ profile.profileId }}
      </button>
    </div>

    <p v-if="issue" class="inline-issue">{{ issue }}</p>

    <label class="field">
      <span class="field-label">Profile ID</span>
      <input
        :value="draft.profileId"
        class="field-input mono"
        type="text"
        placeholder="default"
        @input="$emit('updateField', 'profileId', ($event.target as HTMLInputElement).value)"
      />
    </label>

    <p class="field-note">
      {{ pathHint ? `Saved as ${pathHint}` : 'Saved under profiles/<profile-id>.jsonc once the document exists.' }}
    </p>

    <label class="field">
      <span class="field-label">JSONC Document</span>
      <textarea
        :value="draft.contents"
        class="field-input field-textarea mono"
        spellcheck="false"
        @input="$emit('updateField', 'contents', ($event.target as HTMLTextAreaElement).value)"
      />
    </label>

    <div class="action-row">
      <button class="button-primary" type="button" :disabled="!canSave" @click="$emit('save')">
        {{ loading.save ? 'Saving…' : 'Save Profile Document' }}
      </button>
      <p class="field-note">
        Wake stays profile-optional. Open or save a profile to select it for wake, or choose “Wake Without Profile” to
        omit `--config` on the next launch.
      </p>
    </div>
  </article>
</template>

<style scoped>
.control-card {
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

.selection-strip {
  display: flex;
  gap: 0.55rem;
  flex-wrap: wrap;
  margin-bottom: 0.85rem;
}

.selection-chip {
  min-height: 2rem;
  padding: 0.36rem 0.68rem;
  border: 1px solid var(--line-strong);
  background: rgba(255, 255, 255, 0.9);
  color: var(--text-strong);
}

.selection-chip.selected {
  border-color: color-mix(in srgb, var(--accent) 46%, transparent);
  background: var(--accent-soft);
}

.field {
  display: grid;
  gap: 0.35rem;
  margin-bottom: 0.8rem;
}

.field-label {
  color: var(--text-muted);
  font-size: 0.76rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.field-input {
  width: 100%;
  min-height: 2.5rem;
  padding: 0.56rem 0.68rem;
  border: 1px solid var(--line-strong);
  background: #fff;
  color: var(--text-strong);
}

.field-textarea {
  min-height: 13rem;
  resize: vertical;
  line-height: 1.55;
}

.field-input::placeholder {
  color: color-mix(in srgb, var(--text-muted) 78%, white);
}

.field-note {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.88rem;
  line-height: 1.5;
}

.inline-issue {
  margin: 0 0 0.8rem;
  padding: 0.72rem 0.85rem;
  border: 1px solid rgba(162, 77, 68, 0.2);
  background: rgba(162, 77, 68, 0.08);
  color: var(--err);
}

.action-row {
  display: flex;
  align-items: center;
  gap: 0.85rem;
  flex-wrap: wrap;
}

@media (max-width: 780px) {
  .action-row,
  .section-head {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
