<script setup lang="ts">
import { computed } from 'vue'
import type { JsonSectionInput } from '@/presentation/loom/shared/json'

const props = defineProps<{
  sections: JsonSectionInput[]
}>()

const visibleSections = computed(() => props.sections.filter((section) => section))

function pretty(value: unknown): string {
  return JSON.stringify(value, null, 2) ?? 'null'
}

function sectionSummary(value: unknown): string {
  if (value == null) {
    return 'null'
  }

  if (Array.isArray(value)) {
    return `${value.length} item${value.length === 1 ? '' : 's'}`
  }

  if (typeof value === 'object') {
    return `${Object.keys(value as Record<string, unknown>).length} field${
      Object.keys(value as Record<string, unknown>).length === 1 ? '' : 's'
    }`
  }

  if (typeof value === 'string') {
    return value.length > 72 ? `${value.slice(0, 72)}…` : value
  }

  return String(value)
}
</script>

<template>
  <div class="json-stack">
    <details
      v-for="section in visibleSections"
      :key="section.key"
      class="json-section"
      :open="section.defaultOpen"
    >
      <summary class="json-summary-row">
        <strong>{{ section.title }}</strong>
        <span class="json-summary-copy">{{ section.summary ?? sectionSummary(section.value) }}</span>
      </summary>

      <pre>{{ pretty(section.value) }}</pre>
    </details>
  </div>
</template>

<style scoped>
.json-stack {
  display: grid;
  gap: 0.5rem;
}

.json-section {
  border: 1px solid var(--line-soft);
  background: var(--panel);
}

.json-summary-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.62rem 0.78rem;
  cursor: pointer;
  list-style: none;
}

.json-summary-row::-webkit-details-marker {
  display: none;
}

.json-summary-row strong {
  font-size: 0.84rem;
}

.json-summary-copy {
  color: var(--text-muted);
  font-size: 0.76rem;
  text-align: right;
}

.json-section pre {
  padding: 0.78rem;
  border-top: 1px solid var(--line-soft);
  background: var(--json-bg);
  color: var(--text-strong);
  font-size: 0.82rem;
  line-height: 1.55;
}
</style>
