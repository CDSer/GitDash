// 皮肤 token → CSS 变量映射
// 运行时统一注入，内置皮肤与用户皮肤共用同一管线
import type { SkinColorTokens } from '../types/skin';

/** 皮肤气质别名 → 可直接用于 box-shadow 的值 */
const SHADOW_PRESETS: Record<string, string> = {
  soft: 'var(--shadow-md)',
  hard: '0 2px 0 rgba(0, 0, 0, 0.12), 0 8px 20px rgba(0, 0, 0, 0.12)',
  none: 'none',
};

export const TOKEN_CSS_VARS: Record<keyof SkinColorTokens, string> = {
  background: '--background',
  foreground: '--foreground',
  card: '--card',
  cardForeground: '--card-foreground',
  popover: '--popover',
  popoverForeground: '--popover-foreground',
  primary: '--primary',
  primaryForeground: '--primary-foreground',
  secondary: '--secondary',
  secondaryForeground: '--secondary-foreground',
  accent: '--accent',
  accentForeground: '--accent-foreground',
  muted: '--muted',
  mutedForeground: '--muted-foreground',
  destructive: '--destructive',
  destructiveForeground: '--destructive-foreground',
  border: '--border',
  input: '--input',
  ring: '--ring',
  radius: '--radius',
  shadow: '--shadow-card',
};

/** 将 tokens 写到 documentElement；未提供的 key 会先清除，回落到主题基座 */
export function applySkinTokens(tokens: SkinColorTokens | undefined): void {
  const root = document.documentElement;
  for (const cssVar of Object.values(TOKEN_CSS_VARS)) {
    root.style.removeProperty(cssVar);
  }
  if (!tokens) return;

  for (const [key, cssVar] of Object.entries(TOKEN_CSS_VARS) as Array<
    [keyof SkinColorTokens, string]
  >) {
    const raw = tokens[key];
    if (!raw) continue;
    if (key === 'shadow') {
      root.style.setProperty(cssVar, SHADOW_PRESETS[raw] ?? raw);
      continue;
    }
    root.style.setProperty(cssVar, raw);
  }
}

/** 解析后端用户皮肤 token map（保留已知 key） */
export function toSkinTokens(map: Record<string, string>): SkinColorTokens {
  const out: SkinColorTokens = {};
  const keys = Object.keys(TOKEN_CSS_VARS) as Array<keyof SkinColorTokens>;
  for (const k of keys) {
    if (map[k]) out[k] = map[k];
  }
  return out;
}
