<!--
  应用根组件
  提供 Element Plus 全局配置（中文语言包），并挂载页面视图
-->
<template>
  <el-config-provider :locale="zhCn">
    <MainView />
  </el-config-provider>
</template>

<script setup lang="ts">
import zhCn from 'element-plus/es/locale/lang/zh-cn';
import MainView from './views/MainView.vue';
</script>
