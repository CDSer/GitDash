<!--
  设置模态框
  配置 Git 路径、自动 Fetch 间隔、并发数、主题、全局快捷键
-->
<template>
  <el-dialog
    v-model="visible"
    title="设置"
    width="540px"
    append-to-body
  >
    <el-form label-width="150px" @submit.prevent>
      <el-form-item label="Git 可执行文件路径">
        <div class="git-path-row">
          <el-input v-model="gitPath" placeholder="留空以自动检测" />
          <el-button @click="selectGitPath">浏览</el-button>
        </div>
        <div class="form-tip">留空则使用系统 PATH</div>
      </el-form-item>

      <el-form-item label="自动获取间隔">
        <el-input-number
          v-model="settings.auto_fetch_interval"
          :min="10"
          :max="300"
          :step="5"
        />
        <span class="form-tip-inline">秒（10 - 300）</span>
      </el-form-item>

      <el-form-item label="最大并发 Git 操作数">
        <el-input-number
          v-model="settings.max_concurrent_git"
          :min="1"
          :max="10"
        />
        <span class="form-tip-inline">推荐 2 - 5</span>
      </el-form-item>

      <el-form-item label="主题">
        <el-radio-group :model-value="settings.theme" @change="handleThemeChange">
          <el-radio-button v-for="option in themeOptions" :key="option" :value="option">
            {{ themeLabels[option] }}
          </el-radio-button>
        </el-radio-group>
      </el-form-item>

      <el-form-item label="全局快捷键">
        <el-input v-model="settings.global_shortcut" placeholder="CmdOrControl+Shift+G" />
        <div class="form-tip">示例：CmdOrControl+Shift+G</div>
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="visible = false">关闭</el-button>
      <el-button type="primary" @click="saveSettings">保存</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Settings } from '../../types';
import { useAppStore } from '../../stores/appStore';
import { open } from '@tauri-apps/plugin-dialog';

const visible = defineModel<boolean>({ required: true });

const appStore = useAppStore();

const settings = computed({
  get: () => appStore.settings,
  set: (value) => appStore.settings = value
});

const themeOptions = ['system', 'light', 'dark'] as const;

const themeLabels: Record<string, string> = {
  system: '跟随系统',
  light: '浅色',
  dark: '深色',
};

const gitPath = computed({
  get: () => appStore.settings.git_path || '',
  set: (value) => appStore.settings.git_path = value || null
});

async function selectGitPath() {
  try {
    const selected = await open({
      directory: false,
      multiple: false,
      title: '选择 Git 可执行文件'
    });

    if (selected) {
      gitPath.value = selected as string;
    }
  } catch (err) {
    console.error('选择 Git 路径失败：', err);
  }
}

function handleThemeChange(value?: string | number | boolean) {
  settings.value.theme = value as Settings['theme'];
}

function saveSettings() {
  // 设置项的持久化需要后端 update_settings 命令，尚未接入
  visible.value = false;
}
</script>

<style scoped>
.git-path-row {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.form-tip {
  margin-top: 4px;
  font-size: 12px;
  line-height: 1.4;
  color: var(--el-text-color-secondary);
}

.form-tip-inline {
  margin-left: 8px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>
