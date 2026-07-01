import { invoke } from "@tauri-apps/api/core";

export const terminalApi = {
  /**
   * 在系统原生终端中打开指定目录
   * @param cwd 要打开的目录路径，如果不提供则使用当前目录
   */
  async openInTerminal(cwd?: string): Promise<boolean> {
    return await invoke("open_in_terminal", { cwd });
  },
};
