import {
    NAME_START,
    NAME_END,
    SIMPLE_SUFFIXES,
    ALL_WORDS,
    PREFIXES,
    ADV_SUFFIXES,
    NUMBERS,
    SEPARATORS,
    LEET_MAP,
    WORDS,
} from "./wordlists";

function pick<T>(arr: readonly T[]): T {
    return arr[Math.floor(Math.random() * arr.length)];
}

function capitalize(s: string): string {
    return s.charAt(0).toUpperCase() + s.slice(1).toLowerCase();
}

function randomWord(category?: keyof typeof WORDS): string {
    if (category) return pick(WORDS[category]);
    return pick(ALL_WORDS);
}

function genSimple(): string {
    const start = pick(NAME_START);
    const end = pick(NAME_END);
    let name = start + end;

    if (Math.random() < 0.25) {
        name += pick(SIMPLE_SUFFIXES);
    }

    if (Math.random() < 0.15 && name.length >= 4) {
        const mid = 1 + Math.floor(Math.random() * (name.length - 2));
        name =
            name.slice(0, mid) + name[mid].toUpperCase() + name.slice(mid + 1);
    }

    if (Math.random() < 0.3) {
        name += "_";
    }

    if (Math.random() < 0.5) {
        name += Math.floor(Math.random() * 9999);
    }

    return name.slice(0, 16);
}

function genGamer(): string {
    const w1 = capitalize(randomWord());
    const w2 = capitalize(randomWord());
    const sep = pick(SEPARATORS);
    const num = Math.random() > 0.4 ? pick(NUMBERS) : "";
    const sfx = Math.random() > 0.7 ? pick(ADV_SUFFIXES) : "";
    const pfx = Math.random() > 0.6 ? pick(PREFIXES) : "";
    const candidates = [
        `${pfx}${w1}${sep}${w2}${num}${sfx}`,
        `${pfx}${w1}${w2}${num}${sfx}`,
    ];
    return pick(candidates).slice(0, 16);
}

function genCool(): string {
    const cat1: (keyof typeof WORDS)[] = [
        "combat",
        "qualities",
        "elements",
        "space",
        "tech",
    ];
    const cat2: (keyof typeof WORDS)[] = [
        "monsters",
        "animals",
        "actions",
        "abstract",
        "nature",
    ];
    const w1 = capitalize(randomWord(pick(cat1)));
    const w2 = capitalize(randomWord(pick(cat2)));
    const sep = pick(SEPARATORS);
    const num = Math.random() > 0.5 ? Math.floor(Math.random() * 999) : "";
    return `${w1}${sep}${w2}${num}`.slice(0, 16);
}

function genClassic(): string {
    const cat1: (keyof typeof WORDS)[] = [
        "qualities",
        "elements",
        "nature",
        "myth",
    ];
    const cat2: (keyof typeof WORDS)[] = [
        "actions",
        "monsters",
        "animals",
        "abstract",
    ];
    const w1 = capitalize(randomWord(pick(cat1)));
    const w2 = capitalize(randomWord(pick(cat2)));
    return `${w1}${w2}`.slice(0, 16);
}

function genOG(): string {
    const w1 = capitalize(randomWord());
    const w2 = capitalize(randomWord());
    const sep = pick(SEPARATORS);
    const num = pick(NUMBERS);
    return `${pick(PREFIXES)}${w1}${sep}${w2}${num}`.slice(0, 16);
}

function genLeet(): string {
    const w1 = capitalize(randomWord());
    const w2 = capitalize(randomWord());
    const num = pick(NUMBERS);
    const raw = `${w1}${w2}${num}`;
    const leet = raw
        .split("")
        .map((c) =>
            Math.random() > 0.6 && LEET_MAP[c.toLowerCase()]
                ? LEET_MAP[c.toLowerCase()]
                : c
        )
        .join("");
    return leet.slice(0, 16);
}

function genXstyle(): string {
    const w = capitalize(randomWord());
    const num = Math.random() > 0.5 ? pick(NUMBERS) : "";
    return pick([
        `x${w}x${num}`,
        `xx${w}xx${num}`,
        `x${w}${num}`,
        `${w}x${num}`,
    ]).slice(0, 16);
}

function genLetterNum(): string {
    const w = capitalize(randomWord());
    const num = Math.floor(Math.random() * 999);
    return pick([
        `${w}${num}`,
        `${w}_${num}`,
        `${pick(PREFIXES)}${w}${num}`,
    ]).slice(0, 16);
}

export type NameMode = "simple" | "advanced";
export type NameStyle =
    | "random"
    | "gamer"
    | "cool"
    | "classic"
    | "og"
    | "leet"
    | "xstyle"
    | "letters";

const ADVANCED_GENERATORS: Record<
    Exclude<NameStyle, "random">,
    () => string
> = {
    gamer: genGamer,
    cool: genCool,
    classic: genClassic,
    og: genOG,
    leet: genLeet,
    xstyle: genXstyle,
    letters: genLetterNum,
};

const ALL_ADVANCED: (() => string)[] = Object.values(ADVANCED_GENERATORS);

export function randomName(
    mode: NameMode = "simple",
    style?: NameStyle
): string {
    if (mode === "simple") return genSimple();
    if (style && style !== "random") return ADVANCED_GENERATORS[style]();
    return pick(ALL_ADVANCED)();
}
