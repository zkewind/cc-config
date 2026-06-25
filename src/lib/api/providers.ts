import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Provider } from "@/types";
import type { AppId } from "./types";

export interface ProviderSortUpdate {
  id: string;
  sortIndex: number;
}

export interface ProviderSwitchEvent {
  appType: AppId;
  providerId: string;
}

export interface SwitchResult {
  warnings: string[];
}

export interface OpenTerminalOptions {
  cwd?: string;
}

export const providersApi = {
  async getAll(appId: AppId): Promise<Record<string, Provider>> {
    return await invoke("get_providers", { app: appId });
  },

  async getCurrent(appId: AppId): Promise<string> {
    return await invoke("get_current_provider", { app: appId });
  },

  async add(
    provider: Provider,
    appId: AppId,
    addToLive?: boolean,
  ): Promise<boolean> {
    return await invoke("add_provider", { provider, app: appId, addToLive });
  },

  async update(
    provider: Provider,
    appId: AppId,
    originalId?: string,
  ): Promise<boolean> {
    return await invoke("update_provider", {
      provider,
      app: appId,
      originalId,
    });
  },

  async delete(id: string, appId: AppId): Promise<boolean> {
    return await invoke("delete_provider", { id, app: appId });
  },

  async removeFromLiveConfig(id: string, appId: AppId): Promise<boolean> {
    return await invoke("remove_provider_from_live_config", { id, app: appId });
  },

  async switch(id: string, appId: AppId): Promise<SwitchResult> {
    return await invoke("switch_provider", { id, app: appId });
  },

  async importDefault(appId: AppId): Promise<boolean> {
    return await invoke("import_default_config", { app: appId });
  },

  async updateTrayMenu(): Promise<boolean> {
    return await invoke("update_tray_menu");
  },

  async updateSortOrder(
    updates: ProviderSortUpdate[],
    appId: AppId,
  ): Promise<boolean> {
    return await invoke("update_providers_sort_order", { updates, app: appId });
  },

  async onSwitched(
    handler: (event: ProviderSwitchEvent) => void,
  ): Promise<UnlistenFn> {
    return await listen("provider-switched", (event) => {
      const payload = event.payload as ProviderSwitchEvent;
      handler(payload);
    });
  },

  async openTerminal(
    providerId: string,
    appId: AppId,
    options?: OpenTerminalOptions,
  ): Promise<boolean> {
    const { cwd } = options ?? {};
    return await invoke("open_provider_terminal", {
      providerId,
      app: appId,
      cwd,
    });
  },

  async switchForProject(
    providerId: string,
    appId: AppId,
    projectPath: string,
  ): Promise<SwitchResult> {
    return await invoke("switch_provider_for_project", {
      id: providerId,
      app: appId,
      projectPath,
    });
  },

  async getCurrentForProject(
    appId: AppId,
    projectPath: string,
  ): Promise<string> {
    return await invoke("get_current_provider_for_project", {
      app: appId,
      projectPath,
    });
  },

  async importFromProject(appId: AppId, projectPath: string): Promise<string> {
    return await invoke("import_project_provider", {
      app: appId,
      projectPath,
    });
  },

  async resolveForProject(appId: AppId, projectPath: string): Promise<string> {
    return await invoke("resolve_current_provider_for_project", {
      app: appId,
      projectPath,
    });
  },

  async exportJson(appId: AppId, filePath: string): Promise<void> {
    await invoke("export_providers_json", { app: appId, filePath });
  },

  async regenerateName(id: string, appId: AppId): Promise<string> {
    return await invoke("regenerate_provider_name", {
      app: appId,
      providerId: id,
    });
  },
};
