<!--
  橘猫吉祥物（暖橙轻拟物风格，原创造型）
  纯 SVG，支持 size prop 与 mood 切换
-->
<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 64 64"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="mascot mascot--cat"
  >
    <!-- 尾巴（卷尾，可摇动） -->
    <path
      d="M46 42 Q58 46 57 33 Q56 24 47 26"
      stroke="#4A3223"
      stroke-width="7"
      fill="none"
      stroke-linecap="round"
      class="cat-tail cat-tail--outline"
    />
    <path
      d="M46 42 Q58 46 57 33 Q56 24 47 26"
      stroke="#F2994A"
      stroke-width="4.5"
      fill="none"
      stroke-linecap="round"
      class="cat-tail"
    />

    <!-- 身体 -->
    <ellipse cx="32" cy="42" rx="16" ry="14" fill="#F2994A" stroke="#4A3223" stroke-width="1.5" />
    <!-- 肚皮 -->
    <ellipse cx="32" cy="47" rx="9" ry="7" fill="#FFF3E0" opacity="0.9" />

    <!-- 左耳 -->
    <path d="M21 22 L15 6 L27 15 Z" fill="#F2994A" stroke="#4A3223" stroke-width="1.5" stroke-linejoin="round" />
    <path d="M19 13 L17 8 L24 15 Z" fill="#FFB9C5" />
    <!-- 右耳 -->
    <path d="M43 22 L49 6 L34 15 Z" fill="#F2994A" stroke="#4A3223" stroke-width="1.5" stroke-linejoin="round" />
    <path d="M45 13 L47 8 L40 15 Z" fill="#FFB9C5" />

    <!-- 头顶条纹（橘猫花纹） -->
    <path d="M26 20 L28 25" stroke="#D9772B" stroke-width="2" stroke-linecap="round" />
    <path d="M32 19 L32 25" stroke="#D9772B" stroke-width="2" stroke-linecap="round" />
    <path d="M38 20 L36 25" stroke="#D9772B" stroke-width="2" stroke-linecap="round" />

    <!-- 腮红 -->
    <circle cx="17" cy="32" r="3.6" fill="#F9A8D4" opacity="0.75" />
    <circle cx="47" cy="32" r="3.6" fill="#F9A8D4" opacity="0.75" />

    <!-- 眼睛 -->
    <template v-if="mood === 'happy' || mood === 'celebrating'">
      <path d="M23 30 Q26 26 29 30" stroke="#4A3223" stroke-width="1.8" fill="none" stroke-linecap="round" />
      <path d="M35 30 Q38 26 41 30" stroke="#4A3223" stroke-width="1.8" fill="none" stroke-linecap="round" />
    </template>
    <template v-else>
      <circle cx="26" cy="30" r="2.4" fill="#4A3223" />
      <circle cx="38" cy="30" r="2.4" fill="#4A3223" />
      <circle cx="27" cy="29.2" r="0.8" fill="#fff" />
      <circle cx="39" cy="29.2" r="0.8" fill="#fff" />
    </template>

    <!-- 鼻子 -->
    <ellipse cx="32" cy="34" rx="2" ry="1.6" fill="#E8807A" />

    <!-- 嘴 -->
    <path
      v-if="mood === 'happy' || mood === 'celebrating'"
      d="M26 37 Q28 40 32 37 Q36 40 38 37"
      stroke="#4A3223"
      stroke-width="1.5"
      fill="none"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      v-else-if="mood === 'worried'"
      d="M28 39 Q32 36.5 36 39"
      stroke="#4A3223"
      stroke-width="1.5"
      fill="none"
      stroke-linecap="round"
    />
    <path
      v-else
      d="M29.5 38 Q32 39.5 34.5 38"
      stroke="#4A3223"
      stroke-width="1.5"
      fill="none"
      stroke-linecap="round"
    />

    <!-- 胡须 -->
    <g stroke="#4A3223" stroke-width="1.2" stroke-linecap="round" opacity="0.55">
      <path d="M10 30 L18 32" />
      <path d="M10 35 L18 35" />
      <path d="M11 40 L18 38" />
      <path d="M54 30 L46 32" />
      <path d="M54 35 L46 35" />
      <path d="M53 40 L46 38" />
    </g>

    <!-- 前爪 -->
    <ellipse cx="22" cy="50" rx="5.5" ry="4" fill="#F2994A" stroke="#4A3223" stroke-width="1.3" />
    <ellipse cx="42" cy="50" rx="5.5" ry="4" fill="#F2994A" stroke="#4A3223" stroke-width="1.3" />
    <!-- 后脚 -->
    <ellipse cx="26" cy="53.5" rx="4.5" ry="3" fill="#F2994A" stroke="#4A3223" stroke-width="1.3" />
    <ellipse cx="38" cy="53.5" rx="4.5" ry="3" fill="#F2994A" stroke="#4A3223" stroke-width="1.3" />
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
.mascot--cat {
  animation: mascot-bob 3.2s ease-in-out infinite;
}
.cat-tail {
  transform-origin: 46px 48px;
  animation: tail-wag 2.6s ease-in-out infinite;
}
.cat-tail-outline {
  animation: tail-wag 2.6s ease-in-out infinite;
}
@keyframes mascot-bob {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-2px); }
}
@keyframes tail-wag {
  0%, 100% { transform: rotate(0deg); }
  50% { transform: rotate(-5deg); }
}
</style>