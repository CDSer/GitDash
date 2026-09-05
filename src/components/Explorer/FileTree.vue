<!--
  文件树组件（自绘懒加载树，替代 Element Plus el-tree）
  - props.rootPath：项目根目录绝对路径
  - emit('open-file', path)：点击文件时抛出文件路径
  后端依赖：list_directory 命令（已在 src-tauri 中实现）
  图标：使用 @iconify/vue + vscode-icons 图标集
-->
<template>
  <div class="file-tree">
    <FileTreeNode
      v-for="node in rootNodes"
      :key="node.path"
      :node="node"
      :depth="0"
      @open-file="(p: string) => emit('open-file', p)"
    />
    <div v-if="!rootNodes.length && loaded" class="tree-empty">空目录</div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, reactive } from 'vue';
import { listDirectory } from '../../lib/tauriApi';
import type { FileNode } from '../../types';
import FileTreeNode from './FileTreeNode.vue';

interface TreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  has_children: boolean;
  expanded: boolean;
  loaded: boolean;
  children: TreeNode[];
}

const props = defineProps<{ rootPath: string }>();
const emit = defineEmits<{ (e: 'open-file', path: string): void }>();

const rootNodes = ref<TreeNode[]>([]);
const loaded = ref(false);

onMounted(async () => {
  try {
    const list: FileNode[] = await listDirectory(props.rootPath);
    rootNodes.value = list.map((n) =>
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
    console.error('加载根目录失败：', error);
  } finally {
    loaded.value = true;
  }
});
</script>

<style scoped>
.file-tree {
  height: 100%;
  overflow: auto;
  padding: 6px 4px;
}
.tree-empty {
  padding: 16px;
  text-align: center;
  font-size: 12px;
  color: var(--muted-foreground);
}
</style>
