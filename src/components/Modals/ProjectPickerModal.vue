<!--
  从已管理项目中选择并打开标签
-->
<template>
  <Dialog v-model="visible" title="打开项目" width="420px">
    <div v-if="!projects.length" class="py-6 text-center text-sm text-muted-foreground">
      暂无已管理项目，请先添加
    </div>
    <div v-else class="picker-list">
      <button
        v-for="p in projects"
        :key="p.id"
        type="button"
        class="picker-item"
        :disabled="isTabOpen(p.id)"
        @click="open(p.id)"
      >
        <div class="picker-body">
          <div class="picker-name">
            {{ p.name }}
            <span v-if="isTabOpen(p.id)" class="picker-opened">已打开</span>
          </div>
          <div class="picker-path">{{ p.path }}</div>
        </div>
      </button>
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useAppStore } from '../../stores/appStore';
import { useTabStore } from '../../stores/tabStore';
import Dialog from '../ui/Dialog.vue';

const visible = defineModel<boolean>({ required: true });

const appStore = useAppStore();
const tabStore = useTabStore();

const projects = computed(() => appStore.projects);

function isTabOpen(id: string) {
  return tabStore.openProjectIds.includes(id);
}

function open(id: string) {
  tabStore.openProject(id, 'history');
  visible.value = false;
}
</script>

<style scoped>
.picker-list {
  max-height: 360px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.picker-item {
  display: flex;
  align-items: center;
  width: 100%;
  border-radius: 8px;
  padding: 10px 12px;
  text-align: left;
  cursor: pointer;
}
.picker-item:hover:not(:disabled) {
  background-color: var(--accent);
}
.picker-item:disabled {
  opacity: 0.55;
  cursor: default;
}
.picker-body {
  min-width: 0;
  flex: 1;
}
.picker-name {
  font-size: 13px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 8px;
}
.picker-opened {
  font-size: 11px;
  font-weight: 400;
  color: var(--muted-foreground);
}
.picker-path {
  margin-top: 2px;
  font-size: 11px;
  color: var(--muted-foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
