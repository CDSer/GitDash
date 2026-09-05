<template>
  <div ref="root" class="ui-dropdown">
    <div class="ui-dropdown-trigger" @click="onClick" @contextmenu.prevent="onContext">
      <slot name="trigger" />
    </div>
    <div v-if="open" class="ui-dropdown-panel" @click="close">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';

const props = withDefaults(
  defineProps<{
    trigger?: 'click' | 'contextmenu';
  }>(),
  { trigger: 'click' },
);

const open = ref(false);
const root = ref<HTMLElement | null>(null);

function onClick() {
  if (props.trigger === 'click') open.value = !open.value;
}
function onContext() {
  if (props.trigger === 'contextmenu') open.value = !open.value;
}
function close() {
  open.value = false;
}
function onDocClick(e: MouseEvent) {
  if (root.value && !root.value.contains(e.target as Node)) open.value = false;
}

onMounted(() => document.addEventListener('mousedown', onDocClick));
onBeforeUnmount(() => document.removeEventListener('mousedown', onDocClick));
</script>

<style scoped>
.ui-dropdown {
  position: relative;
  display: inline-flex;
}
.ui-dropdown-panel {
  position: absolute;
  z-index: 60;
  top: calc(100% + 4px);
  right: 0;
  min-width: 180px;
  padding: 4px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background-color: var(--popover);
  color: var(--popover-foreground);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}
</style>
