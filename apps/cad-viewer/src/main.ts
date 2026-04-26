import 'element-plus/dist/index.css';
import './theme/tokens.css';
import './theme/element-plus-override.scss';

import { i18n } from '@mlightcad/cad-viewer';
import ElementPlus from 'element-plus';
import { createApp } from 'vue';

import App from './App.vue';
import { bridge } from './bridge/client';
import { BRIDGE_VERSION } from '@kcc/viewer-bridge-types';

const app = createApp(App);
app.use(i18n);
app.use(ElementPlus);
app.mount('#app');

// Announce we are alive as soon as the Vue app has mounted. The parent uses
// this to know the iframe is ready to receive a `load` message.
bridge.send({ v: BRIDGE_VERSION, type: 'ready' });
