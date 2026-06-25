import { useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { providersApi, settingsApi, type AppId } from "@/lib/api";
import type { Provider, UsageScript } from "@/types";
import { injectCodingPlanUsageScript } from "@/config/codingPlanProviders";
import {
  useAddProviderMutation,
  useUpdateProviderMutation,
  useDeleteProviderMutation,
  useSwitchProviderMutation,
} from "@/lib/query";
import { extractErrorMessage } from "@/utils/errorUtils";

/**
 * Hook for managing provider actions (add, update, delete, switch)
 * Extracts business logic from App.tsx
 */
export function useProviderActions(activeApp: AppId, isProxyRunning?: boolean) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();

  const addProviderMutation = useAddProviderMutation(activeApp);
  const updateProviderMutation = useUpdateProviderMutation(activeApp);
  const deleteProviderMutation = useDeleteProviderMutation(activeApp);
  const switchProviderMutation = useSwitchProviderMutation(activeApp);

  // Claude 插件同步逻辑
  const syncClaudePlugin = useCallback(
    async (provider: Provider) => {
      if (activeApp !== "claude") return;

      try {
        const settings = await settingsApi.get();
        if (!settings?.enableClaudePluginIntegration) {
          return;
        }

        const isOfficial = provider.category === "official";
        await settingsApi.applyClaudePluginConfig({ official: isOfficial });

        // 静默执行，不显示成功通知
      } catch (error) {
        const detail =
          extractErrorMessage(error) ||
          t("notifications.syncClaudePluginFailed", {
            defaultValue: "同步 Claude 插件失败",
          });
        toast.error(detail, { duration: 4200 });
      }
    },
    [activeApp, t],
  );

  // 添加供应商
  const addProvider = useCallback(
    async (
      provider: Omit<Provider, "id"> & {
        providerKey?: string;
        addToLive?: boolean;
      },
    ) => {
      const enhanced = injectCodingPlanUsageScript(activeApp, provider);
      await addProviderMutation.mutateAsync(enhanced);
    },
    [addProviderMutation, activeApp],
  );

  // 更新供应商
  const updateProvider = useCallback(
    async (provider: Provider, originalId?: string) => {
      await updateProviderMutation.mutateAsync({ provider, originalId });

      // 更新托盘菜单（失败不影响主操作）
      try {
        await providersApi.updateTrayMenu();
      } catch (trayError) {
        console.error(
          "Failed to update tray menu after updating provider",
          trayError,
        );
      }
    },
    [updateProviderMutation],
  );

  // 检测供应商是否需要代理进程
  const providerNeedsProxy = useCallback(
    (provider: Provider): boolean => {
      if (activeApp === "claude") {
        // apiFormat 为 openai_chat/openai_responses 时需要代理转换
        const apiFormat = provider.meta?.apiFormat;
        if (apiFormat === "openai_chat" || apiFormat === "openai_responses") {
          return true;
        }
      }
      // isFullUrl 模式（代理用作透传网关）
      if (provider.meta?.isFullUrl) {
        return true;
      }
      return false;
    },
    [activeApp],
  );

  // 切换供应商
  const switchProvider = useCallback(
    async (provider: Provider) => {
      // 代理未运行但供应商需要代理时发出警告（不阻止切换）
      if (isProxyRunning === false && providerNeedsProxy(provider)) {
        toast.warning(
          t("notifications.proxyNotRunningWarning", {
            defaultValue: "代理服务未运行，该供应商可能无法正常工作",
          }),
        );
      }

      try {
        const result = await switchProviderMutation.mutateAsync(provider.id);
        await syncClaudePlugin(provider);

        // Show backfill warning if present
        if (result?.warnings?.length) {
          toast.warning(
            t("notifications.backfillWarning", {
              defaultValue:
                "切换成功，但旧供应商配置回填失败，您手动修改的配置可能未保存",
            }),
            { duration: 5000 },
          );
        }

        toast.success(
          t("notifications.switchSuccess", { defaultValue: "切换成功！" }),
          {
            closeButton: true,
          },
        );
      } catch {
        // 错误提示由 mutation 处理
      }
    },
    [
      switchProviderMutation,
      syncClaudePlugin,
      activeApp,
      isProxyRunning,
      providerNeedsProxy,
      t,
    ],
  );

  // 删除供应商
  const deleteProvider = useCallback(
    async (id: string) => {
      await deleteProviderMutation.mutateAsync(id);
    },
    [deleteProviderMutation],
  );

  // 保存用量脚本
  const saveUsageScript = useCallback(
    async (provider: Provider, script: UsageScript) => {
      try {
        const updatedProvider: Provider = {
          ...provider,
          meta: {
            ...provider.meta,
            usage_script: script,
          },
        };

        await providersApi.update(updatedProvider, activeApp);
        await queryClient.invalidateQueries({
          queryKey: ["providers", activeApp],
        });
        // 🔧 保存用量脚本后，也应该失效该 provider 的用量查询缓存
        // 这样主页列表会使用新配置重新查询，而不是使用测试时的缓存
        await queryClient.invalidateQueries({
          queryKey: ["usage", provider.id, activeApp],
        });
        toast.success(
          t("provider.usageSaved", {
            defaultValue: "用量查询配置已保存",
          }),
          { closeButton: true },
        );
      } catch (error) {
        const detail =
          extractErrorMessage(error) ||
          t("provider.usageSaveFailed", {
            defaultValue: "用量查询配置保存失败",
          });
        toast.error(detail);
      }
    },
    [activeApp, queryClient, t],
  );

  return {
    addProvider,
    updateProvider,
    switchProvider,
    deleteProvider,
    saveUsageScript,
    isLoading:
      addProviderMutation.isPending ||
      updateProviderMutation.isPending ||
      deleteProviderMutation.isPending ||
      switchProviderMutation.isPending,
  };
}
