<!--
  状态徽章组件
  显示项目 Git 状态（干净 / 有变更 / 错误 / 未知）
-->
<template>
  <el-tag v-if="!status" size="small" type="info" effect="plain">
    未知
  </el-tag>
  <el-tooltip v-else-if="status.error" :content="status.error" placement="top">
    <el-tag size="small" type="danger" effect="dark">
      错误
    </el-tag>
  </el-tooltip>
  <el-tag v-else-if="status.is_clean" size="small" type="success" effect="plain">
    干净
  </el-tag>
  <el-tag v-else size="small" type="warning" effect="plain">
    有变更
  </el-tag>
</template>

<script setup lang="ts">
import type { ProjectStatus } from '../../types';

defineProps<{
  status: ProjectStatus | null;
}>();
</script>
