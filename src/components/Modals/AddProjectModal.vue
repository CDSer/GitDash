<!--
  添加项目模态框
  支持输入路径、选择文件夹、拖拽文件夹
-->
<template>
  <Dialog v-model="visible" title="添加项目" width="460px">
    <div
      class="drop-zone"
      :class="{ 'is-dragover': isDragging }"
      @dragover.prevent="isDragging = true"
      @dragleave="isDragging = false"
      @drop.prevent="handleDrop"
      @click="selectFolder"
    >
      <UploadCloud :size="40" class="drop-icon" />
      <p class="drop-text">{{ isDragging ? '松开以添加' : '拖拽文件夹到此处' }}</p>
      <p class="drop-hint">或点击浏览</p>
    </div>

    <div class="path-form">
      <Input
        v-model="path"
        placeholder="/path/to/git/repo"
        @keyup.enter="addProject"
      />
    </div>

    <div class="group-form">
      <label class="form-label">添加到分组</label>
      <GroupPicker v-model="selectedGroupId" />
    </div>

    <Alert v-if="error" variant="error" :title="error" />

    <template #footer>
      <Button variant="ghost" @click="visible = false">取消</Button>
      <Button
        variant="primary"
        :disabled="!path.trim() || isAdding"
        @click="addProject"
      >
        {{ isAdding ? '添加中...' : '添加' }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { UploadCloud } from 'lucide-vue-next';
import { useAppStore } from '../../stores/appStore';
import { open } from '@tauri-apps/plugin-dialog';
import Dialog from '../ui/Dialog.vue';
import Input from '../ui/Input.vue';
import Button from '../ui/Button.vue';
import Alert from '../ui/Alert.vue';
import GroupPicker from '../ui/GroupPicker.vue';
import { toast } from '../../lib/toast';

const visible = defineModel<boolean>({ required: true });
const emit = defineEmits(['added']);

const appStore = useAppStore();

const path = ref('');
const selectedGroupId = ref<string | null>(null);
const isDragging = ref(false);
const isAdding = ref(false);
const error = ref('');

async function selectFolder() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 Git 仓库',
    });
    if (selected) {
      path.value = selected as string;
      error.value = '';
    }
  } catch {
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
    await appStore.addProject(path.value.trim(), selectedGroupId.value);
    toast.success('项目已添加');
    emit('added');
    visible.value = false;
    path.value = '';
    selectedGroupId.value = null;
  } catch (err) {
    error.value = err instanceof Error ? err.message : '添加项目失败';
  } finally {
    isAdding.value = false;
  }
}
</script>

<style scoped>
.drop-zone {
  border: 1px dashed var(--border);
  border-radius: 8px;
  padding: 24px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.2s, background-color 0.2s;
}
.drop-zone:hover,
.drop-zone.is-dragover {
  border-color: var(--primary);
  background-color: color-mix(in oklab, var(--primary) 10%, transparent);
}
.drop-icon {
  color: var(--muted-foreground);
}
.drop-text {
  margin: 12px 0 4px;
  font-size: 14px;
  color: var(--foreground);
}
.drop-hint {
  margin: 0;
  font-size: 12px;
  color: var(--muted-foreground);
}
.path-form {
  margin-top: 16px;
}
.group-form {
  margin-top: 16px;
}
.form-label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--foreground);
}
</style>
