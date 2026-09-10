<!--
  合并分支对话框
  选择目标分支，预览 ahead/behind，确认后发起 merge
-->
<template>
  <Dialog v-model="visible" :title="`合并 · ${project?.name ?? ''}`" width="420px">
    <div v-if="project" class="mb space-y-3">
      <div class="mb-row">
        <span class="mb-label">当前分支</span>
        <span class="mb-value">{{ currentBranch || '-' }}</span>
      </div>

      <div class="mb-row">
        <span class="mb-label">合并</span>
        <select v-model="target" class="mb-select" :disabled="busy || !options.length">
          <option value="" disabled>选择要合入的分支</option>
          <option v-for="b in options" :key="b.name" :value="b.name">
            {{ b.display_name }}
          </option>
        </select>
      </div>

      <p v-if="hint" class="mb-hint">{{ hint }}</p>
      <p v-if="!options.length" class="mb-hint">没有其它本地分支可合并</p>
      <p v-else class="mb-tip">将把选中分支合并到当前分支（merge 到 {{ currentBranch || 'HEAD' }}）</p>
    </div>

    <template #footer>
      <Button variant="ghost" :disabled="busy" @click="visible = false">取消</Button>
      <Button variant="primary" :disabled="busy || !target" @click="confirm">
        {{ busy ? '合并中…' : '合并' }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { Branch, MergeResult, Project } from '../../types';
import { gitMerge } from '../../lib/tauriApi';
import Dialog from '../ui/Dialog.vue';
import Button from '../ui/Button.vue';
import { toast } from '../../lib/toast';

const visible = defineModel<boolean>({ required: true });
const props = defineProps<{
  project: Project | null;
  branches: Branch[];
  currentBranch: string;
}>();

const emit = defineEmits<{
  (e: 'merged', result: MergeResult): void;
}>();

const target = ref('');
const busy = ref(false);

const options = computed(() =>
  props.branches.filter((b) => b.is_local && !b.is_current && !b.is_detached),
);

const hint = computed(() => {
  if (!target.value) return '';
  const b = props.branches.find((x) => x.name === target.value);
  if (!b) return '';
  if (b.upstream) return `上游：${b.upstream}`;
  return '';
});

watch(visible, (open) => {
  if (open) {
    target.value = '';
  }
});

async function confirm() {
  if (!props.project || !target.value || busy.value) return;
  busy.value = true;
  try {
    const result = await gitMerge(props.project.id, target.value);
    emit('merged', result);
    if (result.has_conflicts) {
      toast.error('合并存在冲突，请打开源码控制解决');
    } else {
      toast.success(result.message || `已合并 ${target.value}`);
    }
    visible.value = false;
  } catch (e) {
    toast.error(typeof e === 'string' ? e : '合并失败');
  } finally {
    busy.value = false;
  }
}
</script>

<style scoped>
.mb {
  padding-top: 4px;
}
.mb-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.mb-label {
  width: 64px;
  flex-shrink: 0;
  font-size: 13px;
  color: var(--muted-foreground);
}
.mb-value {
  font-size: 13px;
  font-weight: 600;
}
.mb-select {
  flex: 1;
  height: 30px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--background);
  color: var(--foreground);
  font-size: 13px;
  padding: 0 6px;
}
.mb-hint {
  margin: 0;
  font-size: 12px;
  color: #16a34a;
}
.mb-tip {
  margin: 0;
  font-size: 12px;
  color: var(--muted-foreground);
  line-height: 1.4;
}
</style>
