// 用户皮肤包加载器
// 从后端扫描 app_data/skins/ 目录，将 skin.json 转为 SkinDef 并注册
import { defineComponent, h } from 'vue';
import type { SkinDef } from '../types/skin';
import { listUserSkins, type UserSkinPack } from '../lib/tauriApi';
import { registerSkin, builtinSkins } from './index';
import { toSkinTokens } from './tokens';
import SvgMascot from './components/SvgMascot.vue';

/** 将原始 SVG 字符串包成一个可渲染的 Vue 组件 */
function svgToComponent(svg: string | null | undefined) {
  if (!svg) return undefined;
  return defineComponent({
    name: 'UserSvgAsset',
    props: {
      size: { type: Number, default: 32 },
    },
    setup(props) {
      return () =>
        h('div', {
          style: {
            width: props.size + 'px',
            height: props.size + 'px',
            display: 'block',
          },
          innerHTML: svg,
        });
    },
  });
}

/** 将 UserSkinPack 转为 SkinDef */
function packToSkinDef(pack: UserSkinPack): SkinDef {
  const mascotComp = svgToComponent(pack.mascot_svg);
  const decoComp = svgToComponent(pack.decoration_svg);

  const def: SkinDef = {
    id: pack.id,
    name: pack.name,
    description: pack.description ?? undefined,
    author: pack.author ?? undefined,
    version: pack.version ?? undefined,
    preview: {
      primary: pack.preview.primary,
      accent: pack.preview.accent,
      cardLight: pack.preview.cardLight,
      cardDark: pack.preview.cardDark,
    },
    tokens: {
      light: toSkinTokens(pack.tokens.light),
      dark: toSkinTokens(pack.tokens.dark),
    },
  };

  if (mascotComp) {
    def.mascot = {
      idle: mascotComp,
      happy: mascotComp,
      worried: mascotComp,
      celebrating: mascotComp,
    };
  }

  if (decoComp) {
    def.decorations = {
      empty: decoComp,
      allClear: decoComp,
    };
  }

  return def;
}

/**
 * 加载用户皮肤包并注册到 skinRegistry
 * 失败时静默返回（内置皮肤不受影响）
 */
export async function loadUserSkins(): Promise<void> {
  try {
    const packs = await listUserSkins();
    const builtinIds = new Set(builtinSkins.map((s) => s.id));
    for (const pack of packs) {
      // 用户皮肤不允许覆盖内置 id
      if (builtinIds.has(pack.id)) continue;
      registerSkin(packToSkinDef(pack));
    }
  } catch (err) {
    console.warn('加载用户皮肤包失败：', err);
  }
}

export { SvgMascot };
