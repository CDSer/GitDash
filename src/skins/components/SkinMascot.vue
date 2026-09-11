<!--
  皮肤吉祥物渲染器
  根据当前 settings.skin 从注册表取对应吉祥物组件并渲染
  mood 可外部传入；未传时自动联动 useSkinMood
-->
<template>
  <component
    v-if="mascotComponent"
    :is="mascotComponent"
    :size="size"
    :mood="resolvedMood"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useAppStore } from '../../stores/appStore';
import { getSkin } from '../../skins';
import { useSkinMood } from '../../composables/useSkinMood';
import type { SkinMood } from '../../types/skin';

const props = withDefaults(
  defineProps<{ size?: number; mood?: SkinMood | 'auto' }>(),
  { size: 32, mood: 'auto' },
);

const appStore = useAppStore();
const { mood: autoMood } = useSkinMood();

const resolvedMood = computed<SkinMood>(() => {
  if (props.mood === 'auto') return autoMood.value;
  return props.mood;
});

const mascotComponent = computed(() => {
  const skin = getSkin(appStore.settings.skin || 'default');
  if (!skin.mascot) return null;
  const assets = skin.mascot;
  const m = resolvedMood.value;
  if (m !== 'idle' && assets[m]) {
    return assets[m]!;
  }
  return assets.idle;
});
</script>
