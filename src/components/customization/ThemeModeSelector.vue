<template>
    <div
        class="card bg-base-200 shadow-md border border-base-300 p-6 transition-all duration-300 lg:col-span-12"
    >
        <h2 class="card-title flex items-center gap-2">
            <SunMoon class="w-5 h-5 text-primary" />
            {{ t("theme.select_theme") }}
        </h2>
        <p class="text-base-content/70 mb-4">
            {{ t("theme.description") }}
        </p>

        <div class="flex flex-col lg:flex-row gap-4 items-start">
            <div class="w-full lg:flex-1">
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                    <button
                        type="button"
                        class="group flex flex-col gap-2 rounded-xl border p-2.5 text-left transition-all duration-200 outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
                        :class="
                            themeMode === 'light'
                                ? 'border-primary bg-primary/5 shadow-sm'
                                : 'border-base-300 bg-base-100/40 hover:border-base-content/20 hover:bg-base-100/70'
                        "
                        @click="$emit('select-mode', 'light')"
                    >
                        <div
                            data-theme="light"
                            class="w-full aspect-[16/10] rounded-lg border border-base-300 overflow-hidden bg-base-100 shadow-sm"
                        >
                            <div class="h-full w-full flex">
                                <div class="w-[18%] border-r border-base-content/10 bg-base-300 flex flex-col items-center py-1.5 gap-1.5">
                                    <div class="w-4 h-4 rounded-md bg-primary text-primary-content shadow-sm"></div>
                                    <div class="w-4 h-4 rounded-md bg-base-content/10"></div>
                                    <div class="w-4 h-4 rounded-md bg-base-content/10"></div>
                                    <div class="mt-auto"></div>
                                    <div class="w-4 h-4 rounded-md bg-base-content/10"></div>
                                    <div class="w-4 h-4 rounded-md bg-base-content/10"></div>
                                </div>
                                <div class="flex-1 p-1.5 flex flex-col gap-1.5 min-w-0 bg-base-100">
                                    <div class="flex items-center gap-1">
                                        <div class="h-4 flex-1 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                    </div>
                                    <div class="flex-1 grid grid-cols-3 gap-1 min-h-0">
                                        <div class="rounded-md border border-base-content/10 bg-base-300 shadow-sm p-1 flex flex-col">
                                            <div class="h-1 w-3/4 rounded bg-base-content/30"></div>
                                            <div class="mt-0.5 flex items-center gap-0.5">
                                                <div class="w-1 h-1 rounded-full bg-base-content/20"></div>
                                                <div class="h-0.5 w-4 rounded bg-base-content/20"></div>
                                            </div>
                                            <div class="mt-auto flex justify-end pt-0.5">
                                                <div class="h-2.5 w-7 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded-md border border-base-content/10 bg-base-300 shadow-sm p-1 flex flex-col">
                                            <div class="h-1 w-2/3 rounded bg-base-content/30"></div>
                                            <div class="mt-0.5 flex items-center gap-0.5">
                                                <div class="w-1 h-1 rounded-full bg-base-content/20"></div>
                                                <div class="h-0.5 w-4 rounded bg-base-content/20"></div>
                                            </div>
                                            <div class="mt-auto flex justify-end pt-0.5">
                                                <div class="h-2.5 w-7 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded-md border border-base-content/10 bg-base-300 shadow-sm p-1 flex flex-col">
                                            <div class="h-1 w-4/5 rounded bg-base-content/30"></div>
                                            <div class="mt-0.5 flex items-center gap-0.5">
                                                <div class="w-1 h-1 rounded-full bg-base-content/20"></div>
                                                <div class="h-0.5 w-3 rounded bg-base-content/20"></div>
                                            </div>
                                            <div class="mt-auto flex justify-end pt-0.5">
                                                <div class="h-2.5 w-7 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded-md border border-base-content/10 bg-base-300 shadow-sm p-1 flex flex-col">
                                            <div class="h-1 w-2/3 rounded bg-base-content/30"></div>
                                            <div class="mt-0.5 flex items-center gap-0.5">
                                                <div class="w-1 h-1 rounded-full bg-base-content/20"></div>
                                                <div class="h-0.5 w-4 rounded bg-base-content/20"></div>
                                            </div>
                                            <div class="mt-auto flex justify-end pt-0.5">
                                                <div class="h-2.5 w-7 rounded bg-base-300 border border-base-content/10"></div>
                                            </div>
                                        </div>
                                        <div class="rounded-md border border-base-content/10 bg-base-300 shadow-sm p-1 flex flex-col">
                                            <div class="h-1 w-3/4 rounded bg-base-content/30"></div>
                                            <div class="mt-0.5 flex items-center gap-0.5">
                                                <div class="w-1 h-1 rounded-full bg-base-content/20"></div>
                                                <div class="h-0.5 w-3 rounded bg-base-content/20"></div>
                                            </div>
                                            <div class="mt-auto flex justify-end pt-0.5">
                                                <div class="h-2.5 w-7 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded-md border border-base-content/10 bg-base-300 shadow-sm p-1 flex flex-col">
                                            <div class="h-1 w-1/2 rounded bg-base-content/30"></div>
                                            <div class="mt-0.5 flex items-center gap-0.5">
                                                <div class="w-1 h-1 rounded-full bg-base-content/20"></div>
                                                <div class="h-0.5 w-4 rounded bg-base-content/20"></div>
                                            </div>
                                            <div class="mt-auto flex justify-end pt-0.5">
                                                <div class="h-2.5 w-7 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="flex items-center justify-center gap-1.5">
                            <Sun class="w-4 h-4 text-warning" />
                            <span class="text-sm font-medium">{{
                                t("theme.light")
                            }}</span>
                        </div>
                    </button>

                    <button
                        type="button"
                        class="group flex flex-col gap-2 rounded-xl border p-2.5 text-left transition-all duration-200 outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
                        :class="
                            themeMode === 'dark'
                                ? 'border-primary bg-primary/5 shadow-sm'
                                : 'border-base-300 bg-base-100/40 hover:border-base-content/20 hover:bg-base-100/70'
                        "
                        @click="$emit('select-mode', 'dark')"
                    >
                        <div
                            data-theme="dark"
                            class="w-full aspect-[16/10] rounded-lg border border-base-300 overflow-hidden bg-base-100 shadow-sm"
                        >
                            <div class="h-full w-full flex">
                                <div class="w-[18%] border-r border-base-content/10 bg-base-300 flex flex-col items-center py-1.5 gap-1.5">
                                    <div class="w-4 h-4 rounded-md bg-primary text-primary-content shadow-sm"></div>
                                    <div class="w-4 h-4 rounded-md bg-base-content/10"></div>
                                    <div class="w-4 h-4 rounded-md bg-base-content/10"></div>
                                    <div class="mt-auto"></div>
                                    <div class="w-4 h-4 rounded-md bg-base-content/10"></div>
                                    <div class="w-4 h-4 rounded-md bg-base-content/10"></div>
                                </div>
                                <div class="flex-1 p-1.5 flex flex-col gap-1.5 min-w-0 bg-base-100">
                                    <div class="flex items-center gap-1">
                                        <div class="h-4 flex-1 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                    </div>
                                    <div class="flex-1 grid grid-cols-3 gap-1 min-h-0">
                                        <div class="rounded-md border border-base-content/10 bg-base-300 shadow-sm p-1 flex flex-col">
                                            <div class="h-1 w-3/4 rounded bg-base-content/30"></div>
                                            <div class="mt-0.5 flex items-center gap-0.5">
                                                <div class="w-1 h-1 rounded-full bg-base-content/20"></div>
                                                <div class="h-0.5 w-4 rounded bg-base-content/20"></div>
                                            </div>
                                            <div class="mt-auto flex justify-end pt-0.5">
                                                <div class="h-2.5 w-7 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded-md border border-base-content/10 bg-base-300 shadow-sm p-1 flex flex-col">
                                            <div class="h-1 w-2/3 rounded bg-base-content/30"></div>
                                            <div class="mt-0.5 flex items-center gap-0.5">
                                                <div class="w-1 h-1 rounded-full bg-base-content/20"></div>
                                                <div class="h-0.5 w-4 rounded bg-base-content/20"></div>
                                            </div>
                                            <div class="mt-auto flex justify-end pt-0.5">
                                                <div class="h-2.5 w-7 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded-md border border-base-content/10 bg-base-300 shadow-sm p-1 flex flex-col">
                                            <div class="h-1 w-4/5 rounded bg-base-content/30"></div>
                                            <div class="mt-0.5 flex items-center gap-0.5">
                                                <div class="w-1 h-1 rounded-full bg-base-content/20"></div>
                                                <div class="h-0.5 w-3 rounded bg-base-content/20"></div>
                                            </div>
                                            <div class="mt-auto flex justify-end pt-0.5">
                                                <div class="h-2.5 w-7 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded-md border border-base-content/10 bg-base-300 shadow-sm p-1 flex flex-col">
                                            <div class="h-1 w-2/3 rounded bg-base-content/30"></div>
                                            <div class="mt-0.5 flex items-center gap-0.5">
                                                <div class="w-1 h-1 rounded-full bg-base-content/20"></div>
                                                <div class="h-0.5 w-4 rounded bg-base-content/20"></div>
                                            </div>
                                            <div class="mt-auto flex justify-end pt-0.5">
                                                <div class="h-2.5 w-7 rounded bg-base-300 border border-base-content/10"></div>
                                            </div>
                                        </div>
                                        <div class="rounded-md border border-base-content/10 bg-base-300 shadow-sm p-1 flex flex-col">
                                            <div class="h-1 w-3/4 rounded bg-base-content/30"></div>
                                            <div class="mt-0.5 flex items-center gap-0.5">
                                                <div class="w-1 h-1 rounded-full bg-base-content/20"></div>
                                                <div class="h-0.5 w-3 rounded bg-base-content/20"></div>
                                            </div>
                                            <div class="mt-auto flex justify-end pt-0.5">
                                                <div class="h-2.5 w-7 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded-md border border-base-content/10 bg-base-300 shadow-sm p-1 flex flex-col">
                                            <div class="h-1 w-1/2 rounded bg-base-content/30"></div>
                                            <div class="mt-0.5 flex items-center gap-0.5">
                                                <div class="w-1 h-1 rounded-full bg-base-content/20"></div>
                                                <div class="h-0.5 w-4 rounded bg-base-content/20"></div>
                                            </div>
                                            <div class="mt-auto flex justify-end pt-0.5">
                                                <div class="h-2.5 w-7 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="flex items-center justify-center gap-1.5">
                            <Moon class="w-4 h-4 text-info" />
                            <span class="text-sm font-medium">{{
                                t("theme.dark")
                            }}</span>
                        </div>
                    </button>

                    <button
                        type="button"
                        class="group flex flex-col gap-2 rounded-xl border p-2.5 text-left transition-all duration-200 outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
                        :class="
                            themeMode === 'system'
                                ? 'border-primary bg-primary/5 shadow-sm'
                                : 'border-base-300 bg-base-100/40 hover:border-base-content/20 hover:bg-base-100/70'
                        "
                        @click="$emit('select-mode', 'system')"
                    >
                        <div
                            class="w-full aspect-[16/10] rounded-lg border border-base-300 overflow-hidden bg-base-100 shadow-sm flex"
                        >
                            <div data-theme="light" class="w-1/2 h-full flex">
                                <div class="w-[22%] border-r border-base-content/10 bg-base-300 flex flex-col items-center py-1 gap-1">
                                    <div class="w-3 h-3 rounded-sm bg-primary"></div>
                                    <div class="w-3 h-3 rounded-sm bg-base-content/10"></div>
                                    <div class="w-3 h-3 rounded-sm bg-base-content/10"></div>
                                    <div class="mt-auto w-3 h-3 rounded-sm bg-base-content/10"></div>
                                    <div class="w-3 h-3 rounded-sm bg-base-content/10"></div>
                                </div>
                                <div class="flex-1 p-1 flex flex-col gap-1 min-w-0 bg-base-100">
                                    <div class="flex gap-0.5">
                                        <div class="h-3 flex-1 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-3 w-3 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-3 w-3 rounded border border-base-300 bg-base-200/70"></div>
                                    </div>
                                    <div class="flex-1 grid grid-cols-2 gap-0.5 min-h-0">
                                        <div class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col">
                                            <div class="h-0.5 w-3/4 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col">
                                            <div class="h-0.5 w-2/3 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col">
                                            <div class="h-0.5 w-4/5 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col">
                                            <div class="h-0.5 w-1/2 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-base-300 border border-base-content/10"></div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div data-theme="dark" class="w-1/2 h-full flex border-l border-base-300">
                                <div class="w-[22%] border-r border-base-content/10 bg-base-300 flex flex-col items-center py-1 gap-1">
                                    <div class="w-3 h-3 rounded-sm bg-primary"></div>
                                    <div class="w-3 h-3 rounded-sm bg-base-content/10"></div>
                                    <div class="w-3 h-3 rounded-sm bg-base-content/10"></div>
                                    <div class="mt-auto w-3 h-3 rounded-sm bg-base-content/10"></div>
                                    <div class="w-3 h-3 rounded-sm bg-base-content/10"></div>
                                </div>
                                <div class="flex-1 p-1 flex flex-col gap-1 min-w-0 bg-base-100">
                                    <div class="flex gap-0.5">
                                        <div class="h-3 flex-1 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-3 w-3 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-3 w-3 rounded border border-base-300 bg-base-200/70"></div>
                                    </div>
                                    <div class="flex-1 grid grid-cols-2 gap-0.5 min-h-0">
                                        <div class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col">
                                            <div class="h-0.5 w-3/4 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col">
                                            <div class="h-0.5 w-2/3 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col">
                                            <div class="h-0.5 w-4/5 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col">
                                            <div class="h-0.5 w-1/2 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-base-300 border border-base-content/10"></div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="flex items-center justify-center gap-1.5">
                            <Monitor class="w-4 h-4 text-success" />
                            <span class="text-sm font-medium">{{
                                t("theme.system")
                            }}</span>
                        </div>
                    </button>

                    <button
                        type="button"
                        class="group flex flex-col gap-2 rounded-xl border p-2.5 text-left transition-all duration-200 outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
                        :class="
                            themeMode === 'schedule'
                                ? 'border-primary bg-primary/5 shadow-sm'
                                : 'border-base-300 bg-base-100/40 hover:border-base-content/20 hover:bg-base-100/70'
                        "
                        @click="$emit('select-mode', 'schedule')"
                    >
                        <div
                            class="w-full aspect-[16/10] rounded-lg border border-base-300 overflow-hidden bg-base-100 shadow-sm flex relative"
                        >
                            <div data-theme="light" class="w-1/2 h-full flex">
                                <div class="w-[22%] border-r border-base-content/10 bg-base-300 flex flex-col items-center py-1 gap-1">
                                    <div class="w-3 h-3 rounded-sm bg-primary"></div>
                                    <div class="w-3 h-3 rounded-sm bg-base-content/10"></div>
                                    <div class="w-3 h-3 rounded-sm bg-base-content/10"></div>
                                    <div class="mt-auto w-3 h-3 rounded-sm bg-base-content/10"></div>
                                </div>
                                <div class="flex-1 p-1 flex flex-col gap-1 min-w-0 bg-base-100">
                                    <div class="flex gap-0.5">
                                        <div class="h-3 flex-1 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-3 w-3 rounded border border-base-300 bg-base-200/70"></div>
                                    </div>
                                    <div class="flex-1 grid grid-cols-2 gap-0.5 min-h-0">
                                        <div class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col">
                                            <div class="h-0.5 w-3/4 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col">
                                            <div class="h-0.5 w-2/3 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col col-span-2">
                                            <div class="h-0.5 w-1/2 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div data-theme="dark" class="w-1/2 h-full flex border-l border-base-300">
                                <div class="w-[22%] border-r border-base-content/10 bg-base-300 flex flex-col items-center py-1 gap-1">
                                    <div class="w-3 h-3 rounded-sm bg-primary"></div>
                                    <div class="w-3 h-3 rounded-sm bg-base-content/10"></div>
                                    <div class="w-3 h-3 rounded-sm bg-base-content/10"></div>
                                    <div class="mt-auto w-3 h-3 rounded-sm bg-base-content/10"></div>
                                </div>
                                <div class="flex-1 p-1 flex flex-col gap-1 min-w-0 bg-base-100">
                                    <div class="flex gap-0.5">
                                        <div class="h-3 flex-1 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-3 w-3 rounded border border-base-300 bg-base-200/70"></div>
                                    </div>
                                    <div class="flex-1 grid grid-cols-2 gap-0.5 min-h-0">
                                        <div class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col">
                                            <div class="h-0.5 w-3/4 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col">
                                            <div class="h-0.5 w-2/3 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                        <div class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col col-span-2">
                                            <div class="h-0.5 w-1/2 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div
                                class="absolute inset-0 flex items-center justify-center pointer-events-none"
                            >
                                <div
                                    class="rounded-full bg-base-100 border border-base-300 p-1 shadow-sm"
                                >
                                    <Clock class="w-3.5 h-3.5 text-primary" />
                                </div>
                            </div>
                        </div>
                        <div class="flex items-center justify-center gap-1.5">
                            <Clock class="w-4 h-4 text-primary" />
                            <span class="text-sm font-medium">{{
                                t("theme.schedule.title")
                            }}</span>
                        </div>
                    </button>
                </div>
            </div>

            <transition name="schedule-slide">
                <div
                    v-if="themeMode === 'system'"
                    class="w-full lg:w-80 shrink-0 flex flex-col gap-3 border border-base-300 bg-base-100/50 rounded-xl p-4"
                >
                    <div
                        class="flex items-center gap-2 font-medium text-sm text-base-content"
                    >
                        <Monitor class="w-4 h-4 shrink-0 text-primary" />
                        <span>{{ t("theme.system") }}</span>
                    </div>

                    <p class="text-xs text-base-content/60">
                        {{ t("theme.system_description") }}
                    </p>

                    <div
                        class="flex items-center justify-between gap-2 p-2 mt-1 rounded-lg bg-base-200 border border-base-300"
                    >
                        <div class="flex items-center gap-2">
                            <div
                                class="w-2 h-2 rounded-full shrink-0 transition-colors duration-500"
                                :class="
                                    currentSystemTheme === 'light'
                                        ? 'bg-warning'
                                        : 'bg-info'
                                "
                            ></div>
                            <span class="text-xs text-base-content/60">
                                {{ t("theme.system_detected") }}
                            </span>
                        </div>
                        <span class="text-xs font-semibold text-base-content">
                            {{ t(`theme.${currentSystemTheme}`) }}
                        </span>
                    </div>
                </div>
            </transition>

            <transition name="schedule-slide">
                <div
                    v-if="themeMode === 'schedule'"
                    class="w-full lg:w-80 shrink-0 flex flex-col gap-3 border border-base-300 bg-base-100/50 rounded-xl p-4"
                >
                    <div
                        class="flex items-center gap-2 font-medium text-sm text-base-content"
                    >
                        <Clock class="w-4 h-4 shrink-0 text-primary" />
                        <span>{{ t("theme.schedule.light_window") }}</span>
                    </div>

                    <div class="grid grid-cols-2 gap-3">
                        <div class="flex flex-col gap-1">
                            <label class="text-sm text-base-content/60">{{
                                t("theme.schedule.from")
                            }}</label>
                            <input
                                type="time"
                                class="input input-bordered input-sm w-full"
                                :value="scheduleLightStart"
                                @change="
                                    $emit(
                                        'update-schedule',
                                        'lightStart',
                                        ($event.target as HTMLInputElement)
                                            .value
                                    )
                                "
                            />
                        </div>
                        <div class="flex flex-col gap-1">
                            <label class="text-sm text-base-content/60">{{
                                t("theme.schedule.to")
                            }}</label>
                            <input
                                type="time"
                                class="input input-bordered input-sm w-full"
                                :value="scheduleLightEnd"
                                @change="
                                    $emit(
                                        'update-schedule',
                                        'lightEnd',
                                        ($event.target as HTMLInputElement)
                                            .value
                                    )
                                "
                            />
                        </div>
                    </div>

                    <div
                        class="flex items-center justify-between gap-2 p-2 mt-1 rounded-lg bg-base-200 border border-base-300"
                    >
                        <div class="flex items-center gap-2">
                            <div
                                class="w-2 h-2 rounded-full shrink-0 transition-colors duration-500"
                                :class="
                                    schedulePreviewTheme === 'light'
                                        ? 'bg-warning'
                                        : 'bg-info'
                                "
                            ></div>
                            <span class="text-xs text-base-content/60">
                                {{ t("theme.schedule.now_active") }}
                            </span>
                        </div>
                        <span class="text-xs font-semibold text-base-content">
                            {{ t(`theme.${schedulePreviewTheme}`) }}
                        </span>
                    </div>
                </div>
            </transition>
        </div>
    </div>
</template>

<script setup lang="ts">
import { SunMoon, Moon, Sun, Clock, Monitor } from "@lucide/vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

defineProps<{
    themeMode: "dark" | "light" | "system" | "schedule";
    scheduleLightStart: string;
    scheduleLightEnd: string;
    schedulePreviewTheme: string;
    currentSystemTheme: "dark" | "light";
}>();

defineEmits<{
    "select-mode": [mode: "dark" | "light" | "system" | "schedule"];
    "update-schedule": [field: "lightStart" | "lightEnd", value: string];
}>();
</script>

<style scoped>
.schedule-slide-enter-active,
.schedule-slide-leave-active {
    transition: all 0.25s ease;
}

.schedule-slide-enter-from,
.schedule-slide-leave-to {
    opacity: 0;
    transform: translateX(12px);
}
</style>
