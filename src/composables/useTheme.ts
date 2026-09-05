// 主题切换：根据 settings.theme（system/light/dark）在 <html> 上切换 .dark 类
import { watch } from 'vue';
import { useAppStore } from '../stores/appStore';

function systemPrefersDark(): boolean {
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

export function applyTheme(theme: 'system' | 'light' | 'dark'): void {
  const dark = theme === 'dark' || (theme === 'system' && systemPrefersDark());
  document.documentElement.classList.toggle('dark', dark);
}

export function setupTheme(): void {
  const appStore = useAppStore();
  const mq = window.matchMedia('(prefers-color-scheme: dark)');

  const onSystemChange = () => {
    if (appStore.settings.theme === 'system') applyTheme('system');
  };
  mq.addEventListener('change', onSystemChange);

  watch(
    () => appStore.settings.theme,
    (t) => applyTheme(t),
  );

  applyTheme(appStore.settings.theme);
}
