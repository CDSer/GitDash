<!--
  代码编辑器组件（基于 vue-codemirror / CodeMirror 6）
  - props.modelValue：文件内容
  - props.language：扩展名或语言标识（用于选择高亮语言）
  - emit('update:modelValue', value)：内容变化
  - emit('save')：Ctrl/Cmd+S 保存（已拦截浏览器默认保存）
  注意：one-dark 主题固定，后续可跟随应用主题切换
-->
<template>
  <Codemirror
    :model-value="modelValue"
    :extensions="extensions"
    :style="{ height: '100%' }"
    @update:model-value="onInput"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Codemirror } from 'vue-codemirror';
import { keymap } from '@codemirror/view';
import { Prec, type Extension } from '@codemirror/state';
import { StreamLanguage } from '@codemirror/language';
import { oneDark } from '@codemirror/theme-one-dark';

// 完整语法（Lezer）
import { javascript } from '@codemirror/lang-javascript';
import { json } from '@codemirror/lang-json';
import { markdown } from '@codemirror/lang-markdown';
import { html } from '@codemirror/lang-html';
import { css } from '@codemirror/lang-css';
import { rust } from '@codemirror/lang-rust';
import { vue } from '@codemirror/lang-vue';

// 流式语法（legacy-modes）
import { python } from '@codemirror/legacy-modes/mode/python';
import { go } from '@codemirror/legacy-modes/mode/go';
import { shell } from '@codemirror/legacy-modes/mode/shell';
import { yaml } from '@codemirror/legacy-modes/mode/yaml';
import { toml } from '@codemirror/legacy-modes/mode/toml';
import { xml } from '@codemirror/legacy-modes/mode/xml';
import { ruby } from '@codemirror/legacy-modes/mode/ruby';
import { swift } from '@codemirror/legacy-modes/mode/swift';
import { lua } from '@codemirror/legacy-modes/mode/lua';
import { r } from '@codemirror/legacy-modes/mode/r';
import { perl } from '@codemirror/legacy-modes/mode/perl';
import { powerShell } from '@codemirror/legacy-modes/mode/powershell';
import { properties } from '@codemirror/legacy-modes/mode/properties';
import { nginx } from '@codemirror/legacy-modes/mode/nginx';
import { http } from '@codemirror/legacy-modes/mode/http';
import { dockerFile } from '@codemirror/legacy-modes/mode/dockerfile';
import { protobuf } from '@codemirror/legacy-modes/mode/protobuf';
import { julia } from '@codemirror/legacy-modes/mode/julia';
import { haskell } from '@codemirror/legacy-modes/mode/haskell';
import { elm } from '@codemirror/legacy-modes/mode/elm';
import { cmake } from '@codemirror/legacy-modes/mode/cmake';
import { pug } from '@codemirror/legacy-modes/mode/pug';
import { jinja2 } from '@codemirror/legacy-modes/mode/jinja2';
import { clojure } from '@codemirror/legacy-modes/mode/clojure';
import { coffeeScript } from '@codemirror/legacy-modes/mode/coffeescript';
import { crystal } from '@codemirror/legacy-modes/mode/crystal';
import { diff } from '@codemirror/legacy-modes/mode/diff';
import { erlang } from '@codemirror/legacy-modes/mode/erlang';
import { fortran } from '@codemirror/legacy-modes/mode/fortran';
import { groovy } from '@codemirror/legacy-modes/mode/groovy';
import { stylus } from '@codemirror/legacy-modes/mode/stylus';
import { sass } from '@codemirror/legacy-modes/mode/sass';
import { standardSQL } from '@codemirror/legacy-modes/mode/sql';
import { vb } from '@codemirror/legacy-modes/mode/vb';
import { tcl } from '@codemirror/legacy-modes/mode/tcl';
import { pascal } from '@codemirror/legacy-modes/mode/pascal';
import { verilog } from '@codemirror/legacy-modes/mode/verilog';
import { vhdl } from '@codemirror/legacy-modes/mode/vhdl';
import { scheme } from '@codemirror/legacy-modes/mode/scheme';
import {
  c,
  cpp,
  java,
  csharp,
  scala,
  kotlin,
  dart,
  objectiveC,
} from '@codemirror/legacy-modes/mode/clike';

const props = defineProps<{
  modelValue: string;
  language: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'save'): void;
}>();

const stream = (parser: Parameters<typeof StreamLanguage.define>[0]): (() => Extension) =>
  () => StreamLanguage.define(parser);

const js = () => javascript();
const ts = () => javascript({ typescript: true });
const jsx = () => javascript({ jsx: true });
const tsx = () => javascript({ jsx: true, typescript: true });

/**
 * 语言键 → 扩展工厂
 * 键为小写扩展名 / 文件名 / 常见别名
 */
const langMap: Record<string, () => Extension> = {
  // JS / TS
  js: js,
  mjs: js,
  cjs: js,
  es6: js,
  node: js,
  ts: ts,
  mts: ts,
  cts: ts,
  dts: ts,
  jsx: jsx,
  tsx: tsx,
  // 数据 / 标记
  json: () => json(),
  jsonc: () => json(),
  json5: () => json(),
  md: () => markdown(),
  markdown: () => markdown(),
  mdx: () => markdown(),
  html: () => html(),
  htm: () => html(),
  vue: () => vue(),
  css: () => css(),
  scss: () => css(),
  less: () => css(),
  sass: stream(sass),
  styl: stream(stylus),
  stylus: stream(stylus),
  xml: stream(xml),
  svg: stream(xml),
  xsl: stream(xml),
  xsd: stream(xml),
  xsli: stream(xml),
  rss: stream(xml),
  yaml: stream(yaml),
  yml: stream(yaml),
  toml: stream(toml),
  ini: stream(properties),
  conf: stream(properties),
  cfg: stream(properties),
  properties: stream(properties),
  env: stream(properties),
  // 系统 / 配置
  sh: stream(shell),
  bash: stream(shell),
  zsh: stream(shell),
  fish: stream(shell),
  ksh: stream(shell),
  csh: stream(shell),
  tcsh: stream(shell),
  ps1: stream(powerShell),
  psm1: stream(powerShell),
  bat: stream(properties),
  cmd: stream(properties),
  dockerfile: stream(dockerFile),
  docker: stream(dockerFile),
  makefile: stream(shell),
  mak: stream(shell),
  mk: stream(shell),
  cmake: stream(cmake),
  nginx: stream(nginx),
  http: stream(http),
  graphql: stream(protobuf),
  proto: stream(protobuf),
  protobuf: stream(protobuf),
  // C 家族 / JVM
  c: stream(c),
  h: stream(c),
  cc: stream(cpp),
  cpp: stream(cpp),
  cxx: stream(cpp),
  'c++': stream(cpp),
  hpp: stream(cpp),
  hh: stream(cpp),
  hxx: stream(cpp),
  m: stream(objectiveC),
  mm: stream(objectiveC),
  java: stream(java),
  kt: stream(kotlin),
  kts: stream(kotlin),
  scala: stream(scala),
  sc: stream(scala),
  cs: stream(csharp),
  dart: stream(dart),
  groovy: stream(groovy),
  gradle: stream(groovy),
  // 脚本 / 后端
  py: stream(python),
  pyw: stream(python),
  pyi: stream(python),
  pyx: stream(python),
  rb: stream(ruby),
  erb: stream(ruby),
  rake: stream(ruby),
  gemfile: stream(ruby),
  php: () => StreamLanguage.define(shell),
  php3: () => StreamLanguage.define(shell),
  php4: () => StreamLanguage.define(shell),
  php5: () => StreamLanguage.define(shell),
  php7: () => StreamLanguage.define(shell),
  phps: () => StreamLanguage.define(shell),
  phtml: () => StreamLanguage.define(shell),
  pl: stream(perl),
  pm: stream(perl),
  pod: stream(perl),
  lua: stream(lua),
  r: stream(r),
  R: stream(r),
  swift: stream(swift),
  rs: () => rust(),
  go: stream(go),
  hs: stream(haskell),
  lhs: stream(haskell),
  elm: stream(elm),
  erl: stream(erlang),
  hrl: stream(erlang),
  jl: stream(julia),
  f: stream(fortran),
  f90: stream(fortran),
  f95: stream(fortran),
  for: stream(fortran),
  vb: stream(vb),
  vbs: stream(vb),
  vba: stream(vb),
  pas: stream(pascal),
  p: stream(pascal),
  v: stream(verilog),
  vhd: stream(vhdl),
  vhdl: stream(vhdl),
  scm: stream(scheme),
  ss: stream(scheme),
  lisp: stream(scheme),
  tcl: stream(tcl),
  // 模板
  pug: stream(pug),
  jade: stream(pug),
  jinja: stream(jinja2),
  jinja2: stream(jinja2),
  tpl: stream(jinja2),
  twig: stream(jinja2),
  // 其它
  sql: stream(standardSQL),
  mysql: stream(standardSQL),
  psql: stream(standardSQL),
  db: stream(standardSQL),
  diff: stream(diff),
  patch: stream(diff),
  clj: stream(clojure),
  cljs: stream(clojure),
  coffee: stream(coffeeScript),
  crystal: stream(crystal),
};

const extensions = computed<Extension[]>(() => {
  const key = normalizeLangKey(props.language);
  const factory = langMap[key];
  const langExt: Extension[] = factory ? [factory()] : [];

  const saveKeymap = keymap.of([
    {
      key: 'Mod-s',
      preventDefault: true,
      run: () => {
        emit('save');
        return true;
      },
    },
  ]);

  return [Prec.highest(saveKeymap), ...langExt, oneDark];
});

function normalizeLangKey(raw: string): string {
  if (!raw) return '';
  const s = raw.trim().toLowerCase();
  if (s.startsWith('.')) return s.slice(1);
  return s;
}

function onInput(value: string) {
  emit('update:modelValue', value);
}
</script>

<style scoped>
:deep(.cm-editor) {
  height: 100%;
  font-size: 13px;
}

:deep(.cm-scroller) {
  overflow: auto;
  font-family: 'JetBrains Mono', 'Fira Code', Menlo, Consolas, monospace;
}
</style>
