<!--
  设置模态框
  配置 Git 路径、自动 Fetch 间隔、并发数、主题、全局快捷键
-->
<template>
  <Dialog v-model="visible" title="设置" width="540px">
    <div class="space-y-4">
      <div class="form-item">
        <label class="form-label">Git 可执行文件路径</label>
        <div class="flex items-center gap-2">
          <Input v-model="gitPath" placeholder="留空以自动检测" class="flex-1" />
          <Button variant="outline" @click="selectGitPath">浏览</Button>
        </div>
        <p class="form-tip">留空则使用系统 PATH</p>
      </div>

      <div class="form-item">
        <label class="form-label">自动获取间隔</label>
        <div class="flex items-center gap-2">
          <Input
            type="number"
            :min="10"
            :max="300"
            :step="5"
            :model-value="String(settings.auto_fetch_interval)"
            @update:model-value="(v: string) => (settings.auto_fetch_interval = clampInt(v, 10, 300, 30))"
            class="w-32"
          />
          <span class="form-tip-inline">秒（10 - 300）</span>
        </div>
      </div>

      <div class="form-item">
        <label class="form-label">最大并发 Git 操作数</label>
        <div class="flex items-center gap-2">
          <Input
            type="number"
            :min="1"
            :max="10"
            :model-value="String(settings.max_concurrent_git)"
            @update:model-value="(v: string) => (settings.max_concurrent_git = clampInt(v, 1, 10, 3))"
            class="w-32"
          />
          <span class="form-tip-inline">推荐 2 - 5</span>
        </div>
      </div>

      <div class="form-item">
        <label class="form-label">主题</label>
        <div class="inline-flex rounded-md border border-border p-0.5">
          <button
            v-for="option in themeOptions"
            :key="option"
            type="button"
            :class="[
              'rounded px-3 py-1 text-[13px] transition-colors',
              settings.theme === option
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:bg-accent',
            ]"
            @click="settings.theme = option"
          >
            {{ themeLabels[option] }}
          </button>
        </div>
      </div>

      <div class="form-item">
        <label class="form-label">全局快捷键</label>
        <Input v-model="settings.global_shortcut" placeholder="CmdOrControl+Shift+G" class="max-w-xs" />
        <p class="form-tip">示例：CmdOrControl+Shift+G</p>
      </div>

      <div class="form-item">
        <label class="form-label">批量扫描黑名单</label>
        <textarea
          v-model="blacklistText"
          rows="5"
          class="blacklist-textarea"
          placeholder="每行一个目录名，例如：&#10;node_modules&#10;target&#10;dist"
        />
        <p class="form-tip">批量导入时跳过这些目录名，每行一个</p>
      </div>
    </div>

    <template #footer>
      <Button variant="ghost" @click="visible = false">关闭</Button>
      <Button variant="primary" @click="saveSettings">保存</Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useAppStore } from '../../stores/appStore';
import { open } from '@tauri-apps/plugin-dialog';
import { toast } from '../../lib/toast';
import Dialog from '../ui/Dialog.vue';
import Input from '../ui/Input.vue';
import Button from '../ui/Button.vue';

const visible = defineModel<boolean>({ required: true });

const appStore = useAppStore();

const settings = computed({
  get: () => appStore.settings,
  set: (value) => (appStore.settings = value),
});

const themeOptions = ['system', 'light', 'dark'] as const;

const themeLabels: Record<string, string> = {
  system: '跟随系统',
  light: '浅色',
  dark: '深色',
};

const gitPath = computed({
  get: () => appStore.settings.git_path || '',
  set: (value) => (appStore.settings.git_path = value || null),
});

const blacklistText = computed({
  get: () => (appStore.settings.scan_blacklist || []).join('\n'),
  set: (value) => {
    appStore.settings.scan_blacklist = value
      .split('\n')
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
  },
});

function clampInt(v: string, min: number, max: number, fallback: number): number {
  const n = parseInt(v, 10);
  if (Number.isNaN(n)) return fallback;
  return Math.min(max, Math.max(min, n));
}

async function selectGitPath() {
  try {
    const selected = await open({
      directory: false,
      multiple: false,
      title: '选择 Git 可执行文件',
    });
    if (selected) gitPath.value = selected as string;
  } catch (err) {
    console.error('选择 Git 路径失败：', err);
  }
}

async function saveSettings() {
  try {
    await appStore.saveSettings();
    visible.value = false;
  } catch (err) {
    toast.error('保存设置失败：' + (err as Error).message);
  }
}
</script>

<style scoped>
.form-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.form-label {
  font-size: 13px;
  font-weight: 500;
}
.form-tip {
  font-size: 12px;
  line-height: 1.4;
  color: var(--muted-foreground);
}
.form-tip-inline {
  font-size: 12px;
  color: var(--muted-foreground);
}
.blacklist-textarea {
  width: 100%;
  min-height: 100px;
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid var(--input);
  background-color: var(--background);
  color: var(--foreground);
  font-size: 13px;
  line-height: 1.5;
  resize: vertical;
  font-family: var(--font-mono);
}
.blacklist-textarea:focus-visible {
  outline: none;
  border-color: var(--ring);
  box-shadow: 0 0 0 2px color-mix(in oklab, var(--ring) 30%, transparent);
}
</style>
