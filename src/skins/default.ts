// 默认皮肤（indigo，即现有 shadcn 默认）
// tokens 留空 = 完全跟随 tailwind.css 基座
import type { SkinDef } from '../types/skin';

export const defaultSkin: SkinDef = {
  id: 'default',
  name: '默认',
  description: 'GitDash 原生 indigo 主题',
  preview: {
    primary: '#007AFF',
    accent: '#E9E9EB',
    cardLight: '#ffffff',
    cardDark: '#2c2c2e',
  },
  tokens: {
    light: {},
    dark: {},
  },
};
