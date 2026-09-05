<!--
  文件树组件（基于 Element Plus el-tree 懒加载）
  - props.rootPath：项目根目录绝对路径
  - emit('open-file', path)：点击文件时抛出文件路径
  后端依赖：list_directory 命令（已在 src-tauri 中实现）
  图标：使用 @iconify/vue + vscode-icons 图标集，按文件/文件夹类型显示对应图标
-->
<template>
  <el-tree
    class="file-tree"
    :load="loadNode"
    lazy
    node-key="path"
    :props="treeProps"
    :expand-on-click-node="false"
    :highlight-current="true"
    @node-click="onNodeClick"
  >
    <template #default="{ node, data }">
      <span class="tree-node">
        <Icon :icon="iconFor(data, node.expanded)" class="node-icon" />
        <span class="node-label" :title="data.path">{{ data.name }}</span>
      </span>
    </template>
  </el-tree>
</template>

<script setup lang="ts">
import { Icon } from '@iconify/vue';
import { listDirectory } from '../../lib/tauriApi';
import type { FileNode } from '../../types';

const props = defineProps<{ rootPath: string }>();
const emit = defineEmits<{ (e: 'open-file', path: string): void }>();

interface TreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  has_children: boolean;
  leaf: boolean;
}

const treeProps = {
  label: 'name',
  isLeaf: 'leaf' as const,
};

// 目录名 -> vscode-icons 图标基础名（展开时追加 -open）
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

function folderIcon(name: string, expanded: boolean) {
  const base = folderMap[name.toLowerCase()] ?? 'default-folder';
  const suffix = expanded ? '-open' : '';
  return `vscode-icons:${base}${suffix}`;
}

// 特殊文件名 -> 图标
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

// 扩展名 -> 图标
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

function fileIcon(name: string) {
  const lower = name.toLowerCase();
  if (fileMap[lower]) return `vscode-icons:${fileMap[lower]}`;
  const ext = name.includes('.') ? name.split('.').pop()!.toLowerCase() : '';
  const icon = extMap[ext] ?? 'default-file';
  return `vscode-icons:${icon}`;
}

function iconFor(data: TreeNode, expanded: boolean) {
  return data.is_dir ? folderIcon(data.name, expanded) : fileIcon(data.name);
}

async function loadNode(
  node: any,
  resolve: (data: TreeNode[]) => void
) {
  // level 0 是 el-tree 的虚拟根，直接列 rootPath 的内容
  const path = node.level === 0 ? props.rootPath : node.data!.path;
  try {
    const nodes: FileNode[] = await listDirectory(path);
    resolve(
      nodes.map((n) => ({
        name: n.name,
        path: n.path,
        is_dir: n.is_dir,
        has_children: n.has_children,
        leaf: !n.is_dir,
      }))
    );
  } catch (error) {
    console.error('加载目录失败：', error);
    resolve([]);
  }
}

function onNodeClick(data: TreeNode) {
  if (!data.is_dir) {
    emit('open-file', data.path);
  }
}
</script>

<style scoped>
.file-tree {
  height: 100%;
  overflow: auto;
  --el-tree-node-hover-bg-color: var(--el-fill-color-light);
}

.tree-node {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  font-size: 13px;
  color: var(--el-text-color-regular);
}

.node-icon {
  flex-shrink: 0;
  width: 16px;
  height: 16px;
  font-size: 16px;
}

.node-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
