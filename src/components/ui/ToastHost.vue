<template>
  <Teleport to="body">
    <div class="toast-host">
      <div
        v-for="t in toasts"
        :key="t.id"
        :class="['toast', `toast--${t.type}`]"
      >
        <span class="toast-dot" />
        <span class="toast-msg">{{ t.message }}</span>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { toasts } from '../../lib/toast';
</script>

<style scoped>
.toast-host {
  position: fixed;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: center;
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  border-radius: var(--radius-lg);
  background-color: var(--popover);
  color: var(--popover-foreground);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-md);
  font-size: 13px;
  max-width: 80vw;
  animation: toast-in 160ms cubic-bezier(0.16, 1, 0.3, 1);
}

.toast-dot {
  width: 8px;
  height: 8px;
  border-radius: 9999px;
  flex-shrink: 0;
}

.toast--success .toast-dot {
  background-color: oklch(0.7 0.18 150);
}
.toast--error .toast-dot {
  background-color: var(--destructive);
}
.toast--info .toast-dot {
  background-color: var(--primary);
}

.toast-msg {
  white-space: pre-wrap;
}

@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
