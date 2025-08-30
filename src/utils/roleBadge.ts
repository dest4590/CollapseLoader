export function getRoleBadge(role: string | null | undefined, t: (key: string) => string) {
    console.log(role);
    
    if (!role) return null;

    const mapping: Record<string, { class: string; key: string }> = {
        user: { class: 'badge badge-outline', key: 'roles.user' },
        admin: { class: 'badge badge-error', key: 'roles.admin' },
        developer: { class: 'badge badge-primary', key: 'roles.developer' },
        owner: { class: 'badge badge-error', key: 'roles.owner' },
    };

    const info = mapping[role] || { class: 'badge badge-outline', key: `roles.${role}` };
    return { text: t(info.key), className: info.class };
}

export default getRoleBadge;
