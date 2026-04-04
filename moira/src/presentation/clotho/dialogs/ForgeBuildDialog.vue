<script setup lang="ts">
import LoomDialogShell from '@/presentation/loom/chrome/LoomDialogShell.vue'

const props = defineProps<{
  canForge: boolean
  draft: {
    buildId: string
    sourceDir: string
  }
  issue: string | null
  loading: {
    forge: boolean
  }
  open: boolean
}>()

const emit = defineEmits<{
  close: []
  forge: []
  updateField: [field: 'buildId' | 'sourceDir', value: string]
}>()

const titleId = 'clotho-forge-build-dialog'
</script>

<template>
  <LoomDialogShell :open="open" :title-id="titleId" :dismissible="!loading.forge" close-label="Cancel" @close="emit('close')">
    <template #header>
      <p class="dialog-kicker">Clotho / Forge</p>
      <h3 :id="titleId">Forge from local source</h3>
      <p class="dialog-subtitle">
        Accept a Beluna repo root or <code>core/</code> crate root, compile explicitly, and register the result as a
        reusable launch target.
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
        <span class="field-label">Source Dir</span>
        <input
          :value="props.draft.sourceDir"
          class="field-input mono"
          type="text"
          placeholder="/absolute/path/to/Beluna or /absolute/path/to/Beluna/core"
          @input="emit('updateField', 'sourceDir', ($event.target as HTMLInputElement).value)"
        />
      </label>

      <div class="dialog-actions">
        <p class="field-note">Forge is explicit and reusable: Clotho compiles now, then Atropos only consumes the prepared target later.</p>
        <button class="button-primary" type="button" :disabled="!canForge" @click="emit('forge')">
          {{ loading.forge ? 'Forging…' : 'Forge Target' }}
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
