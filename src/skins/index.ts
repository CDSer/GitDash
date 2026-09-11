// 皮肤注册表：内置皮肤在这里注册，用户皮肤包后续可动态合并
import type { SkinDef, SkinRegistry } from '../types/skin';
import { defaultSkin } from './default';
import { sparkSkin } from './spark';
import { spongeSkin } from './sponge';
import { crewSkin } from './crew';
import { catSkin } from './cat';

/** 内置皮肤列表（有序，设置面板按此顺序展示） */
export const builtinSkins: SkinDef[] = [
  defaultSkin,
  sparkSkin,
  spongeSkin,
  crewSkin,
  catSkin,
];

/** 运行时注册表：id -> SkinDef */
export const skinRegistry: SkinRegistry = Object.fromEntries(
  builtinSkins.map((s) => [s.id, s]),
);

/** 按 id 获取皮肤，未知 id 回落到 default */
export function getSkin(id: string): SkinDef {
  return skinRegistry[id] ?? skinRegistry.default;
}

/** 注册自定义皮肤（用户皮肤包 / 未来扩展） */
export function registerSkin(def: SkinDef): void {
  skinRegistry[def.id] = def;
}
