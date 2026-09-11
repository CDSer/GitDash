// 皮肤情绪联动：根据应用状态推导吉祥物当前情绪
// 优先级：celebrating > worried > happy > idle
import { computed } from 'vue';
import { useAppStore } from '../stores/appStore';
import type { SkinMood } from '../types/skin';

export function useSkinMood() {
  const appStore = useAppStore();

  const mood = computed<SkinMood>(() => {
    const projects = appStore.projects;
    if (projects.length === 0) return 'idle';

    const statuses = appStore.statuses;
    let dirty = 0;
    let conflict = 0;
    let inProgress = 0;
    let ahead = 0;

    for (const p of projects) {
      const s = statuses.get(p.id);
      if (!s) continue;
      if (!s.is_clean) dirty++;
      if (s.conflict_count > 0) conflict++;
      if (s.in_progress) inProgress++;
      if (s.ahead > 0) ahead++;
    }

    // 冲突或进行中操作 → worried
    if (conflict > 0 || inProgress > 0) return 'worried';
    // 全部干净且无 ahead/behind → celebrating（全绿）
    if (dirty === 0 && ahead === 0) return 'celebrating';
    // 有一定提交产出（ahead > 0）→ happy
    if (ahead > 0) return 'happy';
    // 脏仓库较多 → worried
    if (dirty >= 3) return 'worried';
    return 'idle';
  });

  return { mood };
}
