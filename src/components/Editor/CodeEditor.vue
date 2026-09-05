<!--
  代码编辑器组件（基于 vue-codemirror / CodeMirror 6）
  - props.modelValue：文件内容
  - props.language：文件扩展名（用于选择高亮语言）
  - emit('update:modelValue', value)：内容变化
  - emit('save')：Ctrl/Cmd+S 保存（已拦截浏览器默认保存）
  注意：one-dark 主题固定，后续可跟随应用主题切换
-->
<template>
  <Codemirror
    :model-value="modelValue"
    :extensions="extensions"
    :style="{ height: '100%' }"
    @update:model-value="onInput"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Codemirror } from 'vue-codemirror';
import { keymap } from '@codemirror/view';
import { Prec, type Extension } from '@codemirror/state';
import { oneDark } from '@codemirror/theme-one-dark';
import { javascript } from '@codemirror/lang-javascript';
import { json } from '@codemirror/lang-json';
import { markdown } from '@codemirror/lang-markdown';
import { html } from '@codemirror/lang-html';
import { css } from '@codemirror/lang-css';
import { rust } from '@codemirror/lang-rust';

const props = defineProps<{
  modelValue: string;
  language: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'save'): void;
}>();

// 扩展名 -> CodeMirror 语言包（未匹配时回退为纯文本）
const langMap: Record<string, () => Extension> = {
  js: javascript,
  mjs: javascript,
  cjs: javascript,
  ts: () => javascript({ typescript: true }),
  jsx: () => javascript({ jsx: true }),
  tsx: () => javascript({ jsx: true, typescript: true }),
  json,
  md: markdown,
  markdown: markdown,
  html: html,
  htm: html,
  vue: html,
  css,
  scss: css,
  less: css,
  rust,
  rs: rust,
};

const extensions = computed<Extension[]>(() => {
  const factory = langMap[props.language.toLowerCase()];
  const langExt: Extension[] = factory ? [factory()] : [];

  // Ctrl/Cmd+S 触发保存，并阻止浏览器默认行为
  const saveKeymap = keymap.of([
    {
      key: 'Mod-s',
      preventDefault: true,
      run: () => {
        emit('save');
        return true;
      },
    },
  ]);

  return [Prec.highest(saveKeymap), ...langExt, oneDark];
});

function onInput(value: string) {
  emit('update:modelValue', value);
}
</script>

<style scoped>
:deep(.cm-editor) {
  height: 100%;
  font-size: 13px;
}

:deep(.cm-scroller) {
  overflow: auto;
  font-family: 'JetBrains Mono', 'Fira Code', Menlo, Consolas, monospace;
}
</style>
