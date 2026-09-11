// 皮肤运行时：将当前皮肤 id 应用到 <html data-skin="...">，
// 并把 tokens 注入为 CSS 变量；watch settings.skin / theme 自动切换
import { watch } from 'vue';
import { useAppStore } from '../stores/appStore';
import { getSkin, skinRegistry } from '../skins';
import { loadUserSkins } from '../skins/loadUserSkins';
import { applySkinTokens } from '../skins/tokens';

function isDarkMode(): boolean {
  return document.documentElement.classList.contains('dark');
}

export function applySkin(id: string): void {
  const skin = getSkin(id);
  document.documentElement.dataset.skin = skin.id;
  const tokens = isDarkMode() ? skin.tokens.dark : skin.tokens.light;
  applySkinTokens(tokens);
}

export function setupSkin(): void {
  const appStore = useAppStore();

  applySkin(appStore.settings.skin || 'default');

  watch(
    () => [appStore.settings.skin, appStore.settings.theme] as const,
    () => {
      applySkin(appStore.settings.skin || 'default');
    },
  );

  // system 主题：系统深浅切换时 theme 类由 useTheme 更新，这里补一次 tokens
  const mq = window.matchMedia('(prefers-color-scheme: dark)');
  const onSystemChange = () => {
    if (appStore.settings.theme === 'system') {
      applySkin(appStore.settings.skin || 'default');
    }
  };
  mq.addEventListener('change', onSystemChange);

  loadUserSkins().then(() => {
    applySkin(appStore.settings.skin || 'default');
  });
}

export { skinRegistry };
