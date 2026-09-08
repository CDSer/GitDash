<!--
  分组选择器
  支持下拉选择已有分组，或快速新建分组
  modelValue 为空字符串表示「未分组」
-->
<template>
  <div class="group-picker">
    <Select
      :model-value="modelValue ?? ''"
      :options="options"
      placeholder="未分组"
      @update:model-value="onSelect"
    />

    <div v-if="!showCreate" class="create-link">
      <Button variant="ghost" size="sm" @click="showCreate = true">
        <Plus :size="13" /> 新建分组
      </Button>
    </div>

    <div v-else class="create-form">
      <Input
        v-model="newName"
        placeholder="分组名称"
        maxlength="20"
        @keyup.enter="createGroup"
      />
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
      <div class="create-actions">
        <Button variant="ghost" size="sm" @click="cancelCreate">取消</Button>
        <Button
          variant="primary"
          size="sm"
          :disabled="!newName.trim() || isCreating"
          @click="createGroup"
        >
          {{ isCreating ? '创建中...' : '创建' }}
        </Button>
      </div>
      <Alert v-if="error" variant="error" :title="error" class="create-error" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Plus } from 'lucide-vue-next';
import { useAppStore } from '../../stores/appStore';
import Select from './Select.vue';
import Input from './Input.vue';
import Button from './Button.vue';
import Alert from './Alert.vue';

const props = defineProps<{
  modelValue?: string | null;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string | null): void;
}>();

const appStore = useAppStore();

const showCreate = ref(false);
const newName = ref('');
const selectedColor = ref('#3b82f6');
const isCreating = ref(false);
const error = ref('');

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

const options = computed(() => {
  const list = [{ label: '未分组', value: '' }];
  appStore.groups.forEach((g) => {
    list.push({ label: g.name, value: g.id });
  });
  return list;
});

watch(
  () => props.modelValue,
  () => {
    error.value = '';
  }
);

function onSelect(value: string) {
  emit('update:modelValue', value || null);
}

function cancelCreate() {
  showCreate.value = false;
  newName.value = '';
  selectedColor.value = '#3b82f6';
  error.value = '';
}

async function createGroup() {
  const name = newName.value.trim();
  if (!name || isCreating.value) return;

  isCreating.value = true;
  error.value = '';

  try {
    const group = await appStore.addGroup(name, selectedColor.value);
    emit('update:modelValue', group.id);
    cancelCreate();
  } catch (err) {
    error.value = err instanceof Error ? err.message : '创建分组失败';
  } finally {
    isCreating.value = false;
  }
}
</script>

<style scoped>
.group-picker {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.create-link {
  display: flex;
  justify-content: flex-start;
}
.create-form {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background-color: var(--background);
}
.color-picker {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.color-item {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  padding: 0;
  transition: transform 0.15s;
}
.color-item.is-active {
  border-color: var(--primary);
  transform: scale(1.12);
}
.create-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
.create-error {
  margin-top: 4px;
}
</style>
