// 太空船员皮肤（致敬 Among Us 配色）
import type { SkinDef } from '../types/skin';
import CrewMascot from './components/CrewMascot.vue';
import CrewDecoration from './components/CrewDecoration.vue';

export const crewSkin: SkinDef = {
  id: 'crew',
  name: '太空船员',
  description: '船舱青 + 紧急红，冷峻科幻',
  preview: {
    primary: 'oklch(0.6 0.12 200)',
    accent: 'oklch(0.6 0.18 25)',
    cardLight: '#f0fdfa',
    cardDark: '#0c1222',
  },
  tokens: {
    light: {
      primary: 'oklch(0.6 0.12 200)',
      primaryForeground: 'oklch(0.98 0 0)',
      accent: 'oklch(0.6 0.18 25)',
      radius: '0.5rem',
    },
    dark: {
      primary: 'oklch(0.68 0.12 200)',
      primaryForeground: 'oklch(0.18 0.02 200)',
      accent: 'oklch(0.65 0.18 25)',
    },
  },
  mascot: {
    idle: CrewMascot,
    happy: CrewMascot,
    worried: CrewMascot,
    celebrating: CrewMascot,
  },
  decorations: {
    empty: CrewDecoration,
    allClear: CrewDecoration,
  },
};
