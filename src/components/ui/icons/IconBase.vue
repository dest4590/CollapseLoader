<template>
  <svg :width="computedWidth" :height="computedHeight" :viewBox="viewBox" :fill="computedFill" :class="svgClass"
    xmlns="http://www.w3.org/2000/svg" :role="role" :aria-label="ariaLabel">
    <slot />
  </svg>
</template>

<script lang="ts">
import { defineComponent, computed, ref, onMounted, onUnmounted } from 'vue'

export default defineComponent({
  name: 'IconBase',
  props: {
    size: { type: [Number, String], default: 24 },
    width: { type: [Number, String], default: null },
    height: { type: [Number, String], default: null },
    viewBox: { type: String, default: '0 0 24 24' },
    fill: { type: String, default: 'currentColor' },
    ariaLabel: { type: String, default: null },
    role: { type: String, default: 'img' },
    svgClass: { type: [String, Object, Array], default: null },
  },
  setup(props) {
    const computedWidth = computed(() => props.width ?? props.size)
    const computedHeight = computed(() => props.height ?? props.size)

    const theme = ref(document.documentElement.getAttribute('data-theme') || 'dark')

    let observer: MutationObserver | null = null
    onMounted(() => {
      observer = new MutationObserver(() => {
        const t = document.documentElement.getAttribute('data-theme') || 'dark'
        if (theme.value !== t) theme.value = t
      })
      observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] })
    })

    onUnmounted(() => {
      if (observer) observer.disconnect()
    })

    const computedFill = computed(() => {
      if (props.fill === 'themed') {
        return theme.value === 'light' ? '#000000' : '#ffffff'
      }
      return props.fill
    })

    return { computedWidth, computedHeight, computedFill }
  },
})
</script>
