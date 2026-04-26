<template>
  <div class="kcc-viewer-root">
    <MlCadViewer
      v-if="sourceUrl"
      locale="en"
      :url="sourceUrl"
      @create="onCreate"
    />
    <div v-else class="kcc-viewer-empty">
      <div class="kcc-viewer-empty__label">Waiting for drawing…</div>
    </div>

    <KccOverlay
      v-if="overlayVisible"
      :features="features"
      :selected-feature-id="selectedFeatureId"
      @feature-click="onFeatureClick"
    />
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue';
import { MlCadViewer } from '@mlightcad/cad-viewer';
import {
  BRIDGE_VERSION,
  type KccFeatureLite,
} from '@kcc/viewer-bridge-types';

import { bridge } from './bridge/client';
import KccOverlay from './overlay/KccOverlay.vue';

const sourceUrl = ref<string | null>(null);
const features = ref<KccFeatureLite[]>([]);
const selectedFeatureId = ref<string | null>(null);
const overlayVisible = ref<boolean>(true);

const unsubscribe = bridge.on(msg => {
  switch (msg.type) {
    case 'load':
      sourceUrl.value = msg.sourceUrl;
      break;
    case 'setKccFeatures':
      features.value = msg.features;
      break;
    case 'setSelectedFeature':
      selectedFeatureId.value = msg.featureId;
      break;
    case 'setKccOverlayVisible':
      overlayVisible.value = msg.visible;
      break;
    case 'setLayerVisible':
      // TODO: wire into AcApDocManager layer table when viewer is initialized
      break;
    case 'zoom':
      // TODO: bind to AcApZoomCmd once we have a handle to the command manager
      break;
    case 'theme':
      document.documentElement.dataset.theme = msg.mode;
      break;
  }
});

onBeforeUnmount(() => {
  unsubscribe();
});

const onCreate = () => {
  // mlightcad has finished initializing. Nothing else needed here yet —
  // the MlCadViewer component watches `:url` and opens it automatically.
};

const onFeatureClick = (id: string) => {
  bridge.send({ v: BRIDGE_VERSION, type: 'featureClicked', featureId: id });
};
</script>

<style scoped>
.kcc-viewer-root {
  position: fixed;
  inset: 0;
  background: var(--kcc-bg);
  overflow: hidden;
}

.kcc-viewer-empty {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--kcc-text-muted);
  font-family: var(--kcc-font-mono);
  font-size: 13px;
  letter-spacing: 0.05em;
}

.kcc-viewer-empty__label {
  padding: 12px 20px;
  border: 1px solid var(--kcc-border);
  border-radius: var(--kcc-radius);
  background: var(--kcc-bg-panel);
}
</style>
