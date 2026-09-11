<template>
  <div class="ui-empty">
    <div class="ui-empty-icon" :class="{ 'ui-empty-icon--skin': useSkinDecoration }">
      <slot name="icon">
        <SkinDecoration v-if="useSkinDecoration" :size="decorationSize" kind="empty" />
        <Inbox v-else :size="32" />
      </slot>
    </div>
    <p class="ui-empty-text">{{ description }}</p>
    <slot />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Inbox } from 'lucide-vue-next';
import { useAppStore } from '../../stores/appStore';
import { getSkin } from '../../skins';
import SkinDecoration from '../../skins/components/SkinDecoration.vue';

const props = withDefaults(
  defineProps<{
    description?: string;
    /** 有皮肤装饰时用插画代替默认 Inbox 图标 */
    skin?: boolean;
    /** 皮肤装饰尺寸（px） */
    decorationSize?: number;
  }>(),
  {
    description: '暂无数据',
    skin: false,
    decorationSize: 64,
  },
);

const appStore = useAppStore();

const useSkinDecoration = computed(() => {
  if (!props.skin) return false;
  const skin = getSkin(appStore.settings.skin || 'default');
  return Boolean(skin.decorations?.empty);
});
</script>

<style scoped>
.ui-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 32px 16px;
  color: var(--muted-foreground);
}
.ui-empty-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.6;
}
.ui-empty-icon--skin {
  opacity: 1;
}
.ui-empty-text {
  margin: 0;
  font-size: 13px;
}
</style>
