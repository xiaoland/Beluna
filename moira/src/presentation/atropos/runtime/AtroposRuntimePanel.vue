<script setup lang="ts">
import { computed } from 'vue'

import { stateTone } from '@/presentation/format'
import type { RuntimeStatus } from '@/projection/atropos'
import AtroposRuntimeCard from './AtroposRuntimeCard.vue'
import ForceKillDialog from './ForceKillDialog.vue'

const props = defineProps<{
  canForceKill: boolean
  canStop: boolean
  canWake: boolean
  issue: string | null
  loading: {
    forceKill: boolean
    runtime: boolean
    stop: boolean
    wake: boolean
  }
  runtime: RuntimeStatus | null
  selectedBuildId: string | null
  selectedProfileId: string | null
  showForceKillConfirm: boolean
}>()

const emit = defineEmits<{
  cancelForceKill: []
  confirmForceKill: []
  refresh: []
  requestForceKill: []
  stop: []
  wake: []
}>()

const runtimeTone = computed(() => stateTone(props.runtime?.phase ?? null))
</script>

<template>
  <section class="panel-shell runtime-panel">
    <div class="panel-head runtime-head">
      <div>
        <p class="panel-kicker">Atropos</p>
        <h2 class="panel-title">Supervision Station</h2>
        <p class="panel-subtitle">
          Wake Core from the selected Clotho inputs, stop it gracefully, and reserve force-kill behind a second
          confirmation dialog.
        </p>
      </div>

      <div class="head-actions">
        <span class="status-badge" :class="`status-${runtimeTone}`">
          {{ runtime?.phase ?? (loading.runtime ? 'loading' : 'idle') }}
        </span>
        <button class="button-secondary" type="button" :disabled="loading.runtime" @click="emit('refresh')">
          Refresh Runtime
        </button>
      </div>
    </div>

    <p v-if="issue" class="issue-banner">{{ issue }}</p>

    <div class="runtime-card-shell">
      <AtroposRuntimeCard
        :can-force-kill="canForceKill"
        :can-stop="canStop"
        :can-wake="canWake"
        :loading="{ forceKill: loading.forceKill, stop: loading.stop, wake: loading.wake }"
        :runtime="runtime"
        :selected-build-id="selectedBuildId"
        :selected-profile-id="selectedProfileId"
        @request-force-kill="emit('requestForceKill')"
        @stop="emit('stop')"
        @wake="emit('wake')"
      />
    </div>

    <ForceKillDialog
      :loading="loading.forceKill"
      :open="showForceKillConfirm"
      :runtime="runtime"
      @cancel="emit('cancelForceKill')"
      @confirm="emit('confirmForceKill')"
    />
  </section>
</template>

<style scoped>
.runtime-panel {
  position: relative;
  z-index: 1;
  overflow: hidden;
}

.runtime-head {
  padding-bottom: 0.95rem;
}

.head-actions {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.issue-banner {
  margin: 0 1rem 1rem;
  padding: 0.72rem 0.85rem;
  border: 1px solid rgba(162, 77, 68, 0.2);
  background: rgba(162, 77, 68, 0.08);
  color: var(--err);
}

.runtime-card-shell {
  padding: 0 1rem 1rem;
}

@media (max-width: 780px) {
  .head-actions {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
