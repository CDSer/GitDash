<!--
  猫咪装饰插画：空状态（无项目）/ 全部已处理
  慵懒趴猫 + 毛线球 / 星星爪印，大尺寸 SVG，带轻微动画
-->
<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 120 120"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="deco deco--cat"
  >
    <!-- 地面阴影 -->
    <ellipse cx="60" cy="106" rx="36" ry="6" fill="#4A3223" opacity="0.12" />

    <!-- 尾巴（右侧卷尾） -->
    <path
      d="M86 80 Q96 72 88 58 Q84 50 78 52"
      stroke="#4A3223"
      stroke-width="9"
      fill="none"
      stroke-linecap="round"
      class="deco-tail deco-tail-outline"
    />
    <path
      d="M86 80 Q96 72 88 58 Q84 50 78 52"
      stroke="#F2994A"
      stroke-width="6"
      fill="none"
      stroke-linecap="round"
      class="deco-tail"
    />

    <!-- 身体 -->
    <ellipse cx="60" cy="80" rx="30" ry="17" fill="#F2994A" stroke="#4A3223" stroke-width="2" />
    <ellipse cx="60" cy="85" rx="17" ry="9" fill="#FFF3E0" opacity="0.85" />

    <!-- 头 -->
    <ellipse cx="60" cy="58" rx="15" ry="13" fill="#F2994A" stroke="#4A3223" stroke-width="2" />

    <!-- 耳朵 -->
    <path d="M48 32 L39 14 L56 30 Z" fill="#F2994A" stroke="#4A3223" stroke-width="2" stroke-linejoin="round" />
    <path d="M47 24 L42 17 L51 26 Z" fill="#FFB9C5" />
    <path d="M72 32 L81 14 L64 30 Z" fill="#F2994A" stroke="#4A3223" stroke-width="2" stroke-linejoin="round" />
    <path d="M73 24 L78 17 L69 26 Z" fill="#FFB9C5" />

    <!-- 头顶条纹 -->
    <path d="M54 48 L56 52" stroke="#D9772B" stroke-width="2.2" stroke-linecap="round" />
    <path d="M60 47 L60 52" stroke="#D9772B" stroke-width="2.2" stroke-linecap="round" />
    <path d="M66 48 L64 52" stroke="#D9772B" stroke-width="2.2" stroke-linecap="round" />

    <!-- 腮红 -->
    <circle cx="42" cy="62" r="4.5" fill="#F9A8D4" opacity="0.75" />
    <circle cx="78" cy="62" r="4.5" fill="#F9A8D4" opacity="0.75" />

    <!-- 眼睛 -->
    <template v-if="kind === 'allClear'">
      <path d="M48 56 Q52 50 56 56" stroke="#4A3223" stroke-width="2.5" fill="none" stroke-linecap="round" />
      <path d="M64 56 Q68 50 72 56" stroke="#4A3223" stroke-width="2.5" fill="none" stroke-linecap="round" />
    </template>
    <template v-else>
      <path d="M50 58 Q53 54 56 58" stroke="#4A3223" stroke-width="2.5" fill="none" stroke-linecap="round" />
      <path d="M64 58 Q67 54 70 58" stroke="#4A3223" stroke-width="2.5" fill="none" stroke-linecap="round" />
    </template>

    <!-- 嘴 -->
    <path
      v-if="kind === 'allClear'"
      d="M54 64 Q56 67 60 64 Q64 67 66 64"
      stroke="#4A3223"
      stroke-width="2"
      fill="none"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      v-else
      d="M58 65 Q60 66.5 62 65"
      stroke="#4A3223"
      stroke-width="2"
      fill="none"
      stroke-linecap="round"
    />

    <!-- 胡须 -->
    <g stroke="#4A3223" stroke-width="1.6" stroke-linecap="round" opacity="0.5">
      <path d="M26 58 L36 60" />
      <path d="M26 64 L36 63" />
      <path d="M94 58 L84 60" />
      <path d="M94 64 L84 63" />
    </g>

    <!-- 前爪 -->
    <ellipse cx="36" cy="92" rx="10" ry="7" fill="#F2994A" stroke="#4A3223" stroke-width="2" />
    <ellipse cx="84" cy="92" rx="10" ry="7" fill="#F2994A" stroke="#4A3223" stroke-width="2" />

    <!-- 全部完成：举手 + 毛线球 + 星星 -->
    <template v-if="kind === 'allClear'">
      <ellipse cx="14" cy="70" rx="7" ry="5" fill="#F2994A" stroke="#4A3223" stroke-width="2" transform="rotate(-18 14 70)" />
      <circle cx="100" cy="30" r="12" fill="#FFB9C5" />
      <path d="M93 24 Q100 30 107 24" stroke="#FFF3E0" stroke-width="2" fill="none" stroke-linecap="round" />
      <path d="M94 38 Q100 33 106 38" stroke="#FFF3E0" stroke-width="2" fill="none" stroke-linecap="round" />
      <path d="M100 42 Q104 90 96 92" stroke="#4A3223" stroke-width="2" fill="none" stroke-linecap="round" opacity="0.4" />
      <g class="deco-sparkles">
        <path d="M18 26 L20 22 L22 26 L20 30 Z" fill="#F6C453" />
        <path d="M102 62 L103.5 59 105 62 103.5 65 Z" fill="#F6C453" />
        <circle cx="26" cy="40" r="2" fill="#F6C453" opacity="0.7" />
      </g>
    </template>

    <!-- 空状态：脚边毛线团 + 爪印 -->
    <template v-else>
      <circle cx="20" cy="42" r="9" fill="#FFB9C5" />
      <path d="M15 39 q5 3 10 -1" stroke="#F77E9D" stroke-width="2" fill="none" stroke-linecap="round" />
      <path d="M15 46 q5 3 10 0" stroke="#F77E9D" stroke-width="2" fill="none" stroke-linecap="round" />
      <path d="M26 42 Q38 78 32 86" stroke="#4A3223" stroke-width="2" fill="none" stroke-linecap="round" opacity="0.35" />
      <g class="deco-paws" opacity="0.35">
        <ellipse cx="98" cy="24" rx="6" ry="5" fill="#F2994A" />
        <circle cx="94" cy="18" r="1.6" fill="#F2994A" />
        <circle cx="98" cy="16.5" r="1.6" fill="#F2994A" />
        <circle cx="102" cy="18" r="1.6" fill="#F2994A" />
        <ellipse cx="108" cy="34" rx="5" ry="4" fill="#F2994A" opacity="0.6" />
        <circle cx="105" cy="29.5" r="1.3" fill="#F2994A" opacity="0.6" />
        <circle cx="108.5" cy="28.5" r="1.3" fill="#F2994A" opacity="0.6" />
        <circle cx="112" cy="29.5" r="1.3" fill="#F2994A" opacity="0.6" />
      </g>
    </template>
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
.deco--cat {
  animation: deco-bob 3.4s ease-in-out infinite;
}
.deco-tail,
.deco-tail-outline {
  transform-origin: 86px 80px;
  animation: deco-tail-wag 3s ease-in-out infinite;
}
.deco-sparkles {
  animation: deco-twinkle 2s ease-in-out infinite;
}
.deco-paw {
  animation: deco-twinkle 2.6s ease-in-out infinite;
}
@keyframes deco-bob {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-4px); }
}
@keyframes deco-tail-w {
  0%, 100% { transform: rotate(0deg); }
  50% { transform: rotate(-4deg); }
}
@keyframes deco-twinkle {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}
</style>