import { createApp } from "vue";
import App from "./App.vue";
import NetworkWindow from "./windows/NetworkWindow.vue";
import CustomizationWindow from "./windows/CustomizationWindow.vue";
import i18n from "@services/i18n";
import { loader } from "@guolao/vue-monaco-editor";

loader.config({
    paths: {
        vs: "https://cdn.jsdelivr.net/npm/monaco-editor@0.55/min/vs",
    },
});

const isNetworkWindow = window.location.search.includes("window=network");
const isCustomizationWindow = window.location.search.includes(
    "window=customization"
);

let rootComponent = App;
if (isNetworkWindow) rootComponent = NetworkWindow;
else if (isCustomizationWindow) rootComponent = CustomizationWindow;

const app = createApp(rootComponent).use(i18n);

// initializeApiUrl().finally(() => {

// });

app.mount("#app");
