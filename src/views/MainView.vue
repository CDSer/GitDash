<!--
  主页面视图
  组合：标题栏、侧边栏分组树、工具栏、项目列表、操作队列、模态框
-->
<template>
  <div class="main-view">
    <el-container class="main-body">
      <!-- 侧边栏：分组树 -->
      <el-aside width="220px" class="group-aside">
        <GroupTree />
      </el-aside>

      <el-container>
        <!-- 工具栏 -->
        <el-header height="48px" class="toolbar">
          <div class="toolbar-left">
            <el-button type="primary" :icon="Plus" @click="showAddModal = true">
              添加项目
            </el-button>
            <el-button :icon="Setting" title="设置" @click="showSettings = true" />
          </div>
          <span class="project-count">{{ projects.length }} 个项目</span>
        </el-header>

        <!-- 项目列表 -->
        <el-main class="main-content">
          <ProjectTable />
        </el-main>
      </el-container>
    </el-container>

    <!-- 操作队列面板 -->
    <OperationQueue />

    <!-- 模态框 -->
    <AddProjectModal v-model="showAddModal" />
    <SettingsModal v-model="showSettings" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { Plus, Setting } from '@element-plus/icons-vue';
import { useAppStore } from '../stores/appStore';
import GroupTree from '../components/Sidebar/GroupTree.vue';
import ProjectTable from '../components/ProjectList/ProjectTable.vue';
import OperationQueue from '../components/OperationPanel/OperationQueue.vue';
import AddProjectModal from '../components/Modals/AddProjectModal.vue';
import SettingsModal from '../components/Modals/SettingsModal.vue';

const appStore = useAppStore();

const showAddModal = ref(false);
const showSettings = ref(false);

const projects = computed(() => appStore.projects);

onMounted(async () => {
  await appStore.loadConfig();
});
</script>

<style scoped>
.main-view {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: var(--el-bg-color);
  overflow: hidden;
}

.main-body {
  flex: 1;
  overflow: hidden;
}

.group-aside {
  overflow: hidden;
  border-right: 1px solid var(--el-border-color);
  background-color: var(--el-bg-color-overlay);
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  border-bottom: 1px solid var(--el-border-color);
  background-color: var(--el-bg-color-overlay);
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.project-count {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.main-content {
  padding: 0;
  overflow: hidden;
}
</style>
