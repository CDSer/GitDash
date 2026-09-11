// 猫咪皮肤（暖橙轻拟物风格，参考倒计时/生活类素材的暖橙主色 + 深蓝底色）
import type { SkinDef } from '../types/skin';
import CatMascot from './components/CatMascot.vue';
import CatDecoration from './components/CatDecoration.vue';

export const catSkin: SkinDef = {
  id: 'cat',
  name: '猫咪',
  description: '暖橘主色 + 肉垫粉点缀，慵懒治愈',
  preview: {
    primary: 'oklch(0.72 0.15 60)',
    accent: 'oklch(0.86 0.09 5)',
    cardLight: '#fff7ed',
    cardDark: '#1c1917',
  },
  tokens: {
    light: {
      primary: 'oklch(0.72 0.15 60)',
      primaryForeground: 'oklch(0.25 0.05 50)',
      secondary: 'oklch(0.94 0.03 70)',
      secondaryForeground: 'oklch(0.35 0.05 55)',
      accent: 'oklch(0.86 0.09 5)',
      accentForeground: 'oklch(0.3 0.08 5)',
      muted: 'oklch(0.95 0.02 70)',
      mutedForeground: 'oklch(0.5 0.04 60)',
      border: 'oklch(0.9 0.03 65)',
      ring: 'oklch(0.72 0.15 60)',
      radius: '0.75rem',
    },
    dark: {
      primary: 'oklch(0.78 0.14 60)',
      primaryForeground: 'oklch(0.22 0.04 50)',
      secondary: 'oklch(0.3 0.04 55)',
      secondaryForeground: 'oklch(0.9 0.03 65)',
      accent: 'oklch(0.7 0.1 5)',
      accentForeground: 'oklch(0.25 0.06 5)',
      muted: 'oklch(0.28 0.03 55)',
      mutedForeground: 'oklch(0.7 0.04 65)',
      border: 'oklch(1 0 0 / 12%)',
      ring: 'oklch(0.78 0.14 60)',
    },
  },
  mascot: {
    idle: CatMascot,
    happy: CatMascot,
    worried: CatMascot,
    celebrating: CatMascot,
  },
  decorations: {
    empty: CatDecoration,
    allClear: CatDecoration,
  },
};