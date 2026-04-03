<script setup lang="ts">
import { computed } from 'vue'

import LoomDialogShell from '@/presentation/loom/chrome/LoomDialogShell.vue'
import type { RuntimeStatus } from '@/projection/atropos'

const props = defineProps<{
  loading: boolean
  open: boolean
  runtime: RuntimeStatus | null
}>()

const emit = defineEmits<{
  cancel: []
  confirm: []
}>()

const dialogTitleId = computed(() =>
  props.runtime?.pid != null ? `force-kill-dialog-${props.runtime.pid}` : 'force-kill-dialog',
)
</script>

<template>
  <LoomDialogShell
    :open="open"
    :title-id="dialogTitleId"
    :dismissible="!loading"
    max-width="36rem"
    close-label="Cancel"
    @close="emit('cancel')"
  >
    <template #header>
      <p class="dialog-kicker">Atropos / Force Kill</p>
      <h3 :id="dialogTitleId">Force-kill the supervised Core?</h3>
      <p class="dialog-subtitle">
        This bypasses graceful shutdown and should only be used when the supervised process is stuck.
      </p>
    </template>

    <div class="confirm-body">
      <section class="dialog-card">
        <h4>Current Target</h4>
        <dl class="confirm-grid">
          <div>
            <dt>Build</dt>
            <dd class="mono">{{ runtime?.buildId ?? '—' }}</dd>
          </div>
          <div>
            <dt>PID</dt>
            <dd class="mono">{{ runtime?.pid ?? '—' }}</dd>
          </div>
          <div>
            <dt>Phase</dt>
            <dd>{{ runtime?.phase ?? '—' }}</dd>
          </div>
          <div>
            <dt>Profile Path</dt>
            <dd class="mono">{{ runtime?.profilePath ?? '—' }}</dd>
          </div>
        </dl>
      </section>

      <section class="dialog-card">
        <h4>Second Confirmation</h4>
        <p class="confirm-warning">
          Force-kill is intentionally separated from graceful stop so the default operator path remains recoverable.
        </p>

        <div class="confirm-actions">
          <button type="button" class="button-secondary" :disabled="loading" @click="emit('cancel')">
            Keep Waiting
          </button>
          <button type="button" class="button-danger" :disabled="loading" @click="emit('confirm')">
            {{ loading ? 'Force-Killing…' : 'Confirm Force Kill' }}
          </button>
        </div>
      </section>
    </div>
  </LoomDialogShell>
</template>

<style scoped>
.confirm-body {
  display: grid;
  gap: 0.8rem;
}

.dialog-card {
  padding: 0.95rem;
  border: 1px solid var(--line-soft);
  background: rgba(255, 255, 255, 0.88);
}

.dialog-card h4 {
  margin: 0;
}

.confirm-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.8rem;
  margin: 0.8rem 0 0;
}

.confirm-grid dt {
  color: var(--text-muted);
  font-size: 0.76rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.confirm-grid dd {
  margin: 0.3rem 0 0;
  line-height: 1.45;
}

.confirm-warning {
  margin: 0.7rem 0 0;
  color: var(--text-muted);
  line-height: 1.55;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  margin-top: 1rem;
  flex-wrap: wrap;
}

@media (max-width: 780px) {
  .confirm-grid {
    grid-template-columns: 1fr;
  }
}
</style>
