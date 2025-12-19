import { createApp } from "vue";
import App from "./App.vue";
import Vue3Lottie from "vue3-lottie";
import i18n from './i18n/index';
import { initializeAuthUrl } from './config';
import { loader } from '@guolao/vue-monaco-editor';

import * as Sentry from "@sentry/vue";

loader.config({
  paths: {
    vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.55/min/vs',
  },
})

const app = createApp(App)
  .use(Vue3Lottie)
  .use(i18n);

initializeAuthUrl()
  .finally(() => {
    app.mount('#app');
  });

Sentry.init({
  app,
  dsn: "https://2220bc70de22a1841c3792c4bf314731@o4510521933889536.ingest.de.sentry.io/4510521935528016",
  sendDefaultPii: true,
  integrations: [
    Sentry.replayIntegration()
  ],
  replaysSessionSampleRate: 0.1,
  replaysOnErrorSampleRate: 1.0
});