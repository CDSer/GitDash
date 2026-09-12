<!--
  设置页（系统标签，整页形式）
  顶栏：标题 + 分类 Tab（与项目标签页的模式切换样式一致）+ 保存
  分类：常规 / 外观 / 扫描
-->
<template>
  <div class="settings-page">
    <header class="settings-header">

      <div class="tab-modes" role="tablist" aria-label="设置分类">
        <button
          v-for="t in tabs"
          :key="t.value"
          type="button"
          role="tab"
          class="mode-btn"
          :class="{ 'mode-btn--active': activeTab === t.value }"
          :aria-selected="activeTab === t.value"
          @click="activeTab = t.value"
        >
          <component :is="t.icon" :size="14" />
          {{ t.label }}
        </button>
      </div>

      <div class="settings-header-right">
        <Button size="sm" variant="primary" @click="saveSettings">保存</Button>
      </div>
    </header>

    <div class="settings-body">
      <div class="settings-inner space-y-4">
        <!-- 常规 -->
        <template v-if="activeTab === 'general'">
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
                :min="0"
                :max="3600"
                :step="30"
                :model-value="String(settings.auto_fetch_interval)"
                @update:model-value="(v: string) => (settings.auto_fetch_interval = clampInt(v, 0, 3600, 600))"
                class="w-32"
              />
              <span class="form-tip-inline">秒；0 = 关闭（推荐 ≥ 300）</span>
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
            <label class="form-label">全局快捷键</label>
            <Input v-model="settings.global_shortcut" placeholder="CmdOrControl+Shift+G" class="max-w-xs" />
            <p class="form-tip">示例：CmdOrControl+Shift+G</p>
          </div>
        </template>

        <!-- 外观 -->
        <template v-else-if="activeTab === 'appearance'">
          <div class="form-item">
            <label class="form-label">主题</label>
            <div class="inline-flex rounded-lg border-[0.5px] border-border bg-muted/60 p-0.5">
              <button
                v-for="option in themeOptions"
                :key="option"
                type="button"
                :class="[
                  'rounded-[6px] px-3 py-1 text-[12px] font-medium transition-all duration-150',
                  settings.theme === option
                    ? 'bg-card text-foreground shadow-[0_0.5px_1.5px_rgba(0,0,0,0.08)]'
                    : 'text-muted-foreground hover:text-foreground',
                ]"
                @click="settings.theme = option"
              >
                {{ themeLabels[option] }}
              </button>
            </div>
          </div>

          <div class="form-item">
            <label class="form-label">皮肤</label>
            <div class="skin-grid">
              <button
                v-for="skin in skins"
                :key="skin.id"
                type="button"
                class="skin-card"
                :class="{ 'skin-card--active': settings.skin === skin.id }"
                :title="skin.description"
                @click="onSelectSkin(skin.id)"
              >
                <div class="skin-swatches">
                  <span class="skin-swatch" :style="{ backgroundColor: skin.preview.primary }" />
                  <span class="skin-swatch" :style="{ backgroundColor: skin.preview.accent }" />
                </div>
                <div class="skin-name">{{ skin.name }}</div>
                <div v-if="skin.mascot" class="skin-mascot">
                  <component :is="skin.mascot.idle" :size="28" />
                </div>
              </button>
            </div>
            <p class="form-tip">即时生效，点击保存后持久化</p>
          </div>
        </template>

        <!-- 扫描 -->
        <template v-else>
          <div class="form-item">
            <label class="form-label">批量扫描黑名单</label>
            <textarea
              v-model="blacklistText"
              rows="10"
              class="blacklist-textarea"
              placeholder="每行一个目录名，例如：&#10;node_modules&#10;target&#10;dist"
            />
            <p class="form-tip">批量导入时跳过这些目录名，每行一个</p>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { FolderSearch, Palette, SlidersHorizontal } from 'lucide-vue-next';
import { useAppStore } from '../stores/appStore';
import { open } from '@tauri-apps/plugin-dialog';
import { toast } from '../lib/toast';
import { applySkin } from '../composables/useSkin';
import { builtinSkins, skinRegistry } from '../skins';
import Input from '../components/ui/Input.vue';
import Button from '../components/ui/Button.vue';

const appStore = useAppStore();

/** 设置分类 Tab（样式与 ProjectTabView 的模式切换一致） */
const tabs = [
  { value: 'general', label: '常规', icon: SlidersHorizontal },
  { value: 'appearance', label: '外观', icon: Palette },
  { value: 'scan', label: '扫描', icon: FolderSearch },
] as const;

type SettingsTab = (typeof tabs)[number]['value'];
const activeTab = ref<SettingsTab>('general');

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

/** 内置皮肤 + 用户皮肤包（用户皮肤包由 loadUserSkins 异步注册进 registry） */
const skins = computed(() => {
  const builtinIds = new Set(builtinSkins.map((s) => s.id));
  const userSkins = Object.values(skinRegistry).filter((s) => !builtinIds.has(s.id));
  return [...builtinSkins, ...userSkins];
});

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

function onSelectSkin(id: string) {
  settings.value.skin = id;
  applySkin(id);
}

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
    toast.success('设置已保存');
  } catch (err) {
    toast.error('保存设置失败：' + (err as Error).message);
  }
}
</script>

<style scoped>
.settings-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background-color: var(--background);
}
.settings-header {
  display: flex;
  align-items: center;
  gap: 12px;
  height: 48px;
  flex-shrink: 0;
  padding: 0 16px;
  box-shadow: var(--glass-specular), inset 0 -1px 0 0 var(--separator);
  background-color: var(--toolbar-bg);
  backdrop-filter: saturate(var(--glass-saturate)) blur(var(--glass-blur));
  -webkit-backdrop-filter: saturate(var(--glass-saturate)) blur(var(--glass-blur));
}
.settings-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.settings-title {
  font-size: 13px;
  font-weight: 600;
  letter-spacing: -0.015em;
}
.settings-header-right {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

/* 分类 Tab（与 ProjectTabView 的模式切换样式一致） */
.tab-modes {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 2px;
  border-radius: 8px;
  background-color: var(--muted);
}
.mode-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 28px;
  padding: 0 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--muted-foreground);
  cursor: default;
  transition: background-color 0.15s var(--ease-out), color 0.15s var(--ease-out);
}
.mode-btn:hover {
  color: var(--foreground);
}
.mode-btn--active {
  background-color: var(--card);
  color: var(--foreground);
  box-shadow: 0 0.5px 1.5px rgba(0, 0, 0, 0.08), 0 1px 2px rgba(0, 0, 0, 0.04);
}

.settings-body {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}
.settings-inner {
  max-width: 720px;
  padding: 20px 24px 44px;
}

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
  min-height: 120px;
  padding: 8px 10px;
  border-radius: 6px;
  border: 0.5px solid var(--input);
  background-color: var(--card);
  color: var(--foreground);
  font-size: 13px;
  line-height: 1.5;
  resize: vertical;
  font-family: var(--font-mono);
}
.blacklist-textarea:focus-visible {
  outline: none;
  border-color: var(--ring);
  box-shadow: 0 0 0 3.5px color-mix(in srgb, var(--ring) 28%, transparent);
}
.skin-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
  gap: 8px;
}
.skin-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 10px 6px 8px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--card);
  cursor: default;
  transition: border-color 0.15s var(--ease-out), box-shadow 0.15s var(--ease-out);
}
.skin-card:hover {
  border-color: color-mix(in srgb, var(--muted-foreground) 40%, transparent);
}
.skin-card--active {
  border-color: var(--primary);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 22%, transparent);
}
.skin-swatches {
  display: flex;
  gap: 4px;
}
.skin-swatch {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 1px solid rgba(0, 0, 0, 0.1);
}
.skin-name {
  font-size: 12px;
  color: var(--foreground);
}
.skin-mascot {
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
