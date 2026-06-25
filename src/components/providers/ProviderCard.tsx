import { useMemo } from "react";
import { GripVertical, Wand2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import type {
  DraggableAttributes,
  DraggableSyntheticListeners,
} from "@dnd-kit/core";
import type { Provider } from "@/types";
import type { AppId } from "@/lib/api";
import { cn } from "@/lib/utils";
import { ProviderIcon } from "@/components/ProviderIcon";
import {
  useProviderStatus,
  ProviderStatusIndicator,
  ProviderStatusExpandedPanel,
} from "@/components/providers/ProviderStatusIndicator";
import { ProviderCardMenu } from "@/components/providers/ProviderCardMenu";
import { ProviderSwitchButton } from "@/components/providers/ProviderActions";

interface DragHandleProps {
  attributes: DraggableAttributes;
  listeners: DraggableSyntheticListeners;
  isDragging: boolean;
}

interface ProviderCardProps {
  provider: Provider;
  isCurrent: boolean;
  appId: AppId;
  isInConfig?: boolean;
  onSwitch: (provider: Provider) => void;
  onEdit: (provider: Provider) => void;
  onDelete: (provider: Provider) => void;
  onRemoveFromConfig?: (provider: Provider) => void;
  onConfigureUsage: (provider: Provider) => void;
  onOpenWebsite: (url: string) => void;
  onDuplicate: (provider: Provider) => void;
  onRegenerateName?: (provider: Provider) => void;
  onTest?: (provider: Provider) => void;
  onOpenTerminal?: (provider: Provider) => void;
  isTesting?: boolean;
  dragHandleProps?: DragHandleProps;
}

function isOfficialProvider(provider: Provider, appId: AppId): boolean {
  if (provider.category === "official") {
    return true;
  }
  const config = provider.settingsConfig as Record<string, any>;
  if (appId === "claude") {
    const baseUrl = config?.env?.ANTHROPIC_BASE_URL;
    return !baseUrl || (typeof baseUrl === "string" && baseUrl.trim() === "");
  }
  return false;
}

const extractApiUrl = (provider: Provider, fallbackText: string) => {
  if (provider.notes?.trim()) {
    return provider.notes.trim();
  }
  if (provider.websiteUrl) {
    return provider.websiteUrl;
  }
  const config = provider.settingsConfig;
  if (config && typeof config === "object") {
    const envBase = (config as Record<string, any>)?.env?.ANTHROPIC_BASE_URL;
    if (typeof envBase === "string" && envBase.trim()) {
      return envBase;
    }
  }
  return fallbackText;
};

export function ProviderCard({
  provider,
  isCurrent,
  appId,
  isInConfig = true,
  onSwitch,
  onEdit,
  onDelete,
  onRemoveFromConfig,
  onConfigureUsage,
  onOpenWebsite,
  onDuplicate,
  onRegenerateName,
  onTest,
  onOpenTerminal,
  isTesting,
  dragHandleProps,
}: ProviderCardProps) {
  const { t } = useTranslation();

  const fallbackUrlText = t("provider.notConfigured", {
    defaultValue: "未配置接口地址",
  });

  const displayUrl = useMemo(
    () => extractApiUrl(provider, fallbackUrlText),
    [provider, fallbackUrlText],
  );

  const isClickableUrl = useMemo(() => {
    if (provider.notes?.trim()) return false;
    if (displayUrl === fallbackUrlText) return false;
    return true;
  }, [provider.notes, displayUrl, fallbackUrlText]);

  const isOfficial = isOfficialProvider(provider, appId);
  const isActiveProvider = isCurrent;
  const shouldUseBlue = isActiveProvider;

  const statusState = useProviderStatus({
    provider,
    appId,
    isCurrent,
    isInConfig,
  });

  const handleOpenWebsite = () => {
    if (!isClickableUrl) return;
    onOpenWebsite(displayUrl);
  };

  return (
    <div
      className={cn(
        "relative overflow-hidden rounded-lg border border-border py-2.5 px-3 transition-all duration-300",
        "bg-card text-card-foreground group",
        "hover:border-border-active",
        shouldUseBlue && "border-[#D97757]/60 shadow-sm shadow-[#D97757]/10",
        !isActiveProvider && "hover:shadow-sm",
        dragHandleProps?.isDragging &&
          "cursor-grabbing border-primary shadow-lg scale-105 z-10",
      )}
    >
      <div
        className={cn(
          "absolute inset-0 bg-gradient-to-r to-transparent transition-opacity duration-500 pointer-events-none",
          shouldUseBlue ? "from-[#D97757]/10" : "from-primary/10",
          isActiveProvider ? "opacity-100" : "opacity-0",
        )}
      />

      <div className="relative flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        {/* Left: drag handle + icon + name/url */}
        <div className="flex flex-1 items-center gap-2">
          <button
            type="button"
            className={cn(
              "-ml-1.5 flex-shrink-0 cursor-grab active:cursor-grabbing p-1.5",
              "text-muted-foreground/50 hover:text-muted-foreground transition-colors",
              dragHandleProps?.isDragging && "cursor-grabbing",
            )}
            aria-label={t("provider.dragHandle")}
            {...(dragHandleProps?.attributes ?? {})}
            {...(dragHandleProps?.listeners ?? {})}
          >
            <GripVertical className="h-4 w-4" />
          </button>

          <div className="h-6 w-6 rounded-md bg-muted flex items-center justify-center border border-border group-hover:scale-105 transition-transform duration-300">
            <ProviderIcon
              icon={provider.icon}
              name={provider.name}
              color={provider.iconColor}
              size={20}
            />
          </div>

          <div className="flex flex-wrap items-center gap-2 min-h-7">
            {isClickableUrl ? (
              <button
                type="button"
                onClick={handleOpenWebsite}
                className="text-base font-semibold leading-none hover:underline cursor-pointer"
                title={displayUrl}
              >
                {provider.name}
              </button>
            ) : (
              <h3 className="text-base font-semibold leading-none">
                {provider.name}
              </h3>
            )}

            {!isOfficial && onRegenerateName && (
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  onRegenerateName(provider);
                }}
                className="text-muted-foreground hover:text-foreground transition-colors"
                title={t("provider.regenerateName", {
                  defaultValue: "按规则重新生成名称",
                })}
                aria-label={t("provider.regenerateName", {
                  defaultValue: "按规则重新生成名称",
                })}
              >
                <Wand2 className="h-3.5 w-3.5" />
              </button>
            )}

            {provider.category === "third_party" &&
              provider.meta?.isPartner && (
                <span
                  className="text-yellow-500 dark:text-yellow-400"
                  title={t("provider.officialPartner", {
                    defaultValue: "官方合作伙伴",
                  })}
                >
                  ⭐
                </span>
              )}
          </div>
        </div>

        {/* Right: switch (always visible) + status indicator + action menu */}
        <div className="flex items-center ml-auto min-w-0 gap-3">
          <ProviderSwitchButton
            isCurrent={isCurrent}
            onSwitch={() => onSwitch(provider)}
          />

          <div className="ml-auto">
            <ProviderStatusIndicator
              provider={provider}
              appId={appId}
              isCurrent={isCurrent}
              isInConfig={isInConfig}
              isOfficial={isOfficial}
              status={statusState}
            />
          </div>

          <ProviderCardMenu
            provider={provider}
            appId={appId}
            isCurrent={isCurrent}
            isInConfig={isInConfig}
            isTesting={isTesting}
            isOfficial={isOfficial}
            onEdit={onEdit}
            onDuplicate={onDuplicate}
            onTest={onTest}
            onConfigureUsage={onConfigureUsage}
            onDelete={onDelete}
            onRemoveFromConfig={onRemoveFromConfig}
            onOpenTerminal={onOpenTerminal}
          />
        </div>
      </div>

      <ProviderStatusExpandedPanel
        provider={provider}
        appId={appId}
        isCurrent={isCurrent}
        isInConfig={isInConfig}
        status={statusState}
      />
    </div>
  );
}
