<!--
  太空船员吉祥物（致敬 Among Us 配色，原创造型）
  纯 SVG，支持 size prop 与 mood 切换
-->
<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 64 64"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="mascot mascot--crew"
  >
    <!-- 背包 -->
    <rect x="44" y="24" width="10" height="20" rx="4" fill="#0E9AA7" stroke="#1a1a1a" stroke-width="1.5" />
    <!-- 身体（豆形） -->
    <path
      d="M20 18 Q20 10 32 10 Q44 10 44 18 L44 48 Q44 56 32 56 Q20 56 20 48 Z"
      fill="#0E9AA7"
      stroke="#1a1a1a"
      stroke-width="1.5"
    />
    <!-- 面罩 -->
    <template v-if="mood === 'happy' || mood === 'celebrating'">
      <rect x="24" y="20" width="20" height="12" rx="6" fill="#B8E8F0" stroke="#1a1a1a" stroke-width="1.3" />
      <path d="M28 26 Q32 23 36 26" stroke="#0E9AA7" stroke-width="1.5" fill="none" stroke-linecap="round" />
    </template>
    <template v-else-if="mood === 'worried'">
      <rect x="24" y="20" width="20" height="12" rx="6" fill="#B8E8F0" stroke="#1a1a1a" stroke-width="1.3" />
      <path d="M28 28 Q32 25 36 28" stroke="#0E9AA7" stroke-width="1.5" fill="none" stroke-linecap="round" />
    </template>
    <template v-else>
      <rect x="24" y="20" width="20" height="12" rx="6" fill="#B8E8F0" stroke="#1a1a1a" stroke-width="1.3" />
      <rect x="26" y="22" width="8" height="4" rx="2" fill="#fff" opacity="0.6" />
    </template>
    <!-- 腿 -->
    <rect x="24" y="54" width="6" height="6" rx="2" fill="#0E9AA7" stroke="#1a1a1a" stroke-width="1.3" />
    <rect x="34" y="54" width="6" height="6" rx="2" fill="#0E9AA7" stroke="#1a1a1a" stroke-width="1.3" />
    <!-- 底部阴影 -->
    <ellipse cx="32" cy="62" rx="14" ry="2" fill="#1a1a1a" opacity="0.12" />
  </svg>
</template>

<script setup lang="ts">
withDefaults(defineProps<{ size?: number; mood?: string }>(), {
  size: 32,
  mood: 'idle',
});
</script>

<style scoped>
.mascot {
  display: block;
  flex-shrink: 0;
}
.mascot--crew {
  animation: mascot-float 4s ease-in-out infinite;
}
@keyframes mascot-float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-3px); }
}
</style>
