import { createApp } from "vue";
import App from "./App.vue";
import i18n from "./i18n/index";
import { initializeApiUrl } from "./config";
import { loader } from "@guolao/vue-monaco-editor";

loader.config({
    paths: {
        vs: "https://cdn.jsdelivr.net/npm/monaco-editor@0.55/min/vs",
    },
});

const app = createApp(App).use(i18n);

initializeApiUrl().finally(() => {
    app.mount("#app");
});
