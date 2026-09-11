<!--
  电气鼠装饰插画：空状态（无项目）/ 全部已处理
  大尺寸 SVG，带轻微动画
-->
<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 120 120"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="deco deco--spark"
  >
    <!-- 地面阴影 -->
    <ellipse cx="60" cy="108" rx="28" ry="5" fill="#1a1a1a" opacity="0.1" />

    <!-- 身体 -->
    <ellipse cx="60" cy="72" rx="30" ry="26" fill="#F5C518" stroke="#1a1a1a" stroke-width="2" />
    <ellipse cx="60" cy="80" rx="16" ry="12" fill="#FFF8DC" opacity="0.7" />

    <!-- 尾巴（闪电） -->
    <path
      v-if="kind === 'allClear'"
      d="M88 68 L102 52 L94 52 L106 36 L92 50 L100 50 L86 64 Z"
      fill="#F5C518"
      stroke="#1a1a1a"
      stroke-width="2"
      stroke-linejoin="round"
      class="deco-spark-bolt"
    />
    <path
      v-else
      d="M88 68 L96 58 L92 58 L100 46 L92 56 L96 56 L86 64 Z"
      fill="#F5C518"
      stroke="#1a1a1a"
      stroke-width="2"
      stroke-linejoin="round"
      opacity="0.6"
    />

    <!-- 耳朵 -->
    <path d="M36 38 L28 14 L44 34 Z" fill="#F5C518" stroke="#1a1a1a" stroke-width="2" stroke-linejoin="round" />
    <path d="M32 22 L28 14 L38 28 Z" fill="#1a1a1a" />
    <path d="M84 38 L92 14 L76 34 Z" fill="#F5C518" stroke="#1a1a1a" stroke-width="2" stroke-linejoin="round" />
    <path d="M88 22 L92 14 L82 28 Z" fill="#1a1a1a" />

    <!-- 腮红 -->
    <circle cx="36" cy="72" r="7" fill="#E85D5D" opacity="0.85" />
    <circle cx="84" cy="72" r="7" fill="#E85D5D" opacity="0.85" />

    <!-- 眼睛 -->
    <template v-if="kind === 'allClear'">
      <path d="M46 62 Q50 56 54 62" stroke="#1a1a1a" stroke-width="2.5" fill="none" stroke-linecap="round" />
      <path d="M66 62 Q70 56 74 62" stroke="#1a1a1a" stroke-width="2.5" fill="none" stroke-linecap="round" />
    </template>
    <template v-else>
      <circle cx="50" cy="62" r="4" fill="#1a1a1a" />
      <circle cx="70" cy="62" r="4" fill="#1a1a1a" />
      <circle cx="51.5" cy="60.5" r="1.3" fill="#fff" />
      <circle cx="71.5" cy="60.5" r="1.3" fill="#fff" />
    </template>

    <!-- 嘴 -->
    <path
      v-if="kind === 'allClear'"
      d="M52 72 Q60 80 68 72"
      stroke="#1a1a1a"
      stroke-width="2"
      fill="none"
      stroke-linecap="round"
    />
    <path
      v-else
      d="M56 72 Q60 75 64 72"
      stroke="#1a1a1a"
      stroke-width="2"
      fill="none"
      stroke-linecap="round"
    />

    <!-- 手臂 -->
    <ellipse cx="30" cy="76" rx="8" ry="6" fill="#F5C518" stroke="#1a1a1a" stroke-width="1.8" />
    <ellipse cx="90" cy="76" rx="8" ry="6" fill="#F5C518" stroke="#1a1a1a" stroke-width="1.8" />

    <!-- 脚 -->
    <ellipse cx="44" cy="96" rx="10" ry="6" fill="#F5C518" stroke="#1a1a1a" stroke-width="1.8" />
    <ellipse cx="76" cy="96" rx="10" ry="6" fill="#F5C518" stroke="#1a1a1a" stroke-width="1.8" />

    <!-- 电火花装饰（allClear 时） -->
    <g v-if="kind === 'allClear'" class="deco-sparks">
      <path d="M18 40 L22 34 L20 34 L24 28" stroke="#F5C518" stroke-width="2" fill="none" stroke-linecap="round" />
      <path d="M100 40 L96 34 L98 34 L94 28" stroke="#F5C518" stroke-width="2" fill="none" stroke-linecap="round" />
      <circle cx="24" cy="50" r="2" fill="#F5C518" opacity="0.6" />
      <circle cx="96" cy="50" r="2" fill="#F5C518" opacity="0.6" />
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
.deco--spark {
  animation: deco-bob 3s ease-in-out infinite;
}
.deco-spark-bolt {
  animation: deco-bolt 1.5s ease-in-out infinite;
}
.deco-sparks {
  animation: deco-twinkle 2s ease-in-out infinite;
}
@keyframes deco-bob {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-4px); }
}
@keyframes deco-bolt {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}
@keyframes deco-twinkle {
  0%, 100% { opacity: 0.6; }
  50% { opacity: 1; }
}
</style>
