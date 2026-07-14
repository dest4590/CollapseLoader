export const NAME_START = [
    "Kai","Lex","Val","Max","Zor","Dex","Tor","Ren","Leo","Luc",
    "Rav","Ash","Ard","Nik","Sam","Ben","Kel","Jax","Nol","Vin",
    "Tyr","Sil","Cai","Dar","Fen","Gav","Mal","Neo","Kaz","Rix",
    "Vex","Zed","Ax","Jex","Nyx","Pax","Rex","Syl","Tyb","Wes",
    "Yor","Zan","Bal","Cor","Ez","Fiz","Gor","Hal","Ion","Jar",
    "Kit","Lys","Mav","Oz","Pry","Qu","Raz","Sol","Thr","Ulf",
    "Vor","Wul","Xar","Yaz","Zeph",
];

export const NAME_END = [
    "on","ix","us","en","or","el","ar","an","eo","is",
    "eth","il","ad","os","eus","es","ym","ax","ik","er",
    "as","ai","ur","al","ium","on","ex","az","ith","yn",
    "ora","ius","eon","ane","oth","ene","ica","uno","ira","ona",
];

export const SIMPLE_SUFFIXES = [
    "x","q","zz","ai","xy","zor","ar","yn","ox","ex",
    "zx","qt","ez","ax","yx","","","","","",
];

export const WORDS = {
    combat: [
        "Strike","Snipe","Blade","Claw","Fang","Shot","Rush","Punch","Slash","Thrust",
        "Smash","Crush","Cleave","Rend","Shred","Gore","Spike","Lance","Axe","Dagger",
        "Arrow","Bolt","Shell","Blast","Burst","Surge","Tide","Wave","Pulse","Beat",
    ],
    monsters: [
        "Slayer","Hunter","Raider","Reaper","Wraith","Specter","Phantom","Demon",
        "Dragon","Hydra","Golem","Titan","Kraken","Basilisk","Chimera","Minotaur",
        "Vampire","Werewolf","Banshee","Ghoul","Zombie","Skeleton","Creeper",
        "Enderman","Blaze","Wither","Stray","Husk","Drowned","Pillager",
    ],
    animals: [
        "Wolf","Bear","Hawk","Fox","Cat","Snake","Spider","Raven","Eagle","Lynx",
        "Shark","Tiger","Lion","Panther","Cobra","Falcon","Stallion","Prowl",
        "Scorpion","Viper","Beast","Prowler","Stalker","Charger","Cobra",
    ],
    nature: [
        "Storm","Thunder","Frost","Blaze","Ember","Ash","Dust","Clay","Sand",
        "Snow","Rain","Wind","Gale","Mist","Fog","Cloud","Hail",
        "Root","Vine","Thorn","Bramble","Fern","Oak","Pine","Birch","Cedar",
    ],
    elements: [
        "Iron","Steel","Gold","Silver","Bronze","Copper","Platinum","Obsidian",
        "Onyx","Quartz","Cobalt","Ivory","Amber","Jade","Ruby","Sapphire",
        "Diamond","Crystal","Basalt","Granite","Marble","Flint","Slate",
    ],
    space: [
        "Nova","Nebula","Star","Comet","Asteroid","Pulsar","Quasar","Cosmos",
        "Void","Apex","Zenith","Orbit","Eclipse","Sol","Luna","Mars","Venus",
        "Jupiter","Saturn","Pluto","Neptune","Meteor","Galaxy","Astral",
    ],
    tech: [
        "Pixel","Cyber","Hex","Flux","Bolt","Surge","Byte","Core","Node","Link",
        "Code","Data","Sync","Grid","Mesh","Wire","Bit","Chip","Mod","Hack",
        "Glitch","Proxy","Root","Shell","Kernel","Stack","Heap","Cache","Vector",
    ],
    myth: [
        "Arcane","Mystic","Runic","Elder","Ancient","Forgotten","Lost","Hidden",
        "Sacred","Cursed","Blessed","Hallowed","Doomed","Fated","Bound","Sealed",
        "Warded","Sigil","Glyph","Rune","Tome","Grimoire","Scroll","Codex",
    ],
    qualities: [
        "Dark","Shadow","Grim","Rogue","Swift","Silent","Wicked","Fallen",
        "Lone","Cold","Wild","Fierce","Savage","Lethal","Toxic","Vile",
        "Brave","Bold","Grand","Royal","Noble","Prime","Ultra","Mega","Hyper",
    ],
    actions: [
        "Walker","Ranger","Scout","Warden","Guardian","Sentinel","Keeper",
        "Breaker","Maker","Shaper","Weaver","Binder","Caster","Singer",
        "Seeker","Watcher","Prowler","Stalker","Crawler","Runner","Flyer",
    ],
    abstract: [
        "Honor","Valor","Glory","Fate","Doom","Wrath","Rage","Fury","Pride",
        "Hope","Fear","Pain","Grief","Joy","Bliss","Peace","Chaos","Order",
        "Faith","Grace","Mercy","Vengeance","Justice","Freedom","Power","Legacy","Destiny",
    ],
} as const;

export const PREFIXES = [
    "x","xx","i","ii","The","Real","Not","Best","Pro","Raw",
    "Big","Old","New","Neo","Top","Max","Mr","Dr","Lord",
    "","","","","","","","",
];

export const ADV_SUFFIXES = [
    "x","xd","lol","gg","ez","YT","TV","TTV",
    "","","","","","","","",
];

export const NUMBERS = [
    "","0","1","3","4","7","9","13","42","69",
    "99","1337","007","420","666","777","888","999","00","11",
    "22","33","44","55","66","77","88","100","200","360",
];

export const SEPARATORS = ["","_","","","",""];

export const LEET_MAP: Record<string, string> = {
    a: "4", e: "3", i: "1", o: "0", s: "5", t: "7", b: "8", g: "9",
};

export const ALL_WORDS: readonly string[] = Object.values(WORDS).flat();
