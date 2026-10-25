import { convertFileSrc, invoke } from "@tauri-apps/api/core";

const inflight = new Map<string, Promise<string>>();

const remotePrefixes = ["http://", "https://"];
const inlinePrefixes = ["data:", "blob:"];

async function fetchRemoteImage(url: string): Promise<string> {
    const pending = inflight.get(url);
    if (pending) return pending;

    const request = invoke<string>("fetch_cached_image", { url })
        .finally(() => {
            inflight.delete(url);
        })
        .catch((e) => {
            inflight.delete(url);
            throw e;
        });

    inflight.set(url, request);
    return request;
}

export function resolveImageSource(source: string): Promise<string> {
    if (inlinePrefixes.some((prefix) => source.startsWith(prefix))) {
        return Promise.resolve(source);
    }

    if (remotePrefixes.some((prefix) => source.startsWith(prefix))) {
        return fetchRemoteImage(source);
    }

    return Promise.resolve(convertFileSrc(source));
}
