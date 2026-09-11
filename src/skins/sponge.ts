// 海绵方块皮肤（致敬海绵宝宝配色）
import type { SkinDef } from '../types/skin';
import SpongeMascot from './components/SpongeMascot.vue';
import SpongeDecoration from './components/SpongeDecoration.vue';

export const spongeSkin: SkinDef = {
  id: 'sponge',
  name: '海绵方块',
  description: '海绵黄 + 海星粉，方正俏皮',
  preview: {
    primary: 'oklch(0.78 0.14 95)',
    accent: 'oklch(0.62 0.18 200)',
    cardLight: '#fefce8',
    cardDark: '#1c1917',
  },
  tokens: {
    light: {
      primary: 'oklch(0.78 0.14 95)',
      primaryForeground: 'oklch(0.2 0.05 95)',
      accent: 'oklch(0.62 0.18 200)',
      radius: '0.625rem',
    },
    dark: {
      primary: 'oklch(0.8 0.13 95)',
      primaryForeground: 'oklch(0.18 0.05 95)',
      accent: 'oklch(0.65 0.18 200)',
    },
  },
  mascot: {
    idle: SpongeMascot,
    happy: SpongeMascot,
    worried: SpongeMascot,
    celebrating: SpongeMascot,
  },
  decorations: {
    empty: SpongeDecoration,
    allClear: SpongeDecoration,
  },
};
