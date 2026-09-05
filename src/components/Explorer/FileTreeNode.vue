<!--
  文件树节点（递归自引用组件）
  目录懒加载：首次展开时调用 list_directory
-->
<template>
  <div class="tree-branch">
    <div class="tree-row" :style="{ paddingLeft: depth * 12 + 6 + 'px' }" @click="toggle">
      <ChevronRight v-if="node.is_dir" :size="14" class="tree-caret" :class="{ 'is-open': node.expanded }" />
      <span v-else class="tree-caret-spacer" />
      <Icon :icon="iconFor" class="tree-icon" />
      <span class="tree-label" :title="node.path">{{ node.name }}</span>
    </div>

    <div v-if="node.expanded && node.children.length" class="tree-children">
      <FileTreeNode
        v-for="child in node.children"
        :key="child.path"
        :node="child"
        :depth="depth + 1"
        @open-file="(p: string) => emit('open-file', p)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive } from 'vue';
import { Icon } from '@iconify/vue';
import { ChevronRight } from 'lucide-vue-next';
import { listDirectory } from '../../lib/tauriApi';
import type { FileNode } from '../../types';

interface TreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  has_children: boolean;
  expanded: boolean;
  loaded: boolean;
  children: TreeNode[];
}

const props = defineProps<{
  node: TreeNode;
  depth: number;
}>();

const emit = defineEmits<{ (e: 'open-file', path: string): void }>();

const folderMap: Record<string, string> = {
  src: 'folder-src',
  source: 'folder-src',
  node_modules: 'folder-node',
  docs: 'folder-docs',
  documentation: 'folder-docs',
  out: 'folder-dist',
  dist: 'folder-dist',
  build: 'folder-dist',
  templates: 'folder-template',
  template: 'folder-template',
  public: 'folder-public',
  static: 'folder-public',
  test: 'folder-test',
  tests: 'folder-test',
  __tests__: 'folder-test',
  assets: 'folder-images',
  images: 'folder-images',
  image: 'folder-images',
  img: 'folder-images',
  '.git': 'folder-git',
  '.github': 'folder-github',
  config: 'folder-config',
  configurations: 'folder-config',
  scripts: 'folder-scripts',
  components: 'folder-components',
  views: 'folder-views',
  pages: 'folder-views',
};

const fileMap: Record<string, string> = {
  'package.json': 'file-type-node',
  'package-lock.json': 'file-type-node',
  'yarn.lock': 'file-type-yarn',
  'pnpm-lock.yaml': 'file-type-pnpm',
  'tsconfig.json': 'file-type-tsconfig',
  'tsconfig.node.json': 'file-type-tsconfig',
  'jsconfig.json': 'file-type-jsconfig',
  'vite.config.ts': 'file-type-vite',
  'vite.config.js': 'file-type-vite',
  'cargo.toml': 'file-type-rust',
  'cargo.lock': 'file-type-rust',
  'readme.md': 'file-type-readme',
  'readme.txt': 'file-type-readme',
  '.gitignore': 'file-type-git',
  '.gitattributes': 'file-type-git',
  dockerfile: 'file-type-docker',
  '.env': 'file-type-config',
  '.npmrc': 'file-type-npm',
  '.editorconfig': 'file-type-config',
};

const extMap: Record<string, string> = {
  ts: 'file-type-typescript',
  tsx: 'file-type-typescript',
  js: 'file-type-js',
  jsx: 'file-type-reactjs',
  mjs: 'file-type-js',
  cjs: 'file-type-js',
  vue: 'file-type-vue',
  json: 'file-type-json',
  md: 'file-type-markdown',
  markdown: 'file-type-markdown',
  html: 'file-type-html',
  htm: 'file-type-html',
  css: 'file-type-css',
  scss: 'file-type-scss',
  less: 'file-type-less',
  sass: 'file-type-sass',
  py: 'file-type-python',
  rs: 'file-type-rust',
  go: 'file-type-go',
  java: 'file-type-java',
  kt: 'file-type-kotlin',
  c: 'file-type-c',
  h: 'file-type-c',
  cpp: 'file-type-cpp',
  cc: 'file-type-cpp',
  hpp: 'file-type-cpp',
  sh: 'file-type-shell',
  bash: 'file-type-shell',
  zsh: 'file-type-shell',
  yml: 'file-type-yaml',
  yaml: 'file-type-yaml',
  toml: 'file-type-toml',
  sql: 'file-type-sql',
  php: 'file-type-php',
  rb: 'file-type-ruby',
  swift: 'file-type-swift',
  dart: 'file-type-dart',
  png: 'file-type-image',
  jpg: 'file-type-image',
  jpeg: 'file-type-image',
  gif: 'file-type-image',
  webp: 'file-type-image',
  bmp: 'file-type-image',
  ico: 'file-type-image',
  svg: 'file-type-svg',
  pdf: 'file-type-pdf',
  zip: 'file-type-zip',
  tar: 'file-type-zip',
  gz: 'file-type-zip',
  '7z': 'file-type-zip',
  rar: 'file-type-zip',
  log: 'file-type-log',
  xml: 'file-type-xml',
  txt: 'file-type-text',
  lock: 'file-type-lock',
};

function folderIcon(name: string, expanded: boolean) {
  const base = folderMap[name.toLowerCase()] ?? 'default-folder';
  return `vscode-icons:${base}${expanded ? '-open' : ''}`;
}

function fileIcon(name: string) {
  const lower = name.toLowerCase();
  if (fileMap[lower]) return `vscode-icons:${fileMap[lower]}`;
  const ext = name.includes('.') ? name.split('.').pop()!.toLowerCase() : '';
  return `vscode-icons:${extMap[ext] ?? 'default-file'}`;
}

const iconFor = computed(() =>
  props.node.is_dir ? folderIcon(props.node.name, props.node.expanded) : fileIcon(props.node.name),
);

async function loadDir(node: TreeNode) {
  try {
    const list: FileNode[] = await listDirectory(node.path);
    node.children = list.map((n) =>
      reactive<TreeNode>({
        name: n.name,
        path: n.path,
        is_dir: n.is_dir,
        has_children: n.has_children,
        expanded: false,
        loaded: false,
        children: [],
      }),
    );
  } catch (error) {
    console.error('加载目录失败：', error);
    node.children = [];
  } finally {
    node.loaded = true;
    node.expanded = true;
  }
}

function toggle() {
  if (!props.node.is_dir) {
    emit('open-file', props.node.path);
    return;
  }
  if (props.node.expanded) {
    props.node.expanded = false;
  } else if (!props.node.loaded) {
    void loadDir(props.node);
  } else {
    props.node.expanded = true;
  }
}
</script>

<style scoped>
.tree-row {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 26px;
  padding-right: 8px;
  cursor: pointer;
  font-size: 13px;
  color: var(--foreground);
  border-radius: 4px;
}
.tree-row:hover {
  background-color: var(--accent);
  color: var(--accent-foreground);
}
.tree-caret {
  flex-shrink: 0;
  color: var(--muted-foreground);
  transition: transform 0.12s;
}
.tree-caret.is-open {
  transform: rotate(90deg);
}
.tree-caret-spacer {
  display: inline-block;
  width: 14px;
  flex-shrink: 0;
}
.tree-icon {
  flex-shrink: 0;
  width: 16px;
  height: 16px;
}
.tree-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.tree-children {
  min-height: 0;
}
</style>
