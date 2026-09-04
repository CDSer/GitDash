<!--
  添加项目模态框
  支持输入路径、选择文件夹、拖拽文件夹
-->
<template>
  <el-dialog
    v-model="visible"
    title="添加项目"
    width="460px"
    :close-on-click-modal="false"
    append-to-body
  >
    <div
      class="drop-zone"
      :class="{ 'is-dragover': isDragging }"
      @dragover.prevent="isDragging = true"
      @dragleave="isDragging = false"
      @drop.prevent="handleDrop"
      @click="selectFolder"
    >
      <el-icon :size="40" class="drop-icon"><UploadFilled /></el-icon>
      <p class="drop-text">{{ isDragging ? '松开以添加' : '拖拽文件夹到此处' }}</p>
      <p class="drop-hint">或点击浏览</p>
    </div>

    <el-form class="path-form" label-width="0" @submit.prevent>
      <el-form-item>
        <el-input
          v-model="path"
          placeholder="/path/to/git/repo"
          @keyup.enter="addProject"
        />
      </el-form-item>
    </el-form>

    <el-alert v-if="error" type="error" :title="error" :closable="false" show-icon />

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button
        type="primary"
        :loading="isAdding"
        :disabled="!path.trim()"
        @click="addProject"
      >
        {{ isAdding ? '添加中...' : '添加' }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { UploadFilled } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import { useAppStore } from '../../stores/appStore';
import { open } from '@tauri-apps/plugin-dialog';

const visible = defineModel<boolean>({ required: true });

const emit = defineEmits(['added']);

const appStore = useAppStore();

const path = ref('');
const isDragging = ref(false);
const isAdding = ref(false);
const error = ref('');

async function selectFolder() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 Git 仓库'
    });

    if (selected) {
      path.value = selected as string;
      error.value = '';
    }
  } catch (err) {
    error.value = '选择文件夹失败';
  }
}

function handleDrop(e: DragEvent) {
  isDragging.value = false;
  const file = e.dataTransfer?.files?.[0] as (File & { path?: string }) | undefined;
  const droppedPath = file?.path;

  if (droppedPath) {
    path.value = droppedPath;
    error.value = '';
  } else {
    error.value = '无法获取拖拽目录的完整路径，请点击上方区域浏览选择';
  }
}

async function addProject() {
  if (!path.value.trim()) return;

  isAdding.value = true;
  error.value = '';

  try {
    await appStore.addProject(path.value.trim());
    ElMessage.success('项目已添加');
    emit('added');
    visible.value = false;
    path.value = '';
  } catch (err) {
    error.value = err instanceof Error ? err.message : '添加项目失败';
  } finally {
    isAdding.value = false;
  }
}
</script>

<style scoped>
.drop-zone {
  border: 1px dashed var(--el-border-color);
  border-radius: 8px;
  padding: 24px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.2s, background-color 0.2s;
}

.drop-zone:hover,
.drop-zone.is-dragover {
  border-color: var(--el-color-primary);
  background-color: var(--el-color-primary-light-9);
}

.drop-icon {
  color: var(--el-text-color-secondary);
}

.drop-text {
  margin: 12px 0 4px;
  font-size: 14px;
  color: var(--el-text-color-regular);
}

.drop-hint {
  margin: 0;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.path-form {
  margin-top: 16px;
}
</style>
