// 表头列宽拖拽：宽度写入 localStorage，表头单元格右侧热区触发
import { computed, ref, onBeforeUnmount } from 'vue';

export interface ResizableColumn {
  id: string;
  /** 默认 grid track，如 '120px' / 'minmax(160px, 1.4fr)' / '1fr' */
  defaultTrack: string;
  /** 拖拽最小宽（px），默认 48 */
  min?: number;
  /** 拖拽最大宽（px），默认 800 */
  max?: number;
  /** 固定列不可拖 */
  fixed?: boolean;
}

export function useResizableColumns(storageKey: string, columns: ResizableColumn[]) {
  const widths = ref<Record<string, number>>(loadWidths(storageKey));

  const gridTemplate = computed(() =>
    columns
      .map((c) => {
        const w = widths.value[c.id];
        return Number.isFinite(w) && w! > 0 ? `${w}px` : c.defaultTrack;
      })
      .join(' '),
  );

  function loadWidths(key: string): Record<string, number> {
    try {
      const raw = localStorage.getItem(key);
      if (!raw) return {};
      const parsed = JSON.parse(raw);
      if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
        const out: Record<string, number> = {};
        for (const [k, v] of Object.entries(parsed)) {
          const n = Number(v);
          if (Number.isFinite(n) && n > 0) out[k] = n;
        }
        return out;
      }
    } catch {
      /* ignore bad storage */
    }
    return {};
  }

  function persist() {
    localStorage.setItem(storageKey, JSON.stringify(widths.value));
  }

  function resetColumn(id: string) {
    const next = { ...widths.value };
    delete next[id];
    widths.value = next;
    persist();
  }

  function resetAll() {
    widths.value = {};
    persist();
  }

  function beginResize(id: string, e: PointerEvent) {
    const col = columns.find((c) => c.id === id);
    if (!col || col.fixed) return;

    e.preventDefault();
    e.stopPropagation();

    const handle = e.currentTarget as HTMLElement;
    const cell = handle.closest('[data-col-id]') as HTMLElement | null;
    if (!cell) return;

    const startWidth = cell.getBoundingClientRect().width;
    const startX = e.clientX;
    const min = col.min ?? 48;
    const max = col.max ?? 800;
    let latest = startWidth;
    let rafId: number | null = null;

    document.body.classList.add('select-none');
    document.body.style.userSelect = 'none';
    document.body.style.cursor = 'col-resize';

    function onMove(ev: PointerEvent) {
      latest = Math.min(max, Math.max(min, startWidth + (ev.clientX - startX)));
      if (rafId === null) {
        rafId = requestAnimationFrame(() => {
          rafId = null;
          widths.value = { ...widths.value, [id]: Math.round(latest) };
        });
      }
    }

    function onUp() {
      if (rafId !== null) {
        cancelAnimationFrame(rafId);
        rafId = null;
      }
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
      widths.value = { ...widths.value, [id]: Math.round(latest) };
      persist();
      document.body.classList.remove('select-none');
      document.body.style.userSelect = '';
      document.body.style.cursor = '';
    }

    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
  }

  onBeforeUnmount(() => {
    document.body.classList.remove('select-none');
    document.body.style.userSelect = '';
    document.body.style.cursor = '';
  });

  return {
    gridTemplate,
    widths,
    beginResize,
    resetColumn,
    resetAll,
  };
}
