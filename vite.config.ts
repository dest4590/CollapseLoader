import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";

const host: string | undefined = process.env.TAURI_DEV_HOST;

interface HmrConfig {
    protocol: string;
    host: string | undefined;
    port: number;
}

interface ServerConfig {
    port: number;
    strictPort: boolean;
    host: string | boolean;
    hmr: HmrConfig | undefined;
    watch: {
        ignored: string[];
    };
}
export default defineConfig(async () => ({
    plugins: [vue()],

    optimizeDeps: {
        exclude: ["daisyui"],
    },

    resolve: {
        alias: {
            "@": path.resolve(__dirname, "./src"),
        },
    },

    build: {
        rollupOptions: {
            onwarn(warning: any, warn: (warning: any) => void): void {
                if (warning.code === "EMPTY_BUNDLE") {
                    return;
                }
                warn(warning);
            },
            output: {
                manualChunks(id: string): string | undefined {
                    if (id.includes("node_modules")) {
                        if (id.includes("monaco-editor")) {
                            return "vendor-monaco";
                        }
                        if (id.includes("vue-i18n") || id.includes("@vue")) {
                            return "vendor-vue";
                        }
                        if (id.includes("@tauri-apps")) {
                            return "vendor-tauri";
                        }
                        if (id.includes("lucide-vue-next")) {
                            return "vendor-icons";
                        }
                        if (id.includes("gsap") || id.includes("axios")) {
                            return "vendor-utils";
                        }
                    }
                },
            },
        },
        chunkSizeWarningLimit: 1200,
    },

    clearScreen: false,
    server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host
            ? {
                  protocol: "ws",
                  host,
                  port: 1421,
              }
            : undefined,
        watch: {
            ignored: ["**/src-tauri/**"],
        },
    } as ServerConfig,
}));
