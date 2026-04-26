<template>
  <svg class="kcc-overlay" :viewBox="viewBox" preserveAspectRatio="xMidYMid meet">
    <g v-for="feature in features" :key="feature.id" @click="onClick(feature.id)">
      <circle
        :cx="feature.cx"
        :cy="-feature.cy"
        :r="feature.id === selectedFeatureId ? 14 : 10"
        :fill="colorFor(feature.classification) + '30'"
        :stroke="colorFor(feature.classification)"
        :stroke-width="feature.id === selectedFeatureId ? 3 : 2"
      />
      <circle
        :cx="feature.cx"
        :cy="-feature.cy"
        :r="3"
        :fill="colorFor(feature.classification)"
      />
    </g>
  </svg>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { KccClassification, KccFeatureLite } from '@kcc/viewer-bridge-types';

const props = defineProps<{
  features: KccFeatureLite[];
  selectedFeatureId: string | null;
}>();

const emit = defineEmits<{
  (e: 'feature-click', id: string): void;
}>();

// Placeholder viewBox until we can hook into mlightcad's camera.
// Phase 3b will replace this with screen-space projection using the cad-viewer
// scene's camera matrix; until then the overlay assumes world-space 1:1 and
// covers a nominal 10000x10000 unit area. This is intentionally wrong enough
// to be obvious during spike so we remember to fix it.
const viewBox = computed(() => '-5000 -5000 10000 10000');

const colors: Record<KccClassification, string> = {
  kcc: 'var(--kcc-kcc)',
  important: 'var(--kcc-important)',
  standard: 'var(--kcc-standard)',
};

const colorFor = (c: KccClassification) => colors[c];

const onClick = (id: string) => emit('feature-click', id);
// Avoid unused-warn while the file is under construction.
void props;
</script>

<style scoped>
.kcc-overlay {
  position: absolute;
  inset: 0;
  pointer-events: none;
  width: 100%;
  height: 100%;
}
.kcc-overlay g {
  pointer-events: auto;
  cursor: pointer;
}
</style>
