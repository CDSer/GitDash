<!--
  分组模态框（添加 / 重命名共用）
  group 为 null 时是添加模式，否则是编辑模式
-->
<template>
  <Dialog v-model="visible" :title="isEditing ? '重命名分组' : '添加分组'" width="420px">
    <div class="space-y-4">
      <div class="form-item">
        <label class="form-label">分组名称</label>
        <Input
          v-model="name"
          placeholder="我的分组"
          maxlength="20"
          @keyup.enter="submit"
        />
      </div>

      <div class="form-item">
        <label class="form-label">颜色</label>
        <div class="color-picker">
          <button
            v-for="color in colors"
            :key="color"
            type="button"
            class="color-item"
            :class="{ 'is-active': selectedColor === color }"
            :style="{ backgroundColor: color }"
            @click="selectedColor = color"
          />
        </div>
      </div>
    </div>

    <Alert v-if="error" variant="error" :title="error" />

    <template #footer>
      <Button variant="ghost" @click="visible = false">取消</Button>
      <Button variant="primary" :disabled="!name.trim() || isSaving" @click="submit">
        {{ isEditing ? '保存' : '添加' }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { Group } from '../../types';
import { useAppStore } from '../../stores/appStore';
import Dialog from '../ui/Dialog.vue';
import Input from '../ui/Input.vue';
import Button from '../ui/Button.vue';
import Alert from '../ui/Alert.vue';
import { toast } from '../../lib/toast';

const visible = defineModel<boolean>({ required: true });
const props = defineProps<{ group: Group | null }>();
const emit = defineEmits(['saved']);

const appStore = useAppStore();

const name = ref('');
const selectedColor = ref('#3b82f6');
const isSaving = ref(false);
const error = ref('');

const isEditing = computed(() => !!props.group);

const colors = [
  '#3b82f6',
  '#22c55e',
  '#f59e0b',
  '#ef4444',
  '#a855f7',
  '#ec4899',
  '#06b6d4',
  '#84cc16',
];

watch(visible, (open) => {
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
      toast.success('分组已更新');
    } else {
      await appStore.addGroup(name.value.trim(), selectedColor.value);
      toast.success('分组已添加');
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
.form-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.form-label {
  font-size: 13px;
  font-weight: 500;
}
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
  border-color: var(--primary);
  transform: scale(1.15);
}
</style>
