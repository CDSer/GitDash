<!--
  海绵方块吉祥物（致敬海绵宝宝配色，原创造型）
  纯 SVG，支持 size prop 与 mood 切换
-->
<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 64 64"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="mascot mascot--sponge"
  >
    <!-- 身体（带洞洞的方块） -->
    <rect x="12" y="10" width="40" height="40" rx="6" fill="#F7D046" stroke="#1a1a1a" stroke-width="1.5" />
    <!-- 洞洞 -->
    <circle cx="22" cy="20" r="3.5" fill="#E8B830" />
    <circle cx="36" cy="18" r="2.8" fill="#E8B830" />
    <circle cx="44" cy="26" r="3.2" fill="#E8B830" />
    <circle cx="20" cy="34" r="2.5" fill="#E8B830" />
    <circle cx="42" cy="40" r="3" fill="#E8B830" />
    <circle cx="28" cy="44" r="2.2" fill="#E8B830" />
    <!-- 衬衫（白色底 + 领带） -->
    <rect x="12" y="42" width="40" height="8" rx="0" fill="#F5F5F5" stroke="#1a1a1a" stroke-width="1" />
    <path d="M30 42 L32 46 L34 42 Z" fill="#E85D5D" />
    <rect x="30" y="46" width="4" height="4" rx="1" fill="#E85D5D" />
    <!-- 裤子 -->
    <rect x="12" y="50" width="40" height="4" rx="0" fill="#8B6914" stroke="#1a1a1a" stroke-width="1" />
    <!-- 眼睛 -->
    <template v-if="mood === 'happy' || mood === 'celebrating'">
      <path d="M20 28 Q24 24 28 28" stroke="#1a1a1a" stroke-width="1.8" fill="none" stroke-linecap="round" />
      <path d="M36 28 Q40 24 44 28" stroke="#1a1a1a" stroke-width="1.8" fill="none" stroke-linecap="round" />
    </template>
    <template v-else>
      <circle cx="24" cy="28" r="5" fill="#fff" stroke="#1a1a1a" stroke-width="1.3" />
      <circle cx="40" cy="28" r="5" fill="#fff" stroke="#1a1a1a" stroke-width="1.3" />
      <circle cx="24" cy="28" r="2.2" fill="#3B82F6" />
      <circle cx="40" cy="28" r="2.2" fill="#3B82F6" />
      <circle cx="25" cy="27.2" r="0.8" fill="#fff" />
      <circle cx="41" cy="27.2" r="0.8" fill="#fff" />
    </template>
    <!-- 鼻子 -->
    <ellipse cx="32" cy="33" rx="2.5" ry="1.8" fill="#F7D046" stroke="#1a1a1a" stroke-width="1.2" />
    <!-- 嘴 -->
    <path
      v-if="mood === 'happy' || mood === 'celebrating'"
      d="M22 38 Q32 46 42 38"
      stroke="#1a1a1a"
      stroke-width="1.5"
      fill="none"
      stroke-linecap="round"
    />
    <path
      v-else-if="mood === 'worried'"
      d="M24 40 Q32 36 40 40"
      stroke="#1a1a1a"
      stroke-width="1.5"
      fill="none"
      stroke-linecap="round"
    />
    <path
      v-else
      d="M24 38 Q32 42 40 38"
      stroke="#1a1a1a"
      stroke-width="1.5"
      fill="none"
      stroke-linecap="round"
    />
    <!-- 牙齿（开心时露出） -->
    <template v-if="mood === 'happy' || mood === 'celebrating'">
      <rect x="28" y="38" width="3" height="3" rx="0.5" fill="#fff" stroke="#1a1a1a" stroke-width="0.8" />
      <rect x="32" y="38" width="3" height="3" rx="0.5" fill="#fff" stroke="#1a1a1a" stroke-width="0.8" />
    </template>
    <!-- 手臂 -->
    <line x1="12" y1="40" x2="4" y2="36" stroke="#1a1a1a" stroke-width="2" stroke-linecap="round" />
    <line x1="52" y1="40" x2="60" y2="36" stroke="#1a1a1a" stroke-width="2" stroke-linecap="round" />
    <!-- 手 -->
    <circle cx="4" cy="36" r="2.5" fill="#F7D046" stroke="#1a1a1a" stroke-width="1.2" />
    <circle cx="60" cy="36" r="2.5" fill="#F7D046" stroke="#1a1a1a" stroke-width="1.2" />
    <!-- 腿 -->
    <line x1="24" y1="54" x2="24" y2="60" stroke="#1a1a1a" stroke-width="2" stroke-linecap="round" />
    <line x1="40" y1="54" x2="40" y2="60" stroke="#1a1a1a" stroke-width="2" stroke-linecap="round" />
    <!-- 鞋 -->
    <ellipse cx="22" cy="61" rx="4" ry="2" fill="#1a1a1a" />
    <ellipse cx="42" cy="61" rx="4" ry="2" fill="#1a1a1a" />
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
.mascot--sponge {
  animation: mascot-sway 3.5s ease-in-out infinite;
}
@keyframes mascot-sway {
  0%, 100% { transform: rotate(-1.5deg); }
  50% { transform: rotate(1.5deg); }
}
</style>
