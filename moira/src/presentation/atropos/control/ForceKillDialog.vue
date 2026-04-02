<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, watch } from 'vue'

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

watch(
  () => props.open,
  (open) => {
    if (typeof document === 'undefined') {
      return
    }

    document.body.style.overflow = open ? 'hidden' : ''
  },
)

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  if (typeof document !== 'undefined') {
    document.body.style.overflow = ''
  }

  window.removeEventListener('keydown', handleKeydown)
})

function handleKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && props.open && !props.loading) {
    emit('cancel')
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="dialog-overlay" @click.self="!loading && emit('cancel')">
      <section
        class="dialog-shell confirm-dialog"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="dialogTitleId"
      >
        <header class="dialog-head">
          <div>
            <p class="dialog-kicker">Atropos / Force Kill</p>
            <h3 :id="dialogTitleId">Force-kill the supervised Core?</h3>
            <p class="dialog-subtitle">
              This bypasses graceful shutdown and should only be used when the supervised process is stuck.
            </p>
          </div>

          <button type="button" class="button-secondary dialog-close" :disabled="loading" @click="emit('cancel')">
            Cancel
          </button>
        </header>

        <div class="dialog-body confirm-body">
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
      </section>
    </div>
  </Teleport>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 40;
  display: grid;
  place-items: center;
  padding: 1.2rem;
  background: var(--overlay);
}

.confirm-dialog {
  width: min(36rem, 100%);
  height: auto;
  max-height: min(88vh, 36rem);
  overflow: hidden;
  border: 1px solid var(--line-strong);
  background: var(--panel-strong);
}

.dialog-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem 1rem 0.85rem;
  border-bottom: 1px solid var(--line-soft);
}

.dialog-kicker,
.dialog-subtitle {
  margin: 0.24rem 0 0;
  color: var(--text-muted);
  font-size: 0.84rem;
}

.dialog-head h3,
.dialog-card h4 {
  margin: 0;
}

.dialog-close {
  min-width: 5.4rem;
}

.confirm-body {
  display: grid;
  gap: 0.8rem;
  padding: 1rem;
}

.dialog-card {
  padding: 0.95rem;
  border: 1px solid var(--line-soft);
  background: rgba(255, 255, 255, 0.88);
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
  .dialog-head {
    flex-direction: column;
  }

  .confirm-grid {
    grid-template-columns: 1fr;
  }
}
</style>
