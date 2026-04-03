import { ref } from 'vue'

export type LoomFeatureTab = 'lachesis' | 'atropos' | 'clotho'

export function useLoomNavigation() {
  const activeFeature = ref<LoomFeatureTab>('lachesis')

  function selectFeature(feature: LoomFeatureTab): void {
    activeFeature.value = feature
  }

  return {
    activeFeature,
    selectFeature,
  }
}
