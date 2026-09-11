<!--
  皮肤装饰插画渲染器
  根据当前皮肤取 decorations 里的插画组件；皮肤无插画时渲染 null
-->
<template>
  <component v-if="decorationComponent" :is="decorationComponent" :size="size" :kind="kind" />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useAppStore } from '../../stores/appStore';
import { getSkin } from '../../skins';

const props = withDefaults(
  defineProps<{ size?: number; kind?: 'empty' | 'allClear' }>(),
  { size: 80, kind: 'empty' },
);

const appStore = useAppStore();

const decorationComponent = computed(() => {
  const skin = getSkin(appStore.settings.skin || 'default');
  if (!skin.decorations) return null;
  return skin.decorations[props.kind] ?? null;
});
</script>
