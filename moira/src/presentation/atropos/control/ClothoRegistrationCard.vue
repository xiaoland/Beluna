<script setup lang="ts">
defineProps<{
  canRegister: boolean
  draft: {
    buildId: string
    executablePath: string
    workingDir: string
    sourceDir: string
  }
  loading: {
    register: boolean
  }
  selectedBuildId: string | null
}>()

defineEmits<{
  register: []
  updateField: [field: 'buildId' | 'executablePath' | 'workingDir' | 'sourceDir', value: string]
}>()
</script>

<template>
  <article class="control-card">
    <div class="section-head">
      <h3>Clotho Registration</h3>
      <span v-if="selectedBuildId" class="selection-pill mono">{{ selectedBuildId }}</span>
    </div>

    <label class="field">
      <span class="field-label">Build ID</span>
      <input
        :value="draft.buildId"
        class="field-input mono"
        type="text"
        placeholder="dev-core"
        @input="$emit('updateField', 'buildId', ($event.target as HTMLInputElement).value)"
      />
    </label>

    <label class="field">
      <span class="field-label">Executable Path</span>
      <input
        :value="draft.executablePath"
        class="field-input mono"
        type="text"
        placeholder="/absolute/path/to/beluna"
        @input="$emit('updateField', 'executablePath', ($event.target as HTMLInputElement).value)"
      />
    </label>

    <div class="field-row">
      <label class="field">
        <span class="field-label">Working Dir</span>
        <input
          :value="draft.workingDir"
          class="field-input mono"
          type="text"
          placeholder="optional"
          @input="$emit('updateField', 'workingDir', ($event.target as HTMLInputElement).value)"
        />
      </label>

      <label class="field">
        <span class="field-label">Source Dir</span>
        <input
          :value="draft.sourceDir"
          class="field-input mono"
          type="text"
          placeholder="optional"
          @input="$emit('updateField', 'sourceDir', ($event.target as HTMLInputElement).value)"
        />
      </label>
    </div>

    <div class="action-row">
      <button class="button-primary" type="button" :disabled="!canRegister" @click="$emit('register')">
        {{ loading.register ? 'Registering…' : 'Register Build' }}
      </button>
      <p class="field-note">
        Wake uses the last successfully registered build ref. Profile selection now lives in the dedicated Clotho
        profile card.
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
  align-items: baseline;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.95rem;
}

.section-head h3 {
  margin: 0;
  font-family: var(--font-display);
  font-size: 1.08rem;
}

.selection-pill {
  color: var(--text-muted);
  font-size: 0.82rem;
}

.field-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
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

.field-input::placeholder {
  color: color-mix(in srgb, var(--text-muted) 78%, white);
}

.action-row {
  display: flex;
  align-items: center;
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
  .action-row,
  .section-head {
    align-items: flex-start;
    flex-direction: column;
  }

  .field-row {
    grid-template-columns: 1fr;
  }
}
</style>
