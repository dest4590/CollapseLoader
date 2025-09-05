export const DISCORD_STATE_KEYS: Record<string, string> = {
    home: 'discord.states.browsing_clients',
    custom_clients: 'discord.states.browsing_custom_clients',
    news: 'discord.states.browsing_news',
    settings: 'discord.states.configuring_settings',
    friends: 'discord.states.browsing_friends',
    'blocked-users': 'discord.states.configuring_settings',
    theme: 'discord.states.enjoying_visuals',
    app_logs: 'discord.states.watching_client_behavior',
    'user-profile': 'discord.states.in_profile',
    about: 'discord.states.watching_about',
    login: 'discord.states.logging_in',
};

export const getDiscordState = (key: string, translate: (k: string) => string) => {
    const stateKey = DISCORD_STATE_KEYS[key] || DISCORD_STATE_KEYS.home;
    return translate(stateKey);
};
