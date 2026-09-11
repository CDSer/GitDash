// 皮肤运行时：将当前皮肤 id 应用到 <html data-skin="...">，
// 并 watch settings.skin 变化自动切换
import { watch } from 'vue';
import { useAppStore } from '../stores/appStore';
import { getSkin, skinRegistry } from '../skins';
import { loadUserSkins } from '../skins/loadUserSkins';

export function applySkin(id: string): void {
  const skin = getSkin(id);
  document.documentElement.dataset.skin = skin.id;
}

export function setupSkin(): void {
  const appStore = useAppStore();

  // 初始应用（config 可能还没加载，用当前 settings 值兜底）
  applySkin(appStore.settings.skin || 'default');

  watch(
    () => appStore.settings.skin,
    (skinId) => {
      applySkin(skinId || 'default');
    },
  );

  // 异步加载用户皮肤包（不阻塞启动）
  loadUserSkins().then(() => {
    // 加载后重新应用当前皮肤（可能用户皮肤包里有同 id？已过滤内置 id，这里只是确保生效）
    applySkin(appStore.settings.skin || 'default');
  });
}

export { skinRegistry };
