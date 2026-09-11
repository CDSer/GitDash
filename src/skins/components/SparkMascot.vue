<!--
  电气鼠吉祥物（致敬皮卡丘配色，原创造型）
  纯 SVG，支持 size prop 与 mood 切换
-->
<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 64 64"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="mascot mascot--spark"
  >
    <!-- 尾巴（闪电形） -->
    <path
      d="M48 36 L56 28 L52 28 L58 20 L50 26 L54 26 L46 34 Z"
      fill="#F5C518"
      stroke="#1a1a1a"
      stroke-width="1.5"
      stroke-linejoin="round"
    />
    <!-- 身体 -->
    <ellipse cx="32" cy="40" rx="16" ry="14" fill="#F5C518" stroke="#1a1a1a" stroke-width="1.5" />
    <!-- 肚皮 -->
    <ellipse cx="32" cy="44" rx="8" ry="6" fill="#FFF8DC" opacity="0.7" />
    <!-- 左耳 -->
    <path d="M18 18 L14 6 L22 16 Z" fill="#F5C518" stroke="#1a1a1a" stroke-width="1.5" stroke-linejoin="round" />
    <path d="M16 10 L14 6 L19 13 Z" fill="#1a1a1a" />
    <!-- 右耳 -->
    <path d="M46 18 L50 6 L42 16 Z" fill="#F5C518" stroke="#1a1a1a" stroke-width="1.5" stroke-linejoin="round" />
    <path d="M48 10 L50 6 L45 13 Z" fill="#1a1a1a" />
    <!-- 腮红 -->
    <circle cx="18" cy="40" r="4" fill="#E85D5D" opacity="0.85" />
    <circle cx="46" cy="40" r="4" fill="#E85D5D" opacity="0.85" />
    <!-- 眼睛 -->
    <template v-if="mood === 'happy' || mood === 'celebrating'">
      <path d="M24 33 Q26 30 28 33" stroke="#1a1a1a" stroke-width="1.8" fill="none" stroke-linecap="round" />
      <path d="M36 33 Q38 30 40 33" stroke="#1a1a1a" stroke-width="1.8" fill="none" stroke-linecap="round" />
    </template>
    <template v-else>
      <circle cx="26" cy="33" r="2.2" fill="#1a1a1a" />
      <circle cx="38" cy="33" r="2.2" fill="#1a1a1a" />
      <circle cx="27" cy="32.2" r="0.7" fill="#fff" />
      <circle cx="39" cy="32.2" r="0.7" fill="#fff" />
    </template>
    <!-- 嘴 -->
    <path
      v-if="mood === 'happy' || mood === 'celebrating'"
      d="M28 38 Q32 42 36 38"
      stroke="#1a1a1a"
      stroke-width="1.5"
      fill="none"
      stroke-linecap="round"
    />
    <path
      v-else-if="mood === 'worried'"
      d="M28 40 Q32 37 36 40"
      stroke="#1a1a1a"
      stroke-width="1.5"
      fill="none"
      stroke-linecap="round"
    />
    <path
      v-else
      d="M30 38 Q32 40 34 38"
      stroke="#1a1a1a"
      stroke-width="1.5"
      fill="none"
      stroke-linecap="round"
    />
    <!-- 手臂 -->
    <ellipse cx="16" cy="42" rx="4" ry="3" fill="#F5C518" stroke="#1a1a1a" stroke-width="1.3" />
    <ellipse cx="48" cy="42" rx="4" ry="3" fill="#F5C518" stroke="#1a1a1a" stroke-width="1.3" />
    <!-- 脚 -->
    <ellipse cx="24" cy="52" rx="5" ry="3" fill="#F5C518" stroke="#1a1a1a" stroke-width="1.3" />
    <ellipse cx="40" cy="52" rx="5" ry="3" fill="#F5C518" stroke="#1a1a1a" stroke-width="1.3" />
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
.mascot--spark {
  animation: mascot-bob 3s ease-in-out infinite;
}
@keyframes mascot-bob {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-2px); }
}
</style>
