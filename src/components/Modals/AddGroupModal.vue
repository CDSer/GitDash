<!--
  分组模态框（添加 / 重命名共用）
  group 为 null 时是添加模式，否则是编辑模式
-->
<template>
  <el-dialog
    v-model="visible"
    :title="isEditing ? '重命名分组' : '添加分组'"
    width="420px"
    :close-on-click-modal="false"
    append-to-body
  >
    <el-form label-width="80px" @submit.prevent>
      <el-form-item label="分组名称">
        <el-input
          v-model="name"
          placeholder="我的分组"
          maxlength="20"
          show-word-limit
          @keyup.enter="submit"
        />
      </el-form-item>

      <el-form-item label="颜色">
        <div class="color-picker">
          <button
            v-for="color in colors"
            :key="color"
            type="button"
            class="color-item"
            :class="{ 'is-active': selectedColor === color }"
            :style="{ backgroundColor: color }"
            @click="selectedColor = color"
          ></button>
        </div>
      </el-form-item>
    </el-form>

    <el-alert v-if="error" type="error" :title="error" :closable="false" show-icon />

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button
        type="primary"
        :loading="isSaving"
        :disabled="!name.trim()"
        @click="submit"
      >
        {{ isEditing ? '保存' : '添加' }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { ElMessage } from 'element-plus';
import type { Group } from '../../types';
import { useAppStore } from '../../stores/appStore';

const visible = defineModel<boolean>({ required: true });

/** 编辑目标分组；为 null 表示添加模式 */
const props = defineProps<{ group: Group | null }>();

const emit = defineEmits(['saved']);

const appStore = useAppStore();

const name = ref('');
const selectedColor = ref('#3b82f6');
const isSaving = ref(false);
const error = ref('');

const isEditing = computed(() => !!props.group);

const colors = [
  '#3b82f6', // blue
  '#22c55e', // green
  '#f59e0b', // yellow
  '#ef4444', // red
  '#a855f7', // purple
  '#ec4899', // pink
  '#06b6d4', // cyan
  '#84cc16', // lime
];

// 每次打开弹窗时用当前模式初始化表单
watch(visible, open => {
  if (open) {
    name.value = props.group?.name ?? '';
    selectedColor.value = props.group?.color ?? '#3b82f6';
    error.value = '';
  }
});

async function submit() {
  if (!name.value.trim() || isSaving.value) return;

  isSaving.value = true;
  error.value = '';

  try {
    if (isEditing.value && props.group) {
      await appStore.renameGroup(props.group.id, name.value.trim(), selectedColor.value);
      ElMessage.success('分组已更新');
    } else {
      await appStore.addGroup(name.value.trim(), selectedColor.value);
      ElMessage.success('分组已添加');
    }
    emit('saved');
    visible.value = false;
  } catch (err) {
    error.value = err instanceof Error ? err.message : '保存失败，请重试';
  } finally {
    isSaving.value = false;
  }
}
</script>

<style scoped>
.color-picker {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.color-item {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  padding: 0;
  transition: transform 0.15s;
}

.color-item.is-active {
  border-color: var(--el-color-primary);
  transform: scale(1.15);
}
</style>
