// 电气鼠皮肤（致敬皮卡丘配色）
import type { SkinDef } from '../types/skin';
import SparkMascot from './components/SparkMascot.vue';
import SparkDecoration from './components/SparkDecoration.vue';

export const sparkSkin: SkinDef = {
  id: 'spark',
  name: '电气鼠',
  description: '明黄主色 + 腮红点缀，圆润活泼',
  preview: {
    primary: 'oklch(0.82 0.15 85)',
    accent: 'oklch(0.62 0.18 25)',
    cardLight: '#fffbeb',
    cardDark: '#1c1917',
  },
  tokens: {
    light: {
      primary: 'oklch(0.82 0.15 85)',
      primaryForeground: 'oklch(0.2 0.05 85)',
      accent: 'oklch(0.62 0.18 25)',
      radius: '0.75rem',
    },
    dark: {
      primary: 'oklch(0.85 0.14 85)',
      primaryForeground: 'oklch(0.18 0.05 85)',
      accent: 'oklch(0.65 0.18 25)',
    },
  },
  mascot: {
    idle: SparkMascot,
    happy: SparkMascot,
    worried: SparkMascot,
    celebrating: SparkMascot,
  },
  decorations: {
    empty: SparkDecoration,
    allClear: SparkDecoration,
  },
};
