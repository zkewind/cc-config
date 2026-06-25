import { CSS } from "@dnd-kit/utilities";
import { DndContext, closestCenter } from "@dnd-kit/core";
import {
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
} from "react";
import { AnimatePresence, motion } from "framer-motion";
import { Search, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import type { Provider } from "@/types";
import type { AppId } from "@/lib/api";
import { providersApi } from "@/lib/api/providers";
import { useDragSort } from "@/hooks/useDragSort";
import { useStreamCheck } from "@/hooks/useStreamCheck";
import { ProviderCard } from "@/components/providers/ProviderCard";
import { ProviderEmptyState } from "@/components/providers/ProviderEmptyState";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { settingsApi } from "@/lib/api/settings";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Plus, Download } from "lucide-react";

interface ProviderListProps {
  providers: Record<string, Provider>;
  currentProviderId: string;
  appId: AppId;
  onSwitch: (provider: Provider) => void;
  onEdit: (provider: Provider) => void;
  onDelete: (provider: Provider) => void;
  onRemoveFromConfig?: (provider: Provider) => void;
  onDuplicate: (provider: Provider) => void;
  onConfigureUsage?: (provider: Provider) => void;
  onOpenWebsite: (url: string) => void;
  onOpenTerminal?: (provider: Provider) => void;
  onCreate?: () => void;
  isLoading?: boolean;
  onSetAsDefault?: (provider: Provider) => void;
}

export function ProviderList({
  providers,
  currentProviderId,
  appId,
  onSwitch,
  onEdit,
  onDelete,
  onRemoveFromConfig,
  onDuplicate,
  onConfigureUsage,
  onOpenWebsite,
  onOpenTerminal,
  onCreate,
  isLoading = false,
}: ProviderListProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { checkProvider, isChecking } = useStreamCheck(appId);
  const { sortedProviders, sensors, handleDragEnd } = useDragSort(
    providers,
    appId,
  );

  const [searchTerm, setSearchTerm] = useState("");
  const [isSearchOpen, setIsSearchOpen] = useState(false);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const [showStreamCheckConfirm, setShowStreamCheckConfirm] = useState(false);
  const [pendingTestProvider, setPendingTestProvider] =
    useState<Provider | null>(null);

  const { data: settings } = useQuery({
    queryKey: ["settings"],
    queryFn: () => settingsApi.get(),
  });

  const handleTest = useCallback(
    (provider: Provider) => {
      if (!settings?.streamCheckConfirmed) {
        setPendingTestProvider(provider);
        setShowStreamCheckConfirm(true);
      } else {
        checkProvider(provider.id, provider.name);
      }
    },
    [checkProvider, settings?.streamCheckConfirmed],
  );

  const handleStreamCheckConfirm = async () => {
    setShowStreamCheckConfirm(false);
    try {
      if (settings) {
        const { webdavSync: _, ...rest } = settings;
        await settingsApi.save({ ...rest, streamCheckConfirmed: true });
        await queryClient.invalidateQueries({ queryKey: ["settings"] });
      }
    } catch (error) {
      console.error("Failed to save stream check confirmed:", error);
    }
    if (pendingTestProvider) {
      checkProvider(pendingTestProvider.id, pendingTestProvider.name);
      setPendingTestProvider(null);
    }
  };

  const handleExport = useCallback(async () => {
    try {
      const now = new Date();
      const stamp = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, "0")}${String(now.getDate()).padStart(2, "0")}`;
      const defaultName = `providers-export-${stamp}.json`;
      const destination = await settingsApi.saveJsonFileDialog(defaultName);
      if (!destination) return;
      await providersApi.exportJson(appId, destination);
      toast.success(
        t("provider.exportSuccess", { defaultValue: "供应商清单已导出" }),
        {
          closeButton: true,
        },
      );
    } catch (error) {
      toast.error(
        t("provider.exportFailed", {
          defaultValue: "导出失败: {{message}}",
          message: error instanceof Error ? error.message : String(error),
        }),
      );
    }
  }, [appId, t]);

  const importMutation = useMutation({
    mutationFn: async (): Promise<boolean> => providersApi.importDefault(appId),
    onSuccess: (imported) => {
      if (imported) {
        queryClient.invalidateQueries({ queryKey: ["providers", appId] });
        toast.success(t("provider.importCurrentDescription"));
      } else {
        toast.info(t("provider.noProviders"));
      }
    },
    onError: (error: Error) => {
      toast.error(error.message);
    },
  });

  const regenerateMutation = useMutation({
    mutationFn: (provider: Provider) =>
      providersApi.regenerateName(provider.id, appId),
    onSuccess: (newName) => {
      queryClient.invalidateQueries({ queryKey: ["providers", appId] });
      toast.success(
        t("provider.regenerateNameSuccess", {
          name: newName,
          defaultValue: "名称已更新为 {{name}}",
        }),
      );
    },
    onError: (error: Error) => {
      toast.error(error.message);
    },
  });

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const key = event.key.toLowerCase();
      if ((event.metaKey || event.ctrlKey) && key === "f") {
        event.preventDefault();
        setIsSearchOpen(true);
        return;
      }

      if (key === "escape") {
        setIsSearchOpen(false);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  useEffect(() => {
    if (isSearchOpen) {
      const frame = requestAnimationFrame(() => {
        searchInputRef.current?.focus();
        searchInputRef.current?.select();
      });
      return () => cancelAnimationFrame(frame);
    }
  }, [isSearchOpen]);

  const filteredProviders = useMemo(() => {
    const keyword = searchTerm.trim().toLowerCase();
    if (!keyword) return sortedProviders;
    return sortedProviders.filter((provider) => {
      const fields = [provider.name, provider.notes, provider.websiteUrl];
      return fields.some((field) =>
        field?.toString().toLowerCase().includes(keyword),
      );
    });
  }, [searchTerm, sortedProviders]);

  if (isLoading) {
    return (
      <div className="space-y-3">
        {[0, 1, 2].map((index) => (
          <div
            key={index}
            className="w-full border border-dashed rounded-lg h-28 border-muted-foreground/40 bg-muted/40"
          />
        ))}
      </div>
    );
  }

  if (sortedProviders.length === 0) {
    return (
      <ProviderEmptyState
        appId={appId}
        onCreate={onCreate}
        onImport={() => importMutation.mutate()}
      />
    );
  }

  const renderProviderList = () => (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragEnd={handleDragEnd}
    >
      <SortableContext
        items={filteredProviders.map((provider) => provider.id)}
        strategy={verticalListSortingStrategy}
      >
        <div className="space-y-1.5">
          {filteredProviders.map((provider) => (
            <SortableProviderCard
              key={provider.id}
              provider={provider}
              isCurrent={provider.id === currentProviderId}
              appId={appId}
              isInConfig={true}
              onSwitch={onSwitch}
              onEdit={onEdit}
              onDelete={onDelete}
              onRemoveFromConfig={onRemoveFromConfig}
              onDuplicate={onDuplicate}
              onRegenerateName={(p) => regenerateMutation.mutate(p)}
              onConfigureUsage={onConfigureUsage}
              onOpenWebsite={onOpenWebsite}
              onOpenTerminal={onOpenTerminal}
              onTest={handleTest}
              isTesting={isChecking(provider.id)}
            />
          ))}
        </div>
      </SortableContext>
    </DndContext>
  );

  return (
    <Card className="flex flex-col flex-1 min-h-0 relative overflow-hidden">
      {/* CardHeader */}
      <CardHeader className="flex flex-row items-center justify-between px-3 py-2 border-b border-border/40 space-y-0 flex-shrink-0">
        <div className="flex items-center gap-2">
          <CardTitle className="text-sm font-semibold">
            {t("provider.title", { defaultValue: "供应商" })}
          </CardTitle>
          <Badge variant="secondary" className="text-xs px-1.5 py-0">
            {sortedProviders.length}
          </Badge>
        </div>
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 text-muted-foreground hover:text-foreground"
            onClick={() => setIsSearchOpen((v) => !v)}
            aria-label={t("provider.searchAriaLabel", {
              defaultValue: "Search providers",
            })}
          >
            <Search className="h-4 w-4" />
          </Button>
          {onCreate && (
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7 text-muted-foreground hover:text-foreground"
              onClick={onCreate}
              aria-label={t("provider.addProvider", {
                defaultValue: "添加供应商",
              })}
            >
              <Plus className="h-4 w-4" />
            </Button>
          )}
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 text-muted-foreground hover:text-foreground"
            onClick={() => void handleExport()}
            aria-label={t("provider.exportList", {
              defaultValue: "导出供应商清单",
            })}
          >
            <Download className="h-4 w-4" />
          </Button>
        </div>
      </CardHeader>

      {/* 浮动搜索框：absolute 定位于 Card 内部，不超出 Card 边界 */}
      <AnimatePresence>
        {isSearchOpen && (
          <motion.div
            key="provider-search"
            initial={{ opacity: 0, y: -8, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -8, scale: 0.98 }}
            transition={{ duration: 0.18, ease: "easeOut" }}
            className="absolute right-2 top-12 z-10 w-[min(90%,22rem)]"
          >
            <div className="p-4 space-y-3 border shadow-md rounded-2xl border-white/10 bg-background/95 shadow-black/20 backdrop-blur-md">
              <div className="relative flex items-center gap-2">
                <Search className="absolute w-4 h-4 -translate-y-1/2 pointer-events-none left-3 top-1/2 text-muted-foreground" />
                <Input
                  ref={searchInputRef}
                  value={searchTerm}
                  onChange={(event) => setSearchTerm(event.target.value)}
                  placeholder={t("provider.searchPlaceholder", {
                    defaultValue: "Search name, notes, or URL...",
                  })}
                  aria-label={t("provider.searchAriaLabel", {
                    defaultValue: "Search providers",
                  })}
                  className="pr-16 pl-9"
                />
                {searchTerm && (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="absolute text-xs -translate-y-1/2 right-11 top-1/2"
                    onClick={() => setSearchTerm("")}
                  >
                    {t("common.clear", { defaultValue: "Clear" })}
                  </Button>
                )}
                <Button
                  variant="ghost"
                  size="icon"
                  className="ml-auto"
                  onClick={() => setIsSearchOpen(false)}
                  aria-label={t("provider.searchCloseAriaLabel", {
                    defaultValue: "Close provider search",
                  })}
                >
                  <X className="w-4 h-4" />
                </Button>
              </div>
              <div className="flex flex-wrap items-center justify-between gap-2 text-[11px] text-muted-foreground">
                <span>
                  {t("provider.searchScopeHint", {
                    defaultValue: "Matches provider name, notes, and URL.",
                  })}
                </span>
                <span>
                  {t("provider.searchCloseHint", {
                    defaultValue: "Press Esc to close",
                  })}
                </span>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* CardContent: Provider 列表 */}
      <CardContent className="flex-1 overflow-y-auto p-2">
        {filteredProviders.length === 0 ? (
          <div className="px-6 py-8 text-sm text-center border border-dashed rounded-lg border-border text-muted-foreground">
            {t("provider.noSearchResults", {
              defaultValue: "No providers match your search.",
            })}
          </div>
        ) : (
          renderProviderList()
        )}
      </CardContent>

      <ConfirmDialog
        isOpen={showStreamCheckConfirm}
        variant="info"
        title={t("confirm.streamCheck.title")}
        message={t("confirm.streamCheck.message")}
        confirmText={t("confirm.streamCheck.confirm")}
        onConfirm={() => void handleStreamCheckConfirm()}
        onCancel={() => {
          setShowStreamCheckConfirm(false);
          setPendingTestProvider(null);
        }}
      />
    </Card>
  );
}

interface SortableProviderCardProps {
  provider: Provider;
  isCurrent: boolean;
  appId: AppId;
  isInConfig: boolean;
  onSwitch: (provider: Provider) => void;
  onEdit: (provider: Provider) => void;
  onDelete: (provider: Provider) => void;
  onRemoveFromConfig?: (provider: Provider) => void;
  onDuplicate: (provider: Provider) => void;
  onRegenerateName?: (provider: Provider) => void;
  onConfigureUsage?: (provider: Provider) => void;
  onOpenWebsite: (url: string) => void;
  onOpenTerminal?: (provider: Provider) => void;
  onTest?: (provider: Provider) => void;
  isTesting: boolean;
}

function SortableProviderCard({
  provider,
  isCurrent,
  appId,
  isInConfig,
  onSwitch,
  onEdit,
  onDelete,
  onRemoveFromConfig,
  onDuplicate,
  onRegenerateName,
  onConfigureUsage,
  onOpenWebsite,
  onOpenTerminal,
  onTest,
  isTesting,
}: SortableProviderCardProps) {
  const {
    setNodeRef,
    attributes,
    listeners,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: provider.id });

  const style: CSSProperties = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  return (
    <div ref={setNodeRef} style={style}>
      <ProviderCard
        provider={provider}
        isCurrent={isCurrent}
        appId={appId}
        isInConfig={isInConfig}
        onSwitch={onSwitch}
        onEdit={onEdit}
        onDelete={onDelete}
        onRemoveFromConfig={onRemoveFromConfig}
        onDuplicate={onDuplicate}
        onRegenerateName={
          onRegenerateName ? (p) => onRegenerateName(p) : undefined
        }
        onConfigureUsage={
          onConfigureUsage ? (item) => onConfigureUsage(item) : () => undefined
        }
        onOpenWebsite={onOpenWebsite}
        onOpenTerminal={onOpenTerminal}
        onTest={onTest}
        isTesting={isTesting}
        dragHandleProps={{
          attributes,
          listeners,
          isDragging,
        }}
      />
    </div>
  );
}
