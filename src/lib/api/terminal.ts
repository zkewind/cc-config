import { invoke } from "@tauri-apps/api/core";

export const terminalApi = {
  /**
   * 在系统原生终端中打开指定目录
   * @param cwd 要打开的目录路径，如果不提供则使用当前目录
   */
  async openInTerminal(cwd?: string): Promise<boolean> {
    return await invoke("open_in_terminal", { cwd });
  },

  /**
   * 在用户首选终端中用 Claude Code 打开指定项目目录。
   * @param cwd 项目目录路径
   * @param skipPermissions 为 true 时附带 --dangerously-skip-permissions
   */
  async openClaudeInTerminal(
    cwd: string,
    skipPermissions: boolean,
  ): Promise<boolean> {
    return await invoke("open_claude_in_terminal", { cwd, skipPermissions });
  },
};
