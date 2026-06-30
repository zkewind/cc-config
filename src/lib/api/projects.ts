import { invoke } from "@tauri-apps/api/core";

export const projectsApi = {
  async getAll(): Promise<string[]> {
    return invoke<string[]>("get_managed_project_paths");
  },

  async add(path: string): Promise<boolean> {
    return invoke<boolean>("add_managed_project_path", { path });
  },

  async remove(path: string): Promise<boolean> {
    return invoke<boolean>("remove_managed_project_path", { path });
  },

  async readSettings(projectPath: string): Promise<string | null> {
    return invoke<string | null>("read_project_settings", { projectPath });
  },

  async saveSettings(projectPath: string, content: string): Promise<void> {
    return invoke<void>("save_project_settings", { projectPath, content });
  },

  /** 同步前端当前选中的项目作用域到后端（供托盘菜单构建「当前项目」区块读取）。 */
  async setCurrentProjectScope(projectPath: string | null): Promise<void> {
    return invoke<void>("set_current_project_scope", { projectPath });
  },

  async openInExplorer(path: string): Promise<void> {
    await invoke("open_folder_in_explorer", { path });
  },
};

/** 将完整路径缩写显示：保留首段和最后一级目录，中间用 ... 替代 */
export function abbreviatePath(path: string, maxLen = 36): string {
  if (path.length <= maxLen) return path;

  const isWin = /^[A-Za-z]:\\/.test(path) || path.startsWith("\\\\");
  const sep = isWin ? "\\" : "/";
  const parts = path.split(sep).filter((p) => p.length > 0);

  if (parts.length === 0) return path;

  const last = parts[parts.length - 1];

  // 根路径前缀：Unix 为 "/"，Windows 为 "C:\" 或 "\\"
  let prefix: string;
  if (!isWin) {
    prefix = "/";
  } else if (path.startsWith("\\\\")) {
    prefix = `\\\\${parts[0]}\\`;
  } else {
    prefix = `${parts[0]}\\`;
  }

  const abbreviated = `${prefix}...${sep}${last}`;
  return abbreviated;
}

function getPathSegments(path: string): { parts: string[]; sep: string } {
  const isWin = /^[A-Za-z]:\\/.test(path) || path.startsWith("\\\\");
  const sep = isWin ? "\\" : "/";
  const parts = path.split(sep).filter((p) => p.length > 0);
  return { parts, sep };
}

function getLastSegment(path: string): string {
  const { parts } = getPathSegments(path);
  return parts[parts.length - 1] ?? path;
}

/**
 * 计算一组路径的显示标签：
 * - 末级目录名唯一 → 只显示末级目录名
 * - 末级目录名重复 → 从末级向上找到第一个能区分的层级（区分级），
 *   相邻时显示「区分级\末级」，跨级时显示「区分级\...\末级」，丢弃上层根。
 *   某条路径短于区分级所需层级（已到顶）→ 原样显示完整路径。
 */
export function computePathLabels(paths: string[]): Map<string, string> {
  const pathsByLast = new Map<string, string[]>();
  for (const p of paths) {
    const last = getLastSegment(p);
    const arr = pathsByLast.get(last) ?? [];
    arr.push(p);
    pathsByLast.set(last, arr);
  }

  const result = new Map<string, string>();

  for (const [last, group] of pathsByLast) {
    if (group.length === 1) {
      result.set(group[0], last);
      continue;
    }

    const segs = group.map((p) => {
      const { parts, sep } = getPathSegments(p);
      return { path: p, parts, sep };
    });
    const maxLen = Math.max(...segs.map((s) => s.parts.length));

    // 从末级往上找最小的 k≥2，使组内「倒数第 k 级」取值（缺失记为 ""）互不相同
    let resolveK = -1;
    for (let k = 2; k <= maxLen; k++) {
      const seen = new Set<string>();
      let distinct = true;
      for (const s of segs) {
        const idx = s.parts.length - k;
        const val = idx >= 0 ? s.parts[idx] : "";
        if (seen.has(val)) {
          distinct = false;
          break;
        }
        seen.add(val);
      }
      if (distinct) {
        resolveK = k;
        break;
      }
    }

    for (const s of segs) {
      const len = s.parts.length;
      const lastSeg = s.parts[len - 1];
      const diffIdx = len - resolveK;
      if (resolveK === -1 || diffIdx < 0) {
        // 找不到区分级，或该路径短于区分级所需层级 → 原样
        result.set(s.path, s.path);
      } else if (resolveK === 2) {
        result.set(s.path, `${s.parts[diffIdx]}${s.sep}${lastSeg}`);
      } else {
        result.set(s.path, `${s.parts[diffIdx]}${s.sep}...${s.sep}${lastSeg}`);
      }
    }
  }

  return result;
}
