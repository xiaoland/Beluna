<script setup lang="ts">
import { computed } from 'vue'

import LoomDialogShell from '@/presentation/loom/chrome/LoomDialogShell.vue'

const props = defineProps<{
  canSave: boolean
  draft: {
    profileId: string
    coreConfig: string
    envFiles: Array<{
      id: string
      path: string
      required: boolean
    }>
    inlineEnvironment: Array<{
      id: string
      key: string
      value: string
    }>
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
  addEnvFile: []
  addInlineEnvironment: []
  close: []
  removeEnvFile: [rowId: string]
  removeInlineEnvironment: [rowId: string]
  save: []
  updateEnvFile: [field: 'path' | 'required', rowId: string, value: string | boolean]
  updateField: [field: 'profileId' | 'coreConfig', value: string]
  updateInlineEnvironment: [field: 'key' | 'value', rowId: string, value: string]
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
        <span class="field-label">Core Config</span>
        <textarea
          :value="props.draft.coreConfig"
          class="field-input field-textarea mono"
          spellcheck="false"
          @input="emit('updateField', 'coreConfig', ($event.target as HTMLTextAreaElement).value)"
        />
      </label>

      <section class="group">
        <div class="group-head">
          <div>
            <p class="field-label">Environment Files</p>
            <p class="field-note">Relative paths resolve from the saved profile document directory.</p>
          </div>
          <button class="button-secondary compact-button" type="button" @click="emit('addEnvFile')">Add Env File</button>
        </div>

        <div v-if="props.draft.envFiles.length > 0" class="group-stack">
          <div v-for="entry in props.draft.envFiles" :key="entry.id" class="env-file-row">
            <input
              :value="entry.path"
              class="field-input mono"
              type="text"
              placeholder="./local.env"
              @input="emit('updateEnvFile', 'path', entry.id, ($event.target as HTMLInputElement).value)"
            />
            <label class="toggle">
              <input
                :checked="entry.required"
                type="checkbox"
                @change="emit('updateEnvFile', 'required', entry.id, ($event.target as HTMLInputElement).checked)"
              />
              <span>Required</span>
            </label>
            <button class="button-secondary compact-button" type="button" @click="emit('removeEnvFile', entry.id)">
              Remove
            </button>
          </div>
        </div>
        <p v-else class="field-note">No env file sources configured.</p>
      </section>

      <section class="group">
        <div class="group-head">
          <div>
            <p class="field-label">Inline Environment</p>
            <p class="field-note">Inline values are written directly into the profile document.</p>
          </div>
          <button class="button-secondary compact-button" type="button" @click="emit('addInlineEnvironment')">
            Add Variable
          </button>
        </div>

        <div v-if="props.draft.inlineEnvironment.length > 0" class="group-stack">
          <div v-for="entry in props.draft.inlineEnvironment" :key="entry.id" class="inline-env-row">
            <input
              :value="entry.key"
              class="field-input mono"
              type="text"
              placeholder="OPENAI_API_KEY"
              @input="emit('updateInlineEnvironment', 'key', entry.id, ($event.target as HTMLInputElement).value)"
            />
            <input
              :value="entry.value"
              class="field-input mono"
              type="text"
              placeholder="sk-..."
              @input="emit('updateInlineEnvironment', 'value', entry.id, ($event.target as HTMLInputElement).value)"
            />
            <button class="button-secondary compact-button" type="button" @click="emit('removeInlineEnvironment', entry.id)">
              Remove
            </button>
          </div>
        </div>
        <p v-else class="field-note">No inline environment variables configured.</p>
      </section>

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
  min-height: 12rem;
  resize: vertical;
  line-height: 1.55;
}

.group {
  display: grid;
  gap: 0.6rem;
}

.group-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.85rem;
  flex-wrap: wrap;
}

.group-stack {
  display: grid;
  gap: 0.55rem;
}

.env-file-row,
.inline-env-row {
  display: grid;
  gap: 0.55rem;
}

.env-file-row {
  grid-template-columns: minmax(0, 1fr) auto auto;
  align-items: center;
}

.inline-env-row {
  grid-template-columns: minmax(0, 0.95fr) minmax(0, 1.05fr) auto;
}

.toggle {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  color: var(--text-muted);
  font-size: 0.9rem;
}

.compact-button {
  align-self: start;
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
  .env-file-row,
  .inline-env-row {
    grid-template-columns: 1fr;
  }

  .dialog-actions {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
