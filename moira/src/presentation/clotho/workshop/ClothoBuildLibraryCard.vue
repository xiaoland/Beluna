<script setup lang="ts">
defineProps<{
  issue: string | null
  loading: {
    register: boolean
  }
  selectedBuild: {
    buildId: string
    executablePath: string
    workingDir: string
    sourceDir: string | null
  } | null
}>()

defineEmits<{
  openRegister: []
}>()
</script>

<template>
  <article class="workshop-card">
    <div class="section-head">
      <div>
        <h3>Known Local Build</h3>
        <p class="section-note">
          Keep the next wake anchored to a named local executable without leaking raw path resolution into Atropos.
        </p>
      </div>

      <button class="button-secondary" type="button" :disabled="loading.register" @click="$emit('openRegister')">
        {{ selectedBuild ? 'Replace Build…' : 'Register Build…' }}
      </button>
    </div>

    <p v-if="issue" class="inline-issue">{{ issue }}</p>

    <div v-if="selectedBuild" class="build-grid">
      <div class="build-item">
        <span class="build-label">Build ID</span>
        <strong class="mono">{{ selectedBuild.buildId }}</strong>
      </div>
      <div class="build-item wide">
        <span class="build-label">Executable Path</span>
        <strong class="mono">{{ selectedBuild.executablePath }}</strong>
      </div>
      <div class="build-item wide">
        <span class="build-label">Working Dir</span>
        <strong class="mono">{{ selectedBuild.workingDir }}</strong>
      </div>
      <div class="build-item wide">
        <span class="build-label">Source Dir</span>
        <strong class="mono">{{ selectedBuild.sourceDir ?? 'Optional / not set' }}</strong>
      </div>
    </div>

    <div v-else class="empty-state build-empty">
      No build is selected yet. Register one local Core executable, then Atropos can wake it from the supervision tab.
    </div>

    <p class="field-note build-note">
      The durable manifest already lives in app-local storage, but this operator selection is still session-local until a
      later persistence slice lands.
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

.inline-issue {
  margin: 0 0 0.8rem;
  padding: 0.72rem 0.85rem;
  border: 1px solid rgba(162, 77, 68, 0.2);
  background: rgba(162, 77, 68, 0.08);
  color: var(--err);
}

.build-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
}

.build-item {
  min-width: 0;
  padding: 0.72rem;
  border: 1px solid var(--line-soft);
  background: rgba(255, 255, 255, 0.82);
}

.build-item.wide {
  grid-column: span 2;
}

.build-label {
  color: var(--text-muted);
  font-size: 0.76rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.build-item strong {
  display: block;
  margin-top: 0.35rem;
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
  .section-head {
    align-items: flex-start;
    flex-direction: column;
  }

  .build-grid {
    grid-template-columns: 1fr;
  }

  .build-item.wide {
    grid-column: span 1;
  }
}
</style>
