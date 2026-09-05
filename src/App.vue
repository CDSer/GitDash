<!--
  应用根组件
  挂载页面视图，并负责主题初始化与全局 Toast 宿主
-->
<template>
  <MainView v-if="!appStore.workspaceProjectId" />
  <WorkspaceView v-else />
  <ToastHost />
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import MainView from './views/MainView.vue';
import WorkspaceView from './views/WorkspaceView.vue';
import ToastHost from './components/ui/ToastHost.vue';
import { useAppStore } from './stores/appStore';
import { setupTheme } from './composables/useTheme';

const appStore = useAppStore();

onMounted(() => {
  setupTheme();
});
</script>
