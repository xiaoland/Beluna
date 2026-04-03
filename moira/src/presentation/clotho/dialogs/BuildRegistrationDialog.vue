<script setup lang="ts">
import LoomDialogShell from '@/presentation/loom/chrome/LoomDialogShell.vue'

const props = defineProps<{
  canRegister: boolean
  draft: {
    buildId: string
    executablePath: string
    workingDir: string
    sourceDir: string
  }
  issue: string | null
  loading: {
    register: boolean
  }
  open: boolean
}>()

const emit = defineEmits<{
  close: []
  register: []
  updateField: [field: 'buildId' | 'executablePath' | 'workingDir' | 'sourceDir', value: string]
}>()

const titleId = 'clotho-build-registration-dialog'
</script>

<template>
  <LoomDialogShell :open="open" :title-id="titleId" :dismissible="!loading.register" close-label="Cancel" @close="emit('close')">
    <template #header>
      <p class="dialog-kicker">Clotho / Known Local Build</p>
      <h3 :id="titleId">Register a local Core build</h3>
      <p class="dialog-subtitle">
        Build id is the logical ref that Atropos uses later; the executable path remains an app-local implementation
        detail resolved by Clotho.
      </p>
    </template>

    <div class="dialog-stack">
      <p v-if="issue" class="inline-issue">{{ issue }}</p>

      <label class="field">
        <span class="field-label">Build ID</span>
        <input
          :value="props.draft.buildId"
          class="field-input mono"
          type="text"
          placeholder="dev-core"
          @input="emit('updateField', 'buildId', ($event.target as HTMLInputElement).value)"
        />
      </label>

      <label class="field">
        <span class="field-label">Executable Path</span>
        <input
          :value="props.draft.executablePath"
          class="field-input mono"
          type="text"
          placeholder="/absolute/path/to/beluna-core"
          @input="emit('updateField', 'executablePath', ($event.target as HTMLInputElement).value)"
        />
      </label>

      <div class="field-row">
        <label class="field">
          <span class="field-label">Working Dir</span>
          <input
            :value="props.draft.workingDir"
            class="field-input mono"
            type="text"
            placeholder="optional"
            @input="emit('updateField', 'workingDir', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <label class="field">
          <span class="field-label">Source Dir</span>
          <input
            :value="props.draft.sourceDir"
            class="field-input mono"
            type="text"
            placeholder="optional"
            @input="emit('updateField', 'sourceDir', ($event.target as HTMLInputElement).value)"
          />
        </label>
      </div>

      <div class="dialog-actions">
        <p class="field-note">
          If working dir is left blank, Clotho will default it from the executable parent directory after registration.
        </p>
        <button class="button-primary" type="button" :disabled="!canRegister" @click="emit('register')">
          {{ loading.register ? 'Registering…' : 'Register Build' }}
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

.field-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
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
  .field-row {
    grid-template-columns: 1fr;
  }

  .dialog-actions {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
