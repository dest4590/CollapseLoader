<template>
    <IconBase v-bind="$props" viewBox="0 0 48 48">
        <path
            d="M40.83,8.48c1.14,0,2,1,1.54,2.86l-5.58,26.3c-.39,1.87-1.52,2.32-3.08,1.45L20.4,29.26a.4.4,0,0,1,0-.65L35.77,14.73c.7-.62-.15-.92-1.07-.36L15.41,26.54a.46.46,0,0,1-.4.05L6.82,24C5,23.47,5,22.22,7.23,21.33L40,8.69a2.16,2.16,0,0,1,.83-.21Z" />
    </IconBase>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import IconBase from './IconBase.vue'

export default defineComponent({
    name: 'TelegramIcon',
    components: { IconBase },
    inheritAttrs: false,
    props: {
        size: { type: [Number, String], default: 24 },
        width: { type: [Number, String], default: null },
        height: { type: [Number, String], default: null },
        fill: { type: String, default: 'themed' },
        ariaLabel: { type: String, default: 'Telegram' },
        svgClass: { type: [String, Object, Array], default: null },
    },
})
</script>
