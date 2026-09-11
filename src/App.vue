<!--
  应用根组件
  挂载路由视图，并负责主题初始化与全局 Toast 宿主
-->
<template>
  <RouterView />
  <ToastHost />
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { RouterView } from 'vue-router';
import ToastHost from './components/ui/ToastHost.vue';
import { setupTheme } from './composables/useTheme';
import { setupSkin } from './composables/useSkin';
import { invoke } from '@tauri-apps/api/core';

function onKeydown(e: KeyboardEvent) {
  // Cmd/Ctrl + Shift + I 切换开发者工具（仅 debug 构建有效）
  if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === 'i') {
    e.preventDefault();
    invoke('toggle_devtools').catch(() => {});
  }
}

onMounted(() => {
  setupTheme();
  setupSkin();
  window.addEventListener('keydown', onKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown);
});
</script>
