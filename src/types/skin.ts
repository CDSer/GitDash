// GitDash 皮肤包类型定义
// 皮肤 = 色板覆盖 + 吉祥物 + 装饰插画，与 light/dark 模式正交

import type { Component } from 'vue';

/** 可被皮肤覆盖的 CSS 变量（oklch 字符串或任意 CSS 颜色） */
export interface SkinColorTokens {
  background?: string;
  foreground?: string;
  card?: string;
  cardForeground?: string;
  popover?: string;
  popoverForeground?: string;
  primary?: string;
  primaryForeground?: string;
  secondary?: string;
  secondaryForeground?: string;
  accent?: string;
  accentForeground?: string;
  muted?: string;
  mutedForeground?: string;
  destructive?: string;
  destructiveForeground?: string;
  border?: string;
  input?: string;
  ring?: string;
  radius?: string;
  /** 阴影气质：soft / hard / none，或直接写 CSS box-shadow */
  shadow?: string;
}

export type SkinMood = 'idle' | 'happy' | 'worried' | 'celebrating';

/** 吉祥物：一组按情绪切换的 Vue 组件（均为纯 SVG，无外部依赖） */
export interface SkinMascotAssets {
  /** 默认 / 待机 */
  idle: Component;
  /** 可选情绪变体，缺省时回落到 idle */
  happy?: Component;
  worried?: Component;
  celebrating?: Component;
}

/** 大尺寸装饰插画（空状态 / Loading 等场景） */
export interface SkinDecorationAssets {
  empty?: Component;
  allClear?: Component;
}

/** 皮肤定义 */
export interface SkinDef {
  /** 唯一 id，同时作为 data-skin 属性值与 CSS 选择器 */
  id: string;
  /** 显示名称 */
  name: string;
  /** 一句话描述 */
  description?: string;
  /** 作者（用户皮肤包用） */
  author?: string;
  /** 版本号（用户皮肤包用） */
  version?: string;
  /** 设置面板预览色块 */
  preview: {
    /** 主色（用于色块 / 预览卡背景） */
    primary: string;
    /** 强调色 */
    accent: string;
    /** 浅色模式下的卡片背景 */
    cardLight: string;
    /** 深色模式下的卡片背景 */
    cardDark: string;
  };
  /** light / dark 各自的色板覆盖 */
  tokens: {
    light: SkinColorTokens;
    dark: SkinColorTokens;
  };
  /** 吉祥物（侧栏顶部 + 可选空状态） */
  mascot?: SkinMascotAssets;
  /** 大尺寸装饰插画 */
  decorations?: SkinDecorationAssets;
  /**
   * 附加 CSS：写在 [data-skin="id"] 作用域下的自定义规则
   * （如特殊阴影、字体、动效曲线）
   */
  extraCss?: string;
}

/** 皮肤注册表：id -> SkinDef */
export type SkinRegistry = Record<string, SkinDef>;
