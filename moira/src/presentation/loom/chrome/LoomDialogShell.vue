<script setup lang="ts">
import { onBeforeUnmount, onMounted, watch } from "vue";

const props = withDefaults(
  defineProps<{
    closeLabel?: string;
    dismissible?: boolean;
    maxWidth?: string;
    open: boolean;
    titleId: string;
  }>(),
  {
    closeLabel: "Close",
    dismissible: true,
    maxWidth: "42rem",
  },
);

const emit = defineEmits<{
  close: [];
}>();

watch(
  () => props.open,
  (open) => {
    if (typeof document === "undefined") {
      return;
    }

    document.body.style.overflow = open ? "hidden" : "";
  },
);

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});

onBeforeUnmount(() => {
  if (typeof document !== "undefined") {
    document.body.style.overflow = "";
  }

  window.removeEventListener("keydown", handleKeydown);
});

function handleKeydown(event: KeyboardEvent): void {
  if (event.key === "Escape" && props.open && props.dismissible) {
    emit("close");
  }
}

function requestClose(): void {
  if (!props.dismissible) {
    return;
  }

  emit("close");
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="dialog-overlay" @click.self="requestClose">
      <section
        class="dialog-shell"
        :style="{ width: `min(${maxWidth}, 100%)` }"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="titleId"
      >
        <header class="dialog-head">
          <div class="dialog-copy">
            <slot name="header" />
          </div>

          <button
            type="button"
            class="button-secondary dialog-close"
            :disabled="!dismissible"
            @click="requestClose"
          >
            {{ closeLabel }}
          </button>
        </header>

        <div class="dialog-body">
          <slot />
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

.dialog-shell {
  max-height: min(88vh, 46rem);
  overflow-y: auto;
  border: 1px solid var(--line-strong);
  background: var(--panel-strong);
  box-shadow: var(--shadow-soft);
}

.dialog-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem 1rem 0.9rem;
  border-bottom: 1px solid var(--line-soft);
}

.dialog-copy :deep(h3) {
  margin: 0;
  font-family: var(--font-display);
  font-size: 1.3rem;
}

.dialog-copy :deep(.dialog-kicker),
.dialog-copy :deep(.dialog-subtitle) {
  margin: 0.24rem 0 0;
  color: var(--text-muted);
  font-size: 0.84rem;
  line-height: 1.5;
}

.dialog-close {
  min-width: 5.4rem;
}

.dialog-body {
  overflow: auto;
  padding: 1rem;
}

@media (max-width: 780px) {
  .dialog-head {
    flex-direction: column;
  }
}
</style>
