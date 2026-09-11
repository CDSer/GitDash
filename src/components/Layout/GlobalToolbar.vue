<!--
  全局顶栏：横跨侧栏 + 内容区的最顶部
  左侧：公共操作（添加项目 / 批量导入）
  右侧：设置等图标按钮
-->
<template>
  <header class="global-toolbar">
    <div class="toolbar-left">
      <button type="button" class="toolbar-btn" title="添加项目" @click="$emit('add-project')">
        <Plus :size="15" />
        <span>添加项目</span>
      </button>
      <button type="button" class="toolbar-btn" title="批量导入" @click="$emit('batch-import')">
        <FolderPlus :size="15" />
        <span>批量导入</span>
      </button>
    </div>

    <div class="toolbar-right">
      <span v-if="projectCount !== undefined && projectCount > 0" class="toolbar-meta">{{ projectCount }} 个项目</span>
      <button type="button" class="toolbar-icon-btn" title="设置" @click="$emit('settings')">
        <Settings :size="16" />
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { Plus, FolderPlus, Settings } from 'lucide-vue-next';

defineProps<{ projectCount?: number }>();
defineEmits<{
  (e: 'add-project'): void;
  (e: 'batch-import'): void;
  (e: 'settings'): void;
}>();
</script>

<style scoped>
.global-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  flex-shrink: 0;
  padding: 0 12px;
  border-bottom: 1px solid var(--border);
  background: var(--card);
  -webkit-app-region: drag;
  app-region: drag;
}
.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 2px;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}
.toolbar-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  padding: 0 10px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--foreground);
  font-size: 13px;
  cursor: pointer;
  transition: background-color 0.12s;
}
.toolbar-btn:hover {
  background: var(--accent);
  color: var(--accent-foreground);
}
.toolbar-icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--muted-foreground);
  cursor: pointer;
  transition: background-color 0.12s, color 0.12s;
}
.toolbar-icon-btn:hover {
  background: var(--accent);
  color: var(--accent-foreground);
}
.toolbar-meta {
  margin-right: 8px;
  font-size: 12px;
  color: var(--muted-foreground);
}
</style>
