import { createApp } from "vue";
import App from "./App.vue";
import Vue3Lottie from "vue3-lottie";
import i18n from './i18n/index';
import { initializeAuthUrl } from './config';
import { loader } from '@guolao/vue-monaco-editor';
import { applyCursorForEvent } from "./utils/events";

loader.config({
  paths: {
    vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.52.2/min/vs',
  },
})

initializeAuthUrl()
  .finally(() => {
    applyCursorForEvent();

    createApp(App)
      .use(Vue3Lottie)
      .use(i18n)
      .mount('#app');
  });
