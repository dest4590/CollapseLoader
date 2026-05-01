export type AnimationCleanup = () => void;
export type AnimationInit = (
    canvas: HTMLCanvasElement | null,
    theme?: string
) => AnimationCleanup;

export interface PreloaderAnimation {
    svgString: string;
    initMatrix: AnimationInit;
}

const matrixSvg = `
<svg xmlns="http://www.w3.org/2000/svg" width="435" height="365" fill="none" viewBox="0 0 435 365">
    <path fill="#000" d="M182.028 36.5L272.733 127.44L309.138 127.44L182.028 2.22629e-05L-7.97733e-06 182.5L182.028 365L309.138 237.56L272.733 237.56L182.028 328.5L36.4056 182.5L182.028 36.5Z" />
    <path fill="#000" d="M182.028 72.81L236.731 127.655L182.028 182.5L236.731 237.345L182.028 292.19L72.6217 182.5L182.028 72.81Z" />
    <path fill="#000" d="M254.65 36.5L345.354 127.44L381.76 127.44L254.65 1.90885e-05L236.447 18.25L236.446 18.2508L254.649 36.5008L254.65 36.5Z" />
    <path fill="#000" d="M381.76 237.56L345.44 237.56L236.49 346.793L254.65 365L381.76 237.56Z" />
    <path fill="#000" d="M249.602 148.733L216.137 182.285L249.602 215.837L249.653 215.786L249.653 215.888L401.535 215.888L401.535 215.837L435 182.285L401.535 148.733L401.484 148.783L249.653 148.783L249.602 148.733Z" />
</svg>
`;

const CANVAS_WIDTH = 217.5;
const CANVAS_HEIGHT = 182.5;
const FONT_SIZE = 7;

const setupCanvas = (
    canvas: HTMLCanvasElement
): CanvasRenderingContext2D | null => {
    const ctx = canvas.getContext("2d");
    if (!ctx) return null;
    canvas.width = CANVAS_WIDTH * 2;
    canvas.height = CANVAS_HEIGHT * 2;
    ctx.scale(2, 2);
    ctx.font = `${FONT_SIZE}px monospace`;
    ctx.textBaseline = "top";
    return ctx;
};

export const animations: Record<string, PreloaderAnimation> = {
    "matrix-horizontal": {
        svgString: matrixSvg,
        initMatrix: (canvas, theme = "dark") => {
            if (!canvas) return () => {};
            const ctx = setupCanvas(canvas);
            if (!ctx) return () => {};

            const rows = Math.floor(CANVAS_HEIGHT / FONT_SIZE);
            type Stream = {
                x: number;
                y: number;
                speed: number;
                burst: number;
            };
            const streams: Stream[] = [];

            for (let i = 0; i < rows; i++) {
                const streamCount = Math.random() > 0.7 ? 2 : 1;
                for (let s = 0; s < streamCount; s++) {
                    streams.push({
                        x: Math.random() * -200,
                        y: i * FONT_SIZE,
                        speed: 2 + Math.random() * 4.5,
                        burst: 0,
                    });
                }
            }

            const chars =
                "01$#@%&*<>[]{}\\/|=+-_~^:;ABCDEFGHIJKLMNOPQRSTUVWXYZ";

            const interval = window.setInterval(() => {
                ctx.fillStyle =
                    theme === "light"
                        ? "rgba(0,0,0,0.06)"
                        : "rgba(255,255,255,0.06)";

                for (const stream of streams) {
                    const char =
                        chars[Math.floor(Math.random() * chars.length)];
                    ctx.fillStyle =
                        theme === "light"
                            ? `rgba(255,255,255,${0.7 + Math.random() * 0.3})`
                            : `rgba(0,0,0,${0.6 + Math.random() * 0.3})`;

                    ctx.fillText(char, stream.x, stream.y);

                    if (Math.random() > 0.96)
                        stream.burst = 10 + Math.random() * 20;

                    const speedBoost = stream.burst > 0 ? 3 : 1;
                    stream.x += stream.speed * speedBoost;
                    if (stream.burst > 0) stream.burst--;

                    if (stream.x > CANVAS_WIDTH + 50) {
                        stream.x = -50 - Math.random() * 200;
                    }
                }
            }, 16);

            return () => clearInterval(interval);
        },
    },

    "matrix-vertical": {
        svgString: matrixSvg,
        initMatrix: (canvas, theme = "dark") => {
            if (!canvas) return () => {};
            const ctx = setupCanvas(canvas);
            if (!ctx) return () => {};

            const columns = Math.floor(CANVAS_WIDTH / FONT_SIZE);
            const drops: number[] = Array.from(
                { length: columns },
                () => Math.random() * -50
            );
            const chars = "01337XYZ_<>[]!@#";

            const interval = window.setInterval(() => {
                ctx.fillStyle =
                    theme === "light"
                        ? "rgba(255,255,255,0.9)"
                        : "rgba(0,0,0,0.9)";

                for (let i = 0; i < drops.length; i++) {
                    const text =
                        chars[Math.floor(Math.random() * chars.length)];
                    const x = i * FONT_SIZE;
                    const y = drops[i] * FONT_SIZE;

                    ctx.fillText(text, x, y);

                    if (y > CANVAS_HEIGHT && Math.random() > 0.1) {
                        drops[i] = 0;
                    }

                    drops[i] += 1.8;
                }
            }, 16);

            return () => clearInterval(interval);
        },
    },
};

export const animationKeys = Object.keys(animations);
