<!--
  太空船员装饰插画：空状态 / 全部已处理
-->
<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 120 120"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="deco deco--crew"
  >
    <!-- 背包 -->
    <rect x="82" y="40" width="18" height="36" rx="8" fill="#0E9AA7" stroke="#1a1a1a" stroke-width="2" />

    <!-- 身体 -->
    <path
      d="M34 28 Q34 14 60 14 Q86 14 86 28 L86 92 Q86 106 60 106 Q34 106 34 92 Z"
      fill="#0E9AA7"
      stroke="#1a1a1a"
      stroke-width="2"
    />

    <!-- 面罩 -->
    <rect x="42" y="32" width="40" height="22" rx="11" fill="#B8E8F0" stroke="#1a1a1a" stroke-width="1.8" />
    <template v-if="kind === 'allClear'">
      <path d="M50 42 Q60 36 70 42" stroke="#0E9AA7" stroke-width="2.2" fill="none" stroke-linecap="round" />
    </template>
    <template v-else>
      <rect x="46" y="36" width="14" height="7" rx="3.5" fill="#fff" opacity="0.6" />
    </template>

    <!-- 腿 -->
    <rect x="44" y="104" width="12" height="12" rx="4" fill="#0E9AA7" stroke="#1a1a1a" stroke-width="1.8" />
    <rect x="64" y="104" width="12" height="12" rx="4" fill="#0E9AA7" stroke="#1a1a1a" stroke-width="1.8" />

    <!-- 地面阴影 -->
    <ellipse cx="60" cy="118" rx="26" ry="3" fill="#1a1a1a" opacity="0.1" />

    <!-- allClear 装饰：星星 -->
    <g v-if="kind === 'allClear'" class="deco-stars">
      <path d="M16 36 L18 32 L20 36 L24 38 L20 40 L18 44 L16 40 L12 38 Z" fill="#F5C518" />
      <path d="M102 50 L104 46 L106 50 L110 52 L106 54 L104 58 L102 54 L98 52 Z" fill="#F5C518" />
      <circle cx="20" cy="60" r="2" fill="#F5C518" opacity="0.7" />
      <circle cx="100" cy="70" r="2" fill="#F5C518" opacity="0.7" />
    </g>
  </svg>
</template>

<script setup lang="ts">
withDefaults(defineProps<{ size?: number; kind?: string }>(), {
  size: 80,
  kind: 'empty',
});
</script>

<style scoped>
.deco {
  display: block;
  margin: 0 auto;
}
.deco--crew {
  animation: deco-float 4s ease-in-out infinite;
}
.deco-stars {
  animation: deco-twinkle 2s ease-in-out infinite;
}
@keyframes deco-float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-6px); }
}
@keyframes deco-twinkle {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}
</style>
