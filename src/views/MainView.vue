<!--
  主页面视图
  组合：侧边栏分组树、工具栏、项目列表、操作队列、模态框
-->
<template>
  <div class="flex h-screen flex-col overflow-hidden bg-background text-foreground">
    <div class="flex min-h-0 flex-1">
      <aside class="w-[220px] shrink-0 overflow-hidden border-r border-border bg-card">
        <GroupTree />
      </aside>

      <div class="flex min-w-0 flex-1 flex-col">
        <header
          class="flex h-12 shrink-0 items-center justify-between border-b border-border bg-card px-4"
        >
          <div class="flex items-center gap-2">
            <Button variant="primary" @click="showAddModal = true">
              <Plus :size="14" /> 添加项目
            </Button>
            <Button variant="ghost" size="icon" title="设置" @click="showSettings = true">
              <Settings :size="16" />
            </Button>
          </div>
          <span class="text-xs text-muted-foreground">{{ projects.length }} 个项目</span>
        </header>

        <main class="min-h-0 flex-1 overflow-hidden">
          <ProjectTable />
        </main>
      </div>
    </div>

    <OperationQueue />

    <AddProjectModal v-model="showAddModal" />
    <SettingsModal v-model="showSettings" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, defineAsyncComponent } from 'vue';
import { Plus, Settings } from 'lucide-vue-next';
import { useAppStore } from '../stores/appStore';
import GroupTree from '../components/Sidebar/GroupTree.vue';
import ProjectTable from '../components/ProjectList/ProjectTable.vue';
import OperationQueue from '../components/OperationPanel/OperationQueue.vue';
import Button from '../components/ui/Button.vue';

const AddProjectModal = defineAsyncComponent(
  () => import('../components/Modals/AddProjectModal.vue'),
);
const SettingsModal = defineAsyncComponent(
  () => import('../components/Modals/SettingsModal.vue'),
);

const appStore = useAppStore();

const showAddModal = ref(false);
const showSettings = ref(false);

const projects = computed(() => appStore.projects);

onMounted(async () => {
  await appStore.loadConfig();
});
</script>
