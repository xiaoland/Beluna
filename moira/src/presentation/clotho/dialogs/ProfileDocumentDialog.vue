<script setup lang="ts">
import { computed } from 'vue'

import LoomDialogShell from '@/presentation/loom/chrome/LoomDialogShell.vue'

const props = defineProps<{
  canSave: boolean
  draft: {
    profileId: string
    contents: string
  }
  issue: string | null
  loading: {
    load: boolean
    save: boolean
  }
  open: boolean
  pathHint: string | null
}>()

const emit = defineEmits<{
  close: []
  save: []
  updateField: [field: 'profileId' | 'contents', value: string]
}>()

const titleId = 'clotho-profile-document-dialog'
const dialogTitle = computed(() => (props.draft.profileId.trim().length > 0 ? 'Edit profile document' : 'Create profile document'))
</script>

<template>
  <LoomDialogShell
    :open="open"
    :title-id="titleId"
    :dismissible="!loading.load && !loading.save"
    close-label="Cancel"
    @close="emit('close')"
  >
    <template #header>
      <p class="dialog-kicker">Clotho / Profiles</p>
      <h3 :id="titleId">{{ dialogTitle }}</h3>
      <p class="dialog-subtitle">
        Profile id is the logical key. Clotho maps it to <code>profiles/&lt;profile-id&gt;.jsonc</code> inside app-local
        storage, so it is not the same thing as a raw filesystem path.
      </p>
    </template>

    <div class="dialog-stack">
      <p v-if="issue" class="inline-issue">{{ issue }}</p>

      <label class="field">
        <span class="field-label">Profile ID</span>
        <input
          :value="props.draft.profileId"
          class="field-input mono"
          type="text"
          placeholder="default"
          @input="emit('updateField', 'profileId', ($event.target as HTMLInputElement).value)"
        />
      </label>

      <p class="field-note">
        {{ pathHint ? `Saved as ${pathHint}` : 'Saved under profiles/<profile-id>.jsonc once the document exists.' }}
      </p>

      <label class="field">
        <span class="field-label">JSONC Document</span>
        <textarea
          :value="props.draft.contents"
          class="field-input field-textarea mono"
          spellcheck="false"
          @input="emit('updateField', 'contents', ($event.target as HTMLTextAreaElement).value)"
        />
      </label>

      <div class="dialog-actions">
        <p class="field-note">
          Saving also selects this profile for the next wake. Use “Wake Without Profile” in the library to make the
          next launch omit <code>--config</code>.
        </p>
        <button class="button-primary" type="button" :disabled="!canSave" @click="emit('save')">
          {{ loading.save ? 'Saving…' : 'Save Profile Document' }}
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

.field {
  display: grid;
  gap: 0.35rem;
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

.field-input::placeholder {
  color: color-mix(in srgb, var(--text-muted) 78%, white);
}

.field-textarea {
  min-height: 16rem;
  resize: vertical;
  line-height: 1.55;
}

.dialog-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.85rem;
  flex-wrap: wrap;
}

.field-note {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.88rem;
  line-height: 1.5;
}

@media (max-width: 780px) {
  .dialog-actions {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
