<script setup lang="ts">
import { computed } from 'vue'

import type { RuntimeStatus } from '@/projection/atropos'

const props = defineProps<{
  canForceKill: boolean
  canStop: boolean
  canWake: boolean
  loading: {
    forceKill: boolean
    stop: boolean
    wake: boolean
  }
  runtime: RuntimeStatus | null
  selectedBuildId: string | null
}>()

defineEmits<{
  requestForceKill: []
  stop: []
  wake: []
}>()

const runtimeSummary = computed(() => {
  if (!props.runtime) {
    return 'Runtime not queried yet.'
  }

  const parts = [`phase=${props.runtime.phase}`]
  if (props.runtime.pid != null) {
    parts.push(`pid=${props.runtime.pid}`)
  }
  if (props.runtime.buildId) {
    parts.push(`build=${props.runtime.buildId}`)
  }

  return parts.join(' · ')
})
</script>

<template>
  <article class="control-card runtime-card">
    <div class="section-head">
      <h3>Atropos Runtime</h3>
      <span class="runtime-summary">{{ runtimeSummary }}</span>
    </div>

    <div class="runtime-grid">
      <div class="runtime-item">
        <span class="runtime-label">Build</span>
        <strong class="mono">{{ runtime?.buildId ?? selectedBuildId ?? '—' }}</strong>
      </div>
      <div class="runtime-item">
        <span class="runtime-label">PID</span>
        <strong class="mono">{{ runtime?.pid ?? '—' }}</strong>
      </div>
      <div class="runtime-item wide">
        <span class="runtime-label">Executable</span>
        <strong class="mono">{{ runtime?.executablePath ?? '—' }}</strong>
      </div>
      <div class="runtime-item wide">
        <span class="runtime-label">Working Dir</span>
        <strong class="mono">{{ runtime?.workingDir ?? '—' }}</strong>
      </div>
      <div class="runtime-item wide">
        <span class="runtime-label">Profile Path</span>
        <strong class="mono">{{ runtime?.profilePath ?? '—' }}</strong>
      </div>
      <div class="runtime-item wide">
        <span class="runtime-label">Terminal Reason</span>
        <strong>{{ runtime?.terminalReason ?? '—' }}</strong>
      </div>
    </div>

    <div class="action-row runtime-actions">
      <button class="button-primary" type="button" :disabled="!canWake" @click="$emit('wake')">
        {{ loading.wake ? 'Waking…' : 'Wake Registered Build' }}
      </button>
      <button class="button-secondary" type="button" :disabled="!canStop" @click="$emit('stop')">
        {{ loading.stop ? 'Stopping…' : 'Graceful Stop' }}
      </button>
      <button class="button-danger" type="button" :disabled="!canForceKill" @click="$emit('requestForceKill')">
        {{ loading.forceKill ? 'Force-Killing…' : 'Force Kill…' }}
      </button>
    </div>

    <p v-if="runtime?.terminalReason" class="field-note">Terminal reason is retained until the next successful wake.</p>
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

.runtime-summary {
  color: var(--text-muted);
  font-size: 0.82rem;
}

.runtime-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
  margin-bottom: 1rem;
}

.runtime-item {
  min-width: 0;
  padding: 0.72rem;
  border: 1px solid var(--line-soft);
  background: rgba(255, 255, 255, 0.82);
}

.runtime-item.wide {
  grid-column: span 2;
}

.runtime-label {
  color: var(--text-muted);
  font-size: 0.76rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.runtime-item strong {
  display: block;
  margin-top: 0.35rem;
  line-height: 1.45;
}

.action-row {
  display: flex;
  align-items: center;
  gap: 0.85rem;
  flex-wrap: wrap;
}

.runtime-actions {
  margin-bottom: 0.7rem;
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

  .runtime-grid {
    grid-template-columns: 1fr;
  }

  .runtime-item.wide {
    grid-column: span 1;
  }
}
</style>
