<template>
    <div class="container mx-auto mt-4">
        <div key="theme" class="grid grid-cols-1 lg:grid-cols-12 gap-8">
            <div
                class="card bg-base-200 shadow-md border border-base-300 p-6 transition-all duration-300"
                :class="themeMode === 'schedule' ? 'lg:col-span-7' : 'lg:col-span-4'"
            >
                <h2 class="card-title flex items-center gap-2">
                    <SunMoon class="w-5 h-5 text-primary" />
                    {{ t("theme.select_theme") }}
                </h2>
                <p class="text-base-content/70 mb-4">
                    {{ t("theme.description") }}
                </p>

                <div :class="['grid', themeMode === 'schedule' ? 'grid-cols-2 sm:grid-cols-2' : 'grid-cols-1', 'gap-4', 'items-start']">
                    <div class="flex flex-col gap-3">
                        <button
                            @click="selectThemeMode('dark')"
                            class="btn border flex items-center justify-between px-6 py-3"
                            :class="{
                                'border-primary/50 bg-primary/10': themeMode === 'dark',
                                'border-base-content/10': themeMode !== 'dark',
                            }"
                        >
                            <div class="flex items-center gap-2">
                                <Moon class="w-5 h-5 text-indigo-400" />
                                <span class="font-medium">{{ t("theme.dark") }}</span>
                            </div>
                            <div v-if="themeMode === 'dark'" class="badge badge-primary">
                                {{ t("theme.selected") }}
                            </div>
                        </button>

                        <button
                            @click="selectThemeMode('light')"
                            class="btn border flex items-center justify-between px-6 py-3"
                            :class="{
                                'border-primary/50 bg-primary/10': themeMode === 'light',
                                'border-base-content/10': themeMode !== 'light',
                            }"
                        >
                            <div class="flex items-center gap-2">
                                <Sun class="w-5 h-5 text-amber-400" />
                                <span class="font-medium">{{ t("theme.light") }}</span>
                            </div>
                            <div v-if="themeMode === 'light'" class="badge badge-primary">
                                {{ t("theme.selected") }}
                            </div>
                        </button>

                        <button
                            @click="selectThemeMode('schedule')"
                            class="btn border flex items-center justify-between px-6 py-3 transition-all duration-300"
                            :class="{
                                'border-primary/50 bg-primary/10': themeMode === 'schedule',
                                'border-base-content/10': themeMode !== 'schedule',
                            }"
                        >
                            <div class="flex items-center gap-2">
                                <div class="relative w-5 h-5 shrink-0">
                                    <Sun
                                        class="absolute inset-0 w-5 h-5 text-amber-400 transition-all duration-500"
                                        :class="themeMode === 'schedule' ? 'opacity-100 scale-100' : 'opacity-60 scale-90'"
                                    />
                                    <Moon
                                        class="absolute inset-0 w-3 h-3 text-indigo-400 transition-all duration-500"
                                        :class="themeMode === 'schedule' ? 'opacity-100 translate-x-2.5 translate-y-2.5' : 'opacity-0 translate-x-1 translate-y-1'"
                                    />
                                </div>
                                <span class="font-medium">{{ t("theme.schedule.title") }}</span>
                            </div>
                            <div v-if="themeMode === 'schedule'" class="badge badge-primary">
                                {{ t("theme.selected") }}
                            </div>
                        </button>
                    </div>

                    <div
                        v-if="themeMode === 'schedule'"
                        class="flex flex-col gap-3 border border-primary/20 bg-primary/5 rounded-lg p-4"
                    >
                            <div class="flex items-center gap-2 font-medium text-sm text-primary">
                                <Clock class="w-4 h-4 shrink-0" />
                                <span>{{ t("theme.schedule.light_window") }}</span>
                            </div>

                            <div class="grid grid-cols-2 gap-3">
                                <div class="flex flex-col gap-1">
                                    <label class="text-sm text-base-content/60">{{
                                        t("theme.schedule.from")
                                    }}</label>
                                    <input
                                        type="time"
                                        class="input input-sm input-bordered w-full"
                                        :value="scheduleLightStart"
                                        @change="
                                            updateScheduleTime(
                                                'lightStart',
                                                ($event.target as HTMLInputElement).value
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
                                        class="input input-sm input-bordered w-full"
                                        :value="scheduleLightEnd"
                                        @change="
                                            updateScheduleTime(
                                                'lightEnd',
                                                ($event.target as HTMLInputElement).value
                                            )
                                        "
                                    />
                                </div>
                            </div>

                            <div class="flex items-center gap-2 p-2 rounded-md bg-base-100/60">
                                <div
                                    class="w-2 h-2 rounded-full shrink-0 transition-colors duration-500"
                                    :class="
                                        schedulePreviewTheme === 'light'
                                            ? 'bg-amber-400'
                                            : 'bg-indigo-400'
                                    "
                                ></div>
                                <span class="text-sm text-base-content/60">
                                    {{ t("theme.schedule.now_active") }}:
                                    <span class="font-medium text-base-content">{{
                                        t(`theme.${schedulePreviewTheme}`)
                                    }}</span>
                                </span>
                            </div>
                        </div>
                </div>
            </div>
        </div>

        <div
            class="card bg-base-200 shadow-md border border-base-300 mb-6 mt-6"
        >
            <div class="card-body">
                <div
                    class="flex flex-col md:flex-row md:items-center md:justify-between gap-4"
                >
                    <h1 class="card-title text-2xl flex items-center gap-3">
                        <Save class="w-6 h-6 text-primary" />
                        {{ t("theme.preset") }}
                    </h1>

                    <div class="flex flex-wrap items-center gap-2 justify-end">
                        <div class="flex gap-2 flex-wrap">
                            <button
                                class="btn btn-accent btn-sm flex items-center gap-2"
                                @click="$emit('change-view', 'marketplace')"
                            >
                                <Store class="w-4 h-4" />
                                <span class="hidden sm:inline">{{
                                    t("marketplace.open_marketplace")
                                }}</span>
                            </button>

                            <button
                                class="btn btn-outline btn-sm flex items-center gap-2"
                                @click="resetStyles"
                            >
                                <RotateCcw class="w-4 h-4" />
                                <span class="hidden sm:inline">{{
                                    t("theme.reset_button")
                                }}</span>
                            </button>

                            <button
                                v-if="!isExternalWindow"
                                class="btn btn-outline btn-primary btn-sm flex items-center gap-2"
                                @click="openInNewWindow"
                            >
                                <ExternalLink class="w-4 h-4" />
                                <span class="hidden sm:inline">{{
                                    $t("theme.actions.open_inspector")
                                }}</span>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div class="card bg-base-200 shadow-md border border-base-300 mb-6">
            <div class="card-body">
                <PresetManager />
            </div>
        </div>

        <div class="lg:col-span-8">
            <div class="card bg-base-200 shadow-md border border-base-300">
                <div class="card-body p-6">
                    <h2 class="card-title flex items-center gap-2">
                        <Palette class="w-6 h-6 text-primary" />
                        {{ t("theme.colors") }}
                    </h2>

                    <div class="mb-8">
                        <h3
                            class="text-xl font-semibold mb-4 text-base-content"
                        >
                            {{ t("theme.base_colors") }}
                        </h3>
                        <div
                            class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4"
                        >
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.base100") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="base100"
                                    @input="
                                        handleColorInput(
                                            'base100',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.base200") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="base200"
                                    @input="
                                        handleColorInput(
                                            'base200',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.base300") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="base300"
                                    @input="
                                        handleColorInput(
                                            'base300',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.base_content") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="baseContent"
                                    @input="
                                        handleColorInput(
                                            'baseContent',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                        </div>
                    </div>

                    <div class="mb-8">
                        <h3
                            class="text-xl font-semibold mb-4 text-base-content"
                        >
                            {{ t("theme.primary_secondary_accent") }}
                        </h3>
                        <div
                            class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4"
                        >
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.primary") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="primaryColor"
                                    @input="
                                        handleColorInput(
                                            'primaryColorOverride',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.primary_content") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="primaryContent"
                                    @input="
                                        handleColorInput(
                                            'primaryContent',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.secondary") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="secondary"
                                    @input="
                                        handleColorInput(
                                            'secondary',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.secondary_content") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="secondaryContent"
                                    @input="
                                        handleColorInput(
                                            'secondaryContent',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.accent") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="accent"
                                    @input="
                                        handleColorInput(
                                            'accent',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.accent_content") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="accentContent"
                                    @input="
                                        handleColorInput(
                                            'accentContent',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                        </div>
                    </div>

                    <div>
                        <h3
                            class="text-xl font-semibold mb-4 text-base-content"
                        >
                            {{ t("theme.semantic_colors") }}
                        </h3>
                        <div
                            class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4"
                        >
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.neutral") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="neutral"
                                    @input="
                                        handleColorInput(
                                            'neutral',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.neutral_content") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="neutralContent"
                                    @input="
                                        handleColorInput(
                                            'neutralContent',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.info") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="info"
                                    @input="
                                        handleColorInput(
                                            'info',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.info_content") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="infoContent"
                                    @input="
                                        handleColorInput(
                                            'infoContent',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.success") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="success"
                                    @input="
                                        handleColorInput(
                                            'success',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.success_content") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="successContent"
                                    @input="
                                        handleColorInput(
                                            'successContent',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.warning") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="warning"
                                    @input="
                                        handleColorInput(
                                            'warning',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.warning_content") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="warningContent"
                                    @input="
                                        handleColorInput(
                                            'warningContent',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.error") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="error"
                                    @input="
                                        handleColorInput(
                                            'error',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                            <div class="form-control">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{ t("theme.error_content") }}</label
                                >
                                <input
                                    type="color"
                                    class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
                                    :value="errorContent"
                                    @input="
                                        handleColorInput(
                                            'errorContent',
                                            ($event.target as HTMLInputElement)
                                                .value
                                        )
                                    "
                                />
                            </div>
                        </div>
                    </div>

                    <div class="mt-8">
                        <h3
                            class="text-xl font-semibold mb-4 text-base-content"
                        >
                            {{ t("customization.background_title") }}
                        </h3>
                        <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
                            <div class="lg:col-span-12">
                                <label
                                    class="label text-sm font-medium text-base-content"
                                    >{{
                                        t("customization.background_image")
                                    }}</label
                                >
                                <div class="relative">
                                    <input
                                        type="text"
                                        class="input input-bordered w-full pr-10"
                                        :value="backgroundImage"
                                        :placeholder="
                                            t(
                                                'customization.background_image_placeholder'
                                            )
                                        "
                                        @input="
                                            handleBackgroundInput(
                                                'backgroundImage',
                                                (
                                                    $event.target as HTMLInputElement
                                                ).value
                                            )
                                        "
                                    />
                                    <button
                                        v-if="backgroundImage"
                                        class="absolute right-2 top-1/2 -translate-y-1/2 btn btn-xs btn-ghost"
                                        @click="
                                            handleBackgroundInput(
                                                'backgroundImage',
                                                ''
                                            )
                                        "
                                    >
                                        &times;
                                    </button>
                                </div>
                                <p class="text-xs text-base-content/50 mt-1">
                                    {{
                                        t("customization.background_image_help")
                                    }}
                                </p>
                            </div>

                            <div class="lg:col-span-6">
                                <div class="flex justify-between mb-2">
                                    <label
                                        class="text-sm font-medium text-base-content"
                                        >{{
                                            t("customization.background_blur")
                                        }}</label
                                    >
                                    <span class="text-xs font-mono"
                                        >{{ backgroundBlur ?? 0 }}px</span
                                    >
                                </div>
                                <input
                                    type="range"
                                    min="0"
                                    max="20"
                                    step="1"
                                    class="range range-primary range-sm"
                                    :value="backgroundBlur ?? 0"
                                    @input="
                                        handleBackgroundInput(
                                            'backgroundBlur',
                                            Number(
                                                (
                                                    $event.target as HTMLInputElement
                                                ).value
                                            )
                                        )
                                    "
                                />
                            </div>

                            <div class="lg:col-span-6">
                                <div class="flex justify-between mb-2">
                                    <label
                                        class="text-sm font-medium text-base-content"
                                        >{{
                                            t(
                                                "customization.background_opacity"
                                            )
                                        }}</label
                                    >
                                    <span class="text-xs font-mono"
                                        >{{ backgroundOpacity ?? 100 }}%</span
                                    >
                                </div>
                                <input
                                    type="range"
                                    min="0"
                                    max="100"
                                    step="1"
                                    class="range range-primary range-sm"
                                    :value="backgroundOpacity ?? 100"
                                    @input="
                                        handleBackgroundInput(
                                            'backgroundOpacity',
                                            Number(
                                                (
                                                    $event.target as HTMLInputElement
                                                ).value
                                            )
                                        )
                                    "
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="card bg-base-200 shadow-md border border-base-300 mt-6">
                <div class="card-body">
                    <div
                        @click="toggleExpertMode"
                        class="cursor-pointer flex items-center justify-between"
                    >
                        <h2 class="card-title flex items-center gap-2">
                            <Code class="w-5 h-5 text-primary" />
                            {{ t("theme.expert_css_title") }}
                        </h2>
                        <button class="btn btn-sm btn-ghost">
                            <ChevronDown
                                v-if="!showExpertOptions"
                                class="w-5 h-5"
                            />
                            <ChevronUp v-else class="w-5 h-5" />
                            {{
                                showExpertOptions
                                    ? t("theme.hide_expert")
                                    : t("theme.show_expert")
                            }}
                        </button>
                    </div>

                    <transition
                        name="expert-fade"
                        @before-enter="expertAnimationActive = true"
                        @after-leave="expertAnimationActive = false"
                    >
                        <div v-if="showExpertOptions" class="mt-4">
                            <div
                                class="bg-warning/10 border border-warning/20 rounded-lg p-4 mb-4"
                            >
                                <div class="flex items-start gap-2">
                                    <HelpCircle
                                        class="w-5 h-5 text-warning shrink-0 mt-0.5"
                                    />
                                    <p class="text-sm text-warning">
                                        {{ t("theme.expert_warning") }}
                                    </p>
                                </div>
                            </div>

                            <div class="flex items-center justify-between mb-2">
                                <label class="flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="checkbox"
                                        v-model="enableCustomCSS"
                                        @change="
                                            handleEnableCustomCSS(
                                                (
                                                    $event.target as HTMLInputElement
                                                )?.checked ?? false
                                            )
                                        "
                                    />
                                    <span>{{
                                        t("theme.enable_custom_css")
                                    }}</span>
                                </label>
                            </div>
                            <div class="flex flex-col gap-2 mb-4">
                                <label class="font-medium mb-1">{{
                                    t("theme.available_classes_label")
                                }}</label>
                                <div class="flex flex-wrap gap-2">
                                    <span
                                        class="bg-base-300 text-xs px-3 py-1 rounded-full font-mono text-base-content/80 border border-base-200 tooltip tooltip-right cursor-pointer"
                                        :data-tip="
                                            t('theme.tooltip_client_card')
                                        "
                                        @click="addExample('.client-card')"
                                    >
                                        client-card
                                    </span>
                                    <span
                                        class="bg-base-300 text-xs px-3 py-1 rounded-full font-mono text-base-content/80 border border-base-200 tooltip tooltip-right cursor-pointer"
                                        :data-tip="
                                            t('theme.tooltip_sidebar_btn')
                                        "
                                        @click="addExample('.sidebar-btn')"
                                    >
                                        sidebar-btn
                                    </span>
                                    <span
                                        class="bg-base-300 text-xs px-3 py-1 rounded-full font-mono text-base-content/80 border border-base-200 tooltip tooltip-right cursor-pointer"
                                        :data-tip="
                                            t(
                                                'theme.tooltip_launch_download_btn'
                                            )
                                        "
                                        @click="
                                            addExample(
                                                '.launch-btn, .download-btn'
                                            )
                                        "
                                    >
                                        download-btn | launch-btn
                                    </span>
                                </div>
                            </div>

                            <div
                                class="grid grid-cols-1 lg:grid-cols-2 gap-4 mt-4"
                            >
                                <div>
                                    <label class="block mb-2 font-medium">{{
                                        t("theme.custom_css_label")
                                    }}</label>
                                    <VueMonacoEditor
                                        v-model:value="customCSS"
                                        language="css"
                                        :theme="
                                            selectedTheme === 'dark'
                                                ? 'vs-dark'
                                                : 'vs'
                                        "
                                        :options="{
                                            readOnly: !enableCustomCSS,
                                            minimap: { enabled: false },
                                            fontSize: 14,
                                            lineNumbers: 'on',
                                            wordWrap: 'on',
                                            automaticLayout: true,
                                            scrollBeyondLastLine: false,
                                        }"
                                        style="
                                            height: 300px;
                                            border-radius: 0.5rem;
                                            border: 1px solid
                                                rgba(255, 255, 255, 0.1);
                                        "
                                    />
                                </div>
                            </div>
                            <div class="flex gap-2 mt-4">
                                <button
                                    class="btn btn-primary btn-sm flex items-center gap-2"
                                    @click="openExportModal"
                                >
                                    <ClipboardCopy class="w-4 h-4" />
                                    {{ t("theme.export_css_btn") }}
                                </button>
                                <button
                                    class="btn btn-secondary btn-sm flex items-center gap-2"
                                    @click="openImportModal"
                                >
                                    <ClipboardPaste class="w-4 h-4" />
                                    {{ t("theme.import_css_btn") }}
                                </button>
                            </div>
                            <div class="mt-6">
                                <h3 class="font-medium text-sm mb-3">
                                    {{ t("theme.css_examples_title") }}
                                </h3>
                                <div
                                    class="grid grid-cols-1 md:grid-cols-3 gap-4"
                                >
                                    <div
                                        v-for="(example, index) in cssExamples"
                                        :key="index"
                                        class="card shadow-md border border-base-300"
                                    >
                                        <div class="card-body p-4">
                                            <h4 class="card-title text-sm">
                                                {{ example.title }}
                                            </h4>
                                            <pre
                                                class="text-xs bg-base-300 p-2 rounded overflow-x-auto mt-2"
                                            ><code>{{ example.code }}</code></pre>
                                            <button
                                                @click="
                                                    insertExample(example.code)
                                                "
                                                class="btn btn-xs btn-primary mt-2"
                                                :disabled="!enableCustomCSS"
                                            >
                                                <ClipboardPaste
                                                    class="w-4 h-4"
                                                />
                                                {{ t("theme.insert_example") }}
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </transition>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, toRefs, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { emit as emitAppEvent, listen } from "@tauri-apps/api/event";
import { useI18n } from "vue-i18n";
import {
    ClipboardCopy,
    ClipboardPaste,
    Palette,
    Save,
    Store,
    SunMoon,
    ExternalLink,
    Clock,
} from "lucide-vue-next";
import { useToast } from "@shared/composables/useToast";
import { settingsService } from "@services/settings/settingsService";
import { themeService } from "@services/theme/themeService";
import PresetManager from "@features/presets/components/PresetManager.vue";
import {
    Moon,
    Sun,
    RotateCcw,
    Code,
    HelpCircle,
    ChevronDown,
    ChevronUp,
} from "lucide-vue-next";
import { VueMonacoEditor } from "@guolao/vue-monaco-editor";
import ImportExportCssModal from "@features/presets/modals/ImportExportCssModal.vue";
import { useModal } from "@shared/composables/useModal";
import { themeScheduler } from "@services/theme/themeScheduler";

defineEmits(["change-view"]);

const i18n = useI18n();
const { t } = i18n;
const { addToast } = useToast();
const { showModal } = useModal();

const themes = ["dark", "light"];
const selectedTheme = ref(
    document.documentElement.getAttribute("data-theme") || "dark"
);
const showExpertOptions = ref(false);
const expertAnimationActive = ref(false);

type ThemeMode = "dark" | "light" | "schedule";

const _getInitialThemeMode = (): ThemeMode => {
    if (themeScheduler.schedule.value.enabled) return "schedule";
    return (document.documentElement.getAttribute("data-theme") as ThemeMode) || "dark";
};

const themeMode = ref<ThemeMode>(_getInitialThemeMode());

const selectThemeMode = async (mode: ThemeMode) => {
    themeMode.value = mode;

    if (mode === "schedule") {
        themeScheduler.updateSchedule({ enabled: true });
        selectedTheme.value = themeScheduler.previewTheme.value;
    } else {
        themeScheduler.updateSchedule({ enabled: false });
        await changeTheme(mode);
    }
};

const scheduleLightStart = computed(
    () => themeScheduler.schedule.value.lightStart
);
const scheduleLightEnd = computed(() => themeScheduler.schedule.value.lightEnd);
const schedulePreviewTheme = themeScheduler.previewTheme;

const updateScheduleTime = (
    field: "lightStart" | "lightEnd",
    value: string
) => {
    themeScheduler.updateSchedule({ [field]: value });
};
const isExternalWindow = window.location.search.includes(
    "window=customization"
);

const {
    customCSS,
    enableCustomCSS,
    primary: primaryColor,
    base100,
    base200,
    base300,
    baseContent,
    primaryContent,
    secondary,
    secondaryContent,
    accent,
    accentContent,
    neutral,
    neutralContent,
    info,
    infoContent,
    success,
    successContent,
    warning,
    warningContent,
    error,
    errorContent,
    backgroundImage,
    backgroundBlur,
    backgroundOpacity,
} = toRefs(themeService.presetSettings);

watch(
    themeService.presetSettings,
    () => {
        themeService.saveCardSettings();
    },
    { deep: true }
);

const cssExamples = [
    {
        title: t("theme.example_1"),
        code: `.client-card {
  backdrop-filter: blur(5px);
  background-color: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
}`,
    },
];

const _colorRefs: Record<string, any> = {
    base100,
    base200,
    base300,
    baseContent,
    primaryColorOverride: primaryColor,
    primaryContent,
    secondary,
    secondaryContent,
    accent,
    accentContent,
    neutral,
    neutralContent,
    info,
    infoContent,
    success,
    successContent,
    warning,
    warningContent,
    error,
    errorContent,
};

const handleColorInput = (settingKey: string, color: string): void => {
    const r = _colorRefs[settingKey];
    if (r) {
        r.value = color && color.trim().length > 0 ? color : null;
    }
};

const handleBackgroundInput = (settingKey: string, value: any): void => {
    const refs: Record<string, any> = {
        backgroundImage,
        backgroundBlur,
        backgroundOpacity,
    };
    const r = refs[settingKey];
    if (r) {
        if (settingKey === "backgroundImage") {
            r.value = value && value.trim().length > 0 ? value.trim() : null;
        } else {
            r.value = value;
        }
    }
};

const changeTheme = async (theme: string) => {
    try {
        selectedTheme.value = theme;
        document.documentElement.setAttribute("data-theme", theme);
        localStorage.setItem("theme", theme);
        await invoke("set_window_theme", { theme });

        await settingsService.editSetting("theme", theme, false);
        await emitAppEvent("theme-mode-update", theme);

        addToast(t("theme.change_success"), "success");
    } catch (error) {
        console.error("Failed to save theme:", error);
        addToast(t("theme.save_failed", { error }), "error");
    }
};

const toggleExpertMode = () => {
    expertAnimationActive.value = true;
    showExpertOptions.value = !showExpertOptions.value;

    setTimeout(() => {
        expertAnimationActive.value = false;
    }, 300);
};

const handleEnableCustomCSS = (val: boolean) => {
    enableCustomCSS.value = val;
};

const insertExample = (code: string) => {
    if (!enableCustomCSS.value) {
        addToast(t("theme.enable_custom_css_first"), "warning");
        return;
    }

    customCSS.value = customCSS.value
        ? `${customCSS.value.trim()}\n\n${code}`
        : code;

    addToast(t("theme.example_inserted"), "success");
};

const addExample = (className: string) => {
    if (!enableCustomCSS.value) {
        addToast(t("theme.enable_custom_css_first"), "warning");
        return;
    }

    const exampleCode = `${className} {\n  \n}`;
    customCSS.value = customCSS.value
        ? `${customCSS.value.trim()}\n\n${exampleCode}`
        : exampleCode;
};

const handleKeyDown = (event: KeyboardEvent) => {
    if ((event.ctrlKey || event.metaKey) && event.key === "s") {
        event.preventDefault();
        themeService.saveCardSettings();
    }
};

const resetStyles = () => {
    themeService.resetPresetSettings();
};

const openInNewWindow = async () => {
    try {
        const { WebviewWindow } = await import("@tauri-apps/api/webviewWindow");
        const webview = new WebviewWindow("customization-inspector", {
            url: "index.html?window=customization",
            title: "Theme Inspector",
            width: 1000,
            height: 800,
            resizable: true,
            decorations: true,
        });

        webview.once("tauri://created", function () {
            console.log("Customization window created");
        });

        webview.once("tauri://error", function (e) {
            console.error("Error creating window:", e);
        });
    } catch (e) {
        console.error("Failed to open new window:", e);
    }
};

const openExportModal = async () => {
    showModal(
        "export-css",
        ImportExportCssModal,
        { title: t("theme.export_css_title") },
        { mode: "export", css: customCSS.value },
        {}
    );
};

const openImportModal = () => {
    showModal(
        "import-css",
        ImportExportCssModal,
        { title: t("theme.import_css_title") },
        { mode: "import" },
        {
            import: (css: string) => {
                if (
                    /script|@import|url\(|expression|<|>|javascript:/i.test(css)
                ) {
                    addToast(t("theme.import_invalid"), "error");
                    return;
                }
                customCSS.value = css;
            },
        }
    );
};

onMounted(() => {
    document.addEventListener("keydown", handleKeyDown);
    listen<string>("theme-mode-update", (event) => {
        if (event.payload) {
            selectedTheme.value = event.payload;
            if (!themeScheduler.schedule.value.enabled) {
                themeMode.value = event.payload as "dark" | "light";
            }
        }
    }).then((unlisten) => {
        _unlistenThemeMode = unlisten;
    });
});

let _unlistenThemeMode: (() => void) | null = null;

onUnmounted(() => {
    document.removeEventListener("keydown", handleKeyDown);
    _unlistenThemeMode?.();
});
</script>

<style scoped>
.animate-fadeInUp {
    animation: fadeInUp 0.5s ease-out forwards;
    opacity: 0;
}

@keyframes fadeInUp {
    from {
        opacity: 0;
        transform: translateY(10px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.settings-card {
    opacity: 0;
    transform: translateY(10px);
    animation: fadeInUp 0.4s ease-out forwards;
}

.expert-fade-enter-active,
.expert-fade-leave-active {
    transition:
        opacity 0.3s cubic-bezier(0.4, 0, 0.2, 1),
        transform 0.3s cubic-bezier(0.4, 0, 0.2, 1),
        max-height 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    overflow: hidden;
    max-height: 2000px;
}

.expert-fade-enter-from,
.expert-fade-leave-to {
    opacity: 0;
    transform: translateY(12px) scale(0.98);
    max-height: 0;
}

textarea.textarea-bordered {
    font-family: "Fira Code", "Menlo", "Monaco", "Courier New", monospace;
    line-height: 1.5;
    tab-size: 2;
}
</style>

<style scoped>
.schedule-expand-enter-active,
.schedule-expand-leave-active {
    transition:
        opacity 0.25s ease,
        transform 0.25s ease,
        max-height 0.3s ease;
    overflow: hidden;
    max-height: 300px;
}

.schedule-expand-enter-from,
.schedule-expand-leave-to {
    opacity: 0;
    transform: translateY(-6px);
    max-height: 0;
}

.schedule-slide-enter-active {
    transition:
        opacity 0.3s cubic-bezier(0.16, 1, 0.3, 1),
        transform 0.35s cubic-bezier(0.16, 1, 0.3, 1);
}

.schedule-slide-leave-active {
    transition: none;
}

.schedule-slide-enter-from {
    opacity: 0;
    transform: translateX(20px) scale(0.97);
}

.schedule-slide-leave-to {
    opacity: 0;
    transform: none;
}


</style>
