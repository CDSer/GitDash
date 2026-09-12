<template>
  <div ref="root" class="ui-select">
    <button
      type="button"
      class="ui-select-trigger"
      :class="{ 'is-open': open }"
      @click="toggle"
    >
      <span :class="cn('truncate', !selectedLabel && 'text-muted-foreground')">
        {{ selectedLabel || placeholder }}
      </span>
      <ChevronDown class="ui-select-caret" :size="13" />
    </button>

    <div v-if="open" class="ui-select-panel" @click.stop>
      <button
        v-for="opt in options"
        :key="opt.value"
        type="button"
        class="ui-select-option"
        :class="{ active: opt.value === modelValue }"
        @click="choose(opt.value)"
      >
        <Check v-if="opt.value === modelValue" class="ui-select-check" :size="14" />
        <span class="truncate">{{ opt.label }}</span>
      </button>
      <div v-if="!options.length" class="ui-select-empty">无选项</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onBeforeUnmount, onMounted } from 'vue';
import { ChevronDown, Check } from 'lucide-vue-next';
import { cn } from '../../lib/utils';

interface Option {
  label: string;
  value: string;
}

const props = withDefaults(
  defineProps<{
    modelValue?: string;
    options?: Option[];
    placeholder?: string;
    class?: string;
  }>(),
  { modelValue: '', options: () => [], placeholder: '请选择' },
);

const emit = defineEmits<{ (e: 'update:modelValue', v: string): void }>();

const open = ref(false);
const root = ref<HTMLElement | null>(null);

const selectedLabel = computed(
  () => props.options.find((o) => o.value === props.modelValue)?.label ?? '',
);

function toggle() {
  open.value = !open.value;
}
function choose(value: string) {
  emit('update:modelValue', value);
  open.value = false;
}
function onDocClick(e: MouseEvent) {
  if (root.value && !root.value.contains(e.target as Node)) open.value = false;
}

onMounted(() => document.addEventListener('mousedown', onDocClick));
onBeforeUnmount(() => document.removeEventListener('mousedown', onDocClick));
</script>

<style scoped>
.ui-select {
  position: relative;
  width: 100%;
}
.ui-select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  width: 100%;
  height: 28px;
  padding: 0 8px 0 10px;
  font-size: 13px;
  border-radius: 6px;
  border: 0.5px solid var(--input);
  background-color: color-mix(in srgb, var(--card) 80%, transparent);
  color: var(--foreground);
  cursor: default;
  transition: border-color 0.15s var(--ease-out), box-shadow 0.15s var(--ease-out);
}
.ui-select-trigger:hover {
  border-color: color-mix(in srgb, var(--input) 70%, var(--muted-foreground));
}
.ui-select-trigger.is-open {
  border-color: var(--ring);
  box-shadow: 0 0 0 3.5px color-mix(in srgb, var(--ring) 28%, transparent);
}
.ui-select-caret {
  flex-shrink: 0;
  color: var(--muted-foreground);
  opacity: 0.7;
}
.ui-select-panel {
  position: absolute;
  z-index: 50;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  max-height: 260px;
  overflow-y: auto;
  padding: 4px;
  border-radius: 8px;
  border: 0.5px solid transparent;
  background-color: var(--popover-glass);
  backdrop-filter: saturate(var(--glass-saturate)) blur(var(--glass-popover-blur));
  -webkit-backdrop-filter: saturate(var(--glass-saturate)) blur(var(--glass-popover-blur));
  color: var(--popover-foreground);
  box-shadow: var(--glass-float-edge), var(--shadow-md);
}
.ui-select-option {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 6px 8px;
  font-size: 13px;
  border: none;
  border-radius: 5px;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: default;
}
.ui-select-option:hover {
  background-color: var(--primary);
  color: #fff;
}
.ui-select-option.active {
  color: var(--primary);
}
.ui-select-option:hover.active {
  color: #fff;
}
.ui-select-check {
  flex-shrink: 0;
  color: var(--primary);
}
.ui-select-option:hover .ui-select-check {
  color: #fff;
}
.ui-select-empty {
  padding: 8px;
  font-size: 12px;
  color: var(--muted-foreground);
  text-align: center;
}
</style>
