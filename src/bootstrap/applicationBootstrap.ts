import { loader } from "@guolao/vue-monaco-editor";
import { createApp, type Component } from "vue";
import App from "@/App.vue";
import i18n from "@services/i18n";
import CustomizationWindow from "@/windows/CustomizationWindow.vue";
import NetworkWindow from "@/windows/NetworkWindow.vue";

const MONACO_VS_PATH = "https://cdn.jsdelivr.net/npm/monaco-editor@0.55/min/vs";
const WINDOW_PARAM = "window";

enum AppWindowKind {
    Main = "main",
    Network = "network",
    Customization = "customization",
}

class MonacoBootstrapper {
    configure() {
        loader.config({
            paths: {
                vs: MONACO_VS_PATH,
            },
        });
    }
}

class WindowComponentResolver {
    constructor(private readonly location: Location) {}

    resolve(): Component {
        switch (this.getWindowKind()) {
            case AppWindowKind.Network:
                return NetworkWindow;
            case AppWindowKind.Customization:
                return CustomizationWindow;
            default:
                return App;
        }
    }

    private getWindowKind(): AppWindowKind {
        const windowKind = new URLSearchParams(this.location.search).get(
            WINDOW_PARAM
        );

        if (windowKind === AppWindowKind.Network) {
            return AppWindowKind.Network;
        }

        if (windowKind === AppWindowKind.Customization) {
            return AppWindowKind.Customization;
        }

        return AppWindowKind.Main;
    }
}

export class ApplicationBootstrap {
    constructor(private readonly location: Location) {}

    mount(selector: string) {
        new MonacoBootstrapper().configure();

        createApp(new WindowComponentResolver(this.location).resolve())
            .use(i18n)
            .mount(selector);
    }
}
