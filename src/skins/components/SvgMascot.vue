<!--
  用户皮肤包 SVG 吉祥物渲染器
  将 mascot_svg / decoration.svg 原始字符串渲染为内联 SVG
-->
<template>
  <div
    v-if="svg"
    class="svg-mascot"
    :style="{ width: size + 'px', height: size + 'px' }"
    v-html="svg"
  />
</template>

<script setup lang="ts">
withDefaults(defineProps<{ svg?: string | null; size?: number }>(), {
  svg: null,
  size: 32,
});
</script>

<style scoped>
.svg-mascot {
  display: block;
  flex-shrink: 0;
}
.svg-mascot :deep(svg) {
  width: 100%;
  height: 100%;
  display: block;
}
</style>
