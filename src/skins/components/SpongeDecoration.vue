<!--
  海绵方块装饰插画：空状态 / 全部已处理
-->
<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 120 120"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="deco deco--sponge"
  >
    <!-- 身体 -->
    <rect x="20" y="16" width="80" height="76" rx="10" fill="#F7D046" stroke="#1a1a1a" stroke-width="2" />
    <!-- 洞洞 -->
    <circle cx="38" cy="32" r="6" fill="#E8B830" />
    <circle cx="64" cy="28" r="5" fill="#E8B830" />
    <circle cx="84" cy="42" r="6" fill="#E8B830" />
    <circle cx="34" cy="58" r="5" fill="#E8B830" />
    <circle cx="80" cy="70" r="5.5" fill="#E8B830" />
    <circle cx="50" cy="78" r="4.5" fill="#E8B830" />

    <!-- 衬衫 -->
    <rect x="20" y="82" width="80" height="14" fill="#F5F5F5" stroke="#1a1a1a" stroke-width="1.5" />
    <path d="M56 82 L60 88 L64 82 Z" fill="#E85D5D" />
    <rect x="56" y="88" width="8" height="8" rx="2" fill="#E85D5D" />

    <!-- 裤子 -->
    <rect x="20" y="96" width="80" height="8" fill="#8B6914" stroke="#1a1a1a" stroke-width="1.5" />

    <!-- 眼睛 -->
    <template v-if="kind === 'allClear'">
      <path d="M40 48 Q46 40 52 48" stroke="#1a1a1a" stroke-width="2.5" fill="none" stroke-linecap="round" />
      <path d="M68 48 Q74 40 80 48" stroke="#1a1a1a" stroke-width="2.5" fill="none" stroke-linecap="round" />
    </template>
    <template v-else>
      <circle cx="46" cy="48" r="9" fill="#fff" stroke="#1a1a1a" stroke-width="1.8" />
      <circle cx="74" cy="48" r="9" fill="#fff" stroke="#1a1a1a" stroke-width="1.8" />
      <circle cx="46" cy="48" r="4" fill="#3B82F6" />
      <circle cx="74" cy="48" r="4" fill="#3B82F6" />
      <circle cx="47.5" cy="46.5" r="1.5" fill="#fff" />
      <circle cx="75.5" cy="46.5" r="1.5" fill="#fff" />
    </template>

    <!-- 鼻子 -->
    <ellipse cx="60" cy="58" rx="4.5" ry="3.5" fill="#F7D046" stroke="#1a1a1a" stroke-width="1.5" />

    <!-- 嘴 -->
    <path
      v-if="kind === 'allClear'"
      d="M42 68 Q60 82 78 68"
      stroke="#1a1a1a"
      stroke-width="2"
      fill="none"
      stroke-linecap="round"
    />
    <path
      v-else
      d="M44 68 Q60 74 76 68"
      stroke="#1a1a1a"
      stroke-width="2"
      fill="none"
      stroke-linecap="round"
    />

    <!-- 牙齿（allClear 露出） -->
    <template v-if="kind === 'allClear'">
      <rect x="52" y="68" width="6" height="6" rx="1" fill="#fff" stroke="#1a1a1a" stroke-width="1" />
      <rect x="60" y="68" width="6" height="6" rx="1" fill="#fff" stroke="#1a1a1a" stroke-width="1" />
    </template>

    <!-- 手臂 -->
    <line x1="20" y1="72" x2="8" y2="64" stroke="#1a1a1a" stroke-width="3" stroke-linecap="round" />
    <line x1="100" y1="72" x2="112" y2="64" stroke="#1a1a1a" stroke-width="3" stroke-linecap="round" />
    <circle cx="8" cy="64" r="5" fill="#F7D046" stroke="#1a1a1a" stroke-width="1.5" />
    <circle cx="112" cy="64" r="5" fill="#F7D046" stroke="#1a1a1a" stroke-width="1.5" />

    <!-- 腿 -->
    <line x1="44" y1="104" x2="44" y2="112" stroke="#1a1a1a" stroke-width="3" stroke-linecap="round" />
    <line x1="76" y1="104" x2="76" y2="112" stroke="#1a1a1a" stroke-width="3" stroke-linecap="round" />
    <ellipse cx="40" cy="114" rx="8" ry="4" fill="#1a1a1a" />
    <ellipse cx="80" cy="114" rx="8" ry="4" fill="#1a1a1a" />
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
.deco--sponge {
  animation: deco-sway 3.5s ease-in-out infinite;
}
@keyframes deco-sway {
  0%, 100% { transform: rotate(-2deg); }
  50% { transform: rotate(2deg); }
}
</style>
