import { useState, useEffect, useCallback } from "react";
import { Plus, X, FolderOpen } from "lucide-react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { projectsApi, computePathLabels } from "@/lib/api/projects";
import { settingsApi, providersApi, type AppId } from "@/lib/api";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { Provider } from "@/types";

export type ProjectScope = "user" | string;

interface Props {
  selectedScope: ProjectScope;
  onScopeChange: (scope: ProjectScope) => void;
  providers: Record<string, Provider>;
  globalCurrentProviderId: string;
  projectCurrentProviderId: string;
  activeApp: AppId;
}

export function ProjectScopeList({
  selectedScope,
  onScopeChange,
  providers,
  globalCurrentProviderId,
  projectCurrentProviderId,
  activeApp,
}: Props) {
  const { t } = useTranslation();
  const [projectPaths, setProjectPaths] = useState<string[]>([]);
  const [isAdding, setIsAdding] = useState(false);
  const [confirmRemovePath, setConfirmRemovePath] = useState<string | null>(
    null,
  );
  const [scopeProviderMap, setScopeProviderMap] = useState<Map<string, string>>(
    new Map(),
  );
  const [col2Width, setCol2Width] = useState(80);

  const loadPaths = useCallback(async () => {
    try {
      const paths = await projectsApi.getAll();
      setProjectPaths(paths);
    } catch (err) {
      console.error("[ProjectScopeList] 加载项目路径失败:", err);
    }
  }, []);

  useEffect(() => {
    void loadPaths();
  }, [loadPaths]);

  useEffect(() => {
    if (projectPaths.length === 0) return;
    const load = async () => {
      const map = new Map<string, string>();
      await Promise.all(
        projectPaths.map(async (path) => {
          try {
            const id = await providersApi.getCurrentForProject(activeApp, path);
            if (id) map.set(path, id);
          } catch {
            // 忽略单个路径加载失败
          }
        }),
      );
      setScopeProviderMap(map);
    };
    void load();
  }, [projectPaths, activeApp]);

  useEffect(() => {
    if (selectedScope === "user") return;
    setScopeProviderMap((prev) => {
      const next = new Map(prev);
      if (projectCurrentProviderId) {
        next.set(selectedScope, projectCurrentProviderId);
      } else {
        next.delete(selectedScope);
      }
      return next;
    });
  }, [selectedScope, projectCurrentProviderId]);

  // 拖动第二列分割线
  const handleCol2DragStart = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();
      const startX = e.clientX;
      const startWidth = col2Width;
      const onMouseMove = (me: MouseEvent) => {
        // 向右拖 → 列2变窄；向左拖 → 列2变宽
        setCol2Width(
          Math.max(40, Math.min(150, startWidth - (me.clientX - startX))),
        );
      };
      const onMouseUp = () => {
        document.removeEventListener("mousemove", onMouseMove);
        document.removeEventListener("mouseup", onMouseUp);
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
      };
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";
      document.addEventListener("mousemove", onMouseMove);
      document.addEventListener("mouseup", onMouseUp);
    },
    [col2Width],
  );

  const handleAddProject = async () => {
    if (isAdding) return;
    setIsAdding(true);
    try {
      const dir = await settingsApi.pickDirectory();
      if (!dir) return;
      const added = await projectsApi.add(dir);
      if (added) {
        setProjectPaths((prev) => [...prev, dir]);
        onScopeChange(dir);
      } else {
        toast.info(
          t("projectScope.alreadyAdded", { defaultValue: "该路径已在列表中" }),
        );
        onScopeChange(dir);
      }
    } catch (err) {
      console.error("[ProjectScopeList] 添加项目路径失败:", err);
      toast.error(
        t("projectScope.addFailed", { defaultValue: "添加项目路径失败" }),
      );
    } finally {
      setIsAdding(false);
    }
  };

  const handleRemoveProject = async (path: string) => {
    try {
      await projectsApi.remove(path);
      setProjectPaths((prev) => prev.filter((p) => p !== path));
      if (selectedScope === path) {
        onScopeChange("user");
      }
    } catch (err) {
      console.error("[ProjectScopeList] 移除项目路径失败:", err);
      toast.error(
        t("projectScope.removeFailed", { defaultValue: "移除项目路径失败" }),
      );
    } finally {
      setConfirmRemovePath(null);
    }
  };

  return (
    <>
      <TooltipProvider delayDuration={400}>
        <Card className="flex flex-col w-full min-w-0 overflow-hidden rounded-xl">
          {/* 标题行 — 列结构与列表项保持一致 */}
          <CardHeader className="flex flex-row items-center px-3 py-2 border-b border-border/40 space-y-0">
            <div className="flex-1 min-w-0 overflow-hidden">
              <CardTitle className="text-xs font-medium text-muted-foreground uppercase tracking-wider truncate">
                {t("projectScope.title", { defaultValue: "设置项" })}
              </CardTitle>
            </div>
            {/* 可拖动分割条（标题行） */}
            <ColDivider onMouseDown={handleCol2DragStart} />
            {/* 第二列标题 */}
            <div
              style={{ width: col2Width }}
              className="shrink-0 flex items-center justify-end overflow-hidden"
            >
              <span className="text-xs text-muted-foreground/50 truncate">
                {t("projectScope.currentProvider", {
                  defaultValue: "当前供应商",
                })}
              </span>
            </div>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-7 w-7 text-muted-foreground hover:text-foreground shrink-0 ml-1"
                  onClick={() => void handleAddProject()}
                  disabled={isAdding}
                >
                  <Plus className="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="right">
                {t("projectScope.addProject", { defaultValue: "添加项目" })}
              </TooltipContent>
            </Tooltip>
          </CardHeader>

          <CardContent className="flex-1 overflow-y-auto overflow-x-hidden p-0 py-1">
            <ScopeItem
              label={t("projectScope.userLevel", { defaultValue: "全局配置" })}
              isSelected={selectedScope === "user"}
              onClick={() => onScopeChange("user")}
              icon={<FolderOpen className="h-3.5 w-3.5 shrink-0" />}
              currentProviderName={
                providers[globalCurrentProviderId]?.name ?? ""
              }
              col2Width={col2Width}
              onCol2DragStart={handleCol2DragStart}
            />

            {(() => {
              const pathLabels = computePathLabels(projectPaths);
              return projectPaths.map((path) => (
                <ScopeItem
                  key={path}
                  label={pathLabels.get(path) ?? path}
                  fullPath={path}
                  isSelected={selectedScope === path}
                  onClick={() => onScopeChange(path)}
                  onRemove={() => setConfirmRemovePath(path)}
                  icon={<FolderOpen className="h-3.5 w-3.5 shrink-0" />}
                  currentProviderName={
                    providers[scopeProviderMap.get(path) ?? ""]?.name ?? ""
                  }
                  col2Width={col2Width}
                  onCol2DragStart={handleCol2DragStart}
                />
              ));
            })()}

            {projectPaths.length === 0 && (
              <div className="px-3 py-2 text-xs text-muted-foreground/60 italic">
                {t("projectScope.noProjects", {
                  defaultValue: "点击 + 添加项目",
                })}
              </div>
            )}
          </CardContent>
        </Card>
      </TooltipProvider>

      <ConfirmDialog
        isOpen={confirmRemovePath !== null}
        title={t("projectScope.removeTitle", { defaultValue: "移除项目" })}
        message={t("projectScope.removeMessage", {
          defaultValue:
            "将从管理列表中移除此项目，项目目录下的 .claude/settings.json 不会被删除。\n\n{{path}}",
          path: confirmRemovePath ?? "",
        })}
        onConfirm={() => {
          if (confirmRemovePath) void handleRemoveProject(confirmRemovePath);
        }}
        onCancel={() => setConfirmRemovePath(null)}
      />
    </>
  );
}

// 列分割条组件
function ColDivider({
  onMouseDown,
}: {
  onMouseDown: (e: React.MouseEvent) => void;
}) {
  return (
    <div
      className="flex-shrink-0 w-3 self-stretch flex items-center justify-center cursor-col-resize group/col"
      onMouseDown={onMouseDown}
    >
      <div className="w-px h-4 rounded-full bg-border/40 group-hover/col:bg-primary/50 transition-colors duration-150" />
    </div>
  );
}

interface ScopeItemProps {
  label: string;
  fullPath?: string;
  isSelected: boolean;
  onClick: () => void;
  onRemove?: () => void;
  icon?: React.ReactNode;
  currentProviderName?: string;
  col2Width: number;
  onCol2DragStart: (e: React.MouseEvent) => void;
}

function ScopeItem({
  label,
  fullPath,
  isSelected,
  onClick,
  onRemove,
  icon,
  currentProviderName,
  col2Width,
  onCol2DragStart,
}: ScopeItemProps) {
  const [hovered, setHovered] = useState(false);

  const inner = (
    <div
      role="button"
      tabIndex={0}
      onClick={onClick}
      onKeyDown={(e) => e.key === "Enter" && onClick()}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      className={cn(
        "group flex items-center w-full px-3 py-1.5 cursor-pointer select-none",
        "transition-colors duration-100",
        isSelected
          ? "bg-gradient-to-r from-[#D97757]/10 to-transparent text-foreground font-medium"
          : "text-muted-foreground hover:bg-muted/60 hover:text-foreground",
      )}
    >
      {/* 第一列：范围信息 */}
      <div className="flex items-center gap-1.5 flex-1 min-w-0 overflow-hidden">
        <span className="shrink-0 opacity-70">{icon}</span>
        <span className="flex-1 truncate min-w-0" title={fullPath ?? label}>
          {label}
        </span>
        {onRemove && (hovered || isSelected) && (
          <button
            onClick={(e) => {
              e.stopPropagation();
              onRemove();
            }}
            className={cn(
              "shrink-0 rounded p-0.5 opacity-0 group-hover:opacity-100",
              "text-muted-foreground hover:text-destructive hover:bg-destructive/10",
              "transition-all duration-100",
            )}
            aria-label="移除"
          >
            <X className="h-3 w-3" />
          </button>
        )}
      </div>

      {/* 可拖动列分割条 */}
      {currentProviderName !== undefined && (
        <ColDivider onMouseDown={onCol2DragStart} />
      )}

      {/* 第二列：当前供应商 */}
      {currentProviderName !== undefined && (
        <div
          style={{ width: col2Width }}
          className="shrink-0 flex items-center justify-end overflow-hidden"
        >
          <span
            className={cn(
              "text-xs truncate",
              isSelected ? "text-foreground/60" : "text-muted-foreground/50",
            )}
          >
            {currentProviderName || "—"}
          </span>
        </div>
      )}
    </div>
  );

  if (fullPath) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <div
            className={cn(
              "border-l-2",
              isSelected ? "border-primary" : "border-transparent",
            )}
          >
            {inner}
          </div>
        </TooltipTrigger>
        <TooltipContent side="right" className="max-w-xs break-all text-xs">
          {fullPath}
        </TooltipContent>
      </Tooltip>
    );
  }

  return (
    <div
      className={cn(
        "border-l-2",
        isSelected ? "border-primary" : "border-transparent",
      )}
    >
      {inner}
    </div>
  );
}
