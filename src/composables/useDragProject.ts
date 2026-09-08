// 项目拖拽组合式函数
// 使用 pointer 事件模拟拖拽，绕过 Tauri Webview 中 HTML5 DnD 不可靠的问题
// 当前入口：左侧分组树中展开的项目行（拖到目标分组标题上完成移动分组）

import { ref } from 'vue';
import { useAppStore } from '../stores/appStore';
import { toast } from '../lib/toast';

const DRAG_THRESHOLD = 5;

export function useDragProject() {
  const appStore = useAppStore();
  const isDragging = ref(false);

  let dragProjectId: string | null = null;
  let ghostEl: HTMLElement | null = null;
  let startX = 0;
  let startY = 0;
  let hasMoved = false;
  let initialRect: DOMRect | null = null;
  let lastDragEndAt = 0;

  function startDrag(projectId: string, event: PointerEvent) {
    // 只响应鼠标左键
    if (event.button !== 0) return;

    dragProjectId = projectId;
    startX = event.clientX;
    startY = event.clientY;
    hasMoved = false;
    initialRect = null;

    window.addEventListener('pointermove', onPointerMove);
    window.addEventListener('pointerup', onPointerUp);
    window.addEventListener('pointercancel', onPointerUp);
  }

  function onPointerMove(event: PointerEvent) {
    if (!dragProjectId) return;

    const dx = event.clientX - startX;
    const dy = event.clientY - startY;

    if (!hasMoved) {
      if (Math.hypot(dx, dy) < DRAG_THRESHOLD) return;

      hasMoved = true;
      isDragging.value = true;
      createGhost();
    }

    if (ghostEl) {
      ghostEl.style.transform = `translate(${dx}px, ${dy}px)`;
    }

    highlightTargetGroup(event.clientX, event.clientY);
  }

  function createGhost() {
    if (!dragProjectId) return;

    // 查找被拖拽的元素：优先在 GroupTree 中查找
    const sourceEl =
      document.querySelector(`[data-project-id="${dragProjectId}"]`) ||
      document.querySelector(`[data-project-row-id="${dragProjectId}"]`);

    if (!sourceEl) return;

    initialRect = sourceEl.getBoundingClientRect();

    ghostEl = sourceEl.cloneNode(true) as HTMLElement;
    ghostEl.style.position = 'fixed';
    ghostEl.style.left = `${initialRect.left}px`;
    ghostEl.style.top = `${initialRect.top}px`;
    ghostEl.style.width = `${initialRect.width}px`;
    ghostEl.style.opacity = '0.85';
    ghostEl.style.pointerEvents = 'none';
    ghostEl.style.zIndex = '9999';
    ghostEl.style.boxShadow = '0 8px 24px rgba(0,0,0,0.25)';
    ghostEl.style.transition = 'none';

    document.body.appendChild(ghostEl);
  }

  function highlightTargetGroup(x: number, y: number) {
    document.querySelectorAll('[data-group-id]').forEach((el) => {
      el.classList.remove('group-block--drop-over');
    });

    const targetEl = document.elementFromPoint(x, y);
    const groupEl = targetEl?.closest('[data-group-id]') as HTMLElement | null;
    if (groupEl && groupEl.dataset.groupId !== 'all') {
      groupEl.classList.add('group-block--drop-over');
    }
  }

  function onPointerUp(event: PointerEvent) {
    window.removeEventListener('pointermove', onPointerMove);
    window.removeEventListener('pointerup', onPointerUp);
    window.removeEventListener('pointercancel', onPointerUp);

    if (ghostEl) {
      ghostEl.remove();
      ghostEl = null;
    }

    if (hasMoved && dragProjectId) {
      const targetEl = document.elementFromPoint(event.clientX, event.clientY);
      const groupEl = targetEl?.closest('[data-group-id]') as HTMLElement | null;

      if (groupEl) {
        const groupId = groupEl.dataset.groupId;
        if (groupId && canDropInto(groupId)) {
          dropProjectToGroup(dragProjectId, groupId);
        }
      }
    }

    document.querySelectorAll('[data-group-id]').forEach((el) => {
      el.classList.remove('group-block--drop-over');
    });

    // 记录拖拽结束时间，供点击入口判断（避免拖回原位后误触点击跳转）
    if (hasMoved) lastDragEndAt = Date.now();

    dragProjectId = null;
    hasMoved = false;
    isDragging.value = false;
    initialRect = null;
  }

  function canDropInto(groupId: string) {
    // 「全部」与系统收藏不接收放置；未分组和自定义分组都接受
    return groupId !== 'all' && groupId !== 'favorites';
  }

  function dropProjectToGroup(projectId: string, groupId: string) {
    const project = appStore.projects.find((p) => p.id === projectId);
    if (!project) return;

    const targetGroupId = groupId === 'untagged' ? null : groupId;
    if (project.group_id === targetGroupId) return;

    appStore.moveToGroup(projectId, targetGroupId);

    const groupName =
      groupId === 'untagged'
        ? '未分组'
        : appStore.groups.find((g) => g.id === groupId)?.name ?? '未分组';

    toast.success(`已将「${project.name}」移动到「${groupName}」`);

    // 展开目标分组，让用户看到反馈
    const groupBlock = document.querySelector(`[data-group-id="${groupId}"]`);
    if (groupBlock) {
      groupBlock.dispatchEvent(new CustomEvent('request-expand'));
    }
  }

  // 拖拽刚结束（350ms 内）时为 true，用于点击入口忽略拖回原位的误触发
  function wasDraggingRecently() {
    return Date.now() - lastDragEndAt < 350;
  }

  return {
    isDragging,
    startDrag,
    wasDraggingRecently,
  };
}
