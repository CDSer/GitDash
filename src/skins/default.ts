// 默认皮肤（indigo，即现有 shadcn 默认）
import type { SkinDef } from '../types/skin';

export const defaultSkin: SkinDef = {
  id: 'default',
  name: '默认',
  description: 'GitDash 原生 indigo 主题',
  preview: {
    primary: 'oklch(0.55 0.2 264)',
    accent: 'oklch(0.94 0.01 260)',
    cardLight: '#ffffff',
    cardDark: '#1e2028',
  },
  tokens: {
    light: {},
    dark: {},
  },
};
