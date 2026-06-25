import { useState, useEffect, useCallback } from "react";
import { Save, Loader2, FolderOpen, FileJson } from "lucide-react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { projectsApi } from "@/lib/api/projects";
import { Button } from "@/components/ui/button";
import JsonEditor from "@/components/JsonEditor";
import { abbreviatePath } from "@/lib/api/projects";

const DEFAULT_SETTINGS = JSON.stringify({ env: {} }, null, 2);

const TOGGLES = [
  {
    key: "hideAttribution",
    label: "claudeConfig.hideAttribution",
    get: (cfg: Record<string, unknown>) =>
      (cfg?.attribution as Record<string, string>)?.commit === "" &&
      (cfg?.attribution as Record<string, string>)?.pr === "",
    set: (cfg: Record<string, unknown>, on: boolean) => {
      if (on) cfg.attribution = { commit: "", pr: "" };
      else delete cfg.attribution;
    },
  },
  {
    key: "teammates",
    label: "claudeConfig.enableTeammates",
    get: (cfg: Record<string, unknown>) => {
      const env = cfg?.env as Record<string, string> | undefined;
      return (
        env?.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS === "1" ||
        (env?.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS as unknown) === 1
      );
    },
    set: (cfg: Record<string, unknown>, on: boolean) => {
      if (!cfg.env) cfg.env = {};
      const env = cfg.env as Record<string, string>;
      if (on) env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS = "1";
      else {
        delete env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS;
        if (Object.keys(env).length === 0) delete cfg.env;
      }
    },
  },
  {
    key: "enableToolSearch",
    label: "claudeConfig.enableToolSearch",
    get: (cfg: Record<string, unknown>) => {
      const env = cfg?.env as Record<string, string> | undefined;
      return (
        env?.ENABLE_TOOL_SEARCH === "true" || env?.ENABLE_TOOL_SEARCH === "1"
      );
    },
    set: (cfg: Record<string, unknown>, on: boolean) => {
      if (!cfg.env) cfg.env = {};
      const env = cfg.env as Record<string, string>;
      if (on) env.ENABLE_TOOL_SEARCH = "true";
      else {
        delete env.ENABLE_TOOL_SEARCH;
        if (Object.keys(env).length === 0) delete cfg.env;
      }
    },
  },
  {
    key: "effortMax",
    label: "claudeConfig.effortMax",
    get: (cfg: Record<string, unknown>) => {
      const env = cfg?.env as Record<string, string> | undefined;
      return env?.CLAUDE_CODE_EFFORT_LEVEL === "max";
    },
    set: (cfg: Record<string, unknown>, on: boolean) => {
      if (!cfg.env) cfg.env = {};
      const env = cfg.env as Record<string, string>;
      if (on) env.CLAUDE_CODE_EFFORT_LEVEL = "max";
      else {
        delete env.CLAUDE_CODE_EFFORT_LEVEL;
        if (Object.keys(env).length === 0) delete cfg.env;
      }
    },
  },
  {
    key: "disableAutoUpgrade",
    label: "claudeConfig.disableAutoUpgrade",
    get: (cfg: Record<string, unknown>) => {
      const env = cfg?.env as Record<string, string> | undefined;
      return (
        env?.DISABLE_AUTOUPDATER === "1" ||
        (env?.DISABLE_AUTOUPDATER as unknown) === 1
      );
    },
    set: (cfg: Record<string, unknown>, on: boolean) => {
      if (!cfg.env) cfg.env = {};
      const env = cfg.env as Record<string, string>;
      if (on) env.DISABLE_AUTOUPDATER = "1";
      else {
        delete env.DISABLE_AUTOUPDATER;
        if (Object.keys(env).length === 0) delete cfg.env;
      }
    },
  },
] as const;

interface Props {
  projectPath: string;
}

export function ProjectSettingsPanel({ projectPath }: Props) {
  const { t } = useTranslation();
  const [content, setContent] = useState(DEFAULT_SETTINGS);
  const [isDirty, setIsDirty] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isDarkMode, setIsDarkMode] = useState(
    document.documentElement.classList.contains("dark"),
  );

  // 跟随系统暗色模式
  useEffect(() => {
    const observer = new MutationObserver(() =>
      setIsDarkMode(document.documentElement.classList.contains("dark")),
    );
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });
    return () => observer.disconnect();
  }, []);

  // 加载项目 settings.json
  const loadSettings = useCallback(async () => {
    setIsLoading(true);
    setIsDirty(false);
    try {
      const raw = await projectsApi.readSettings(projectPath);
      setContent(raw ?? DEFAULT_SETTINGS);
    } catch (err) {
      console.error("[ProjectSettingsPanel] 加载失败:", err);
      toast.error(
        t("projectScope.loadFailed", { defaultValue: "加载项目配置失败" }),
      );
      setContent(DEFAULT_SETTINGS);
    } finally {
      setIsLoading(false);
    }
  }, [projectPath, t]);

  useEffect(() => {
    void loadSettings();
  }, [loadSettings]);

  const handleChange = useCallback((val: string) => {
    setContent(val);
    setIsDirty(true);
  }, []);

  const handleSave = async () => {
    // 先校验 JSON
    try {
      JSON.parse(content);
    } catch {
      toast.error(
        t("provider.jsonInvalid", {
          defaultValue: "JSON 格式错误，请检查后再保存",
        }),
      );
      return;
    }
    setIsSaving(true);
    try {
      await projectsApi.saveSettings(projectPath, content);
      setIsDirty(false);
      toast.success(
        t("projectScope.saveDone", { defaultValue: "项目配置已保存" }),
      );
    } catch (err) {
      console.error("[ProjectSettingsPanel] 保存失败:", err);
      toast.error(
        t("projectScope.saveFailed", { defaultValue: "保存失败" }) +
          (err instanceof Error ? `: ${err.message}` : ""),
      );
    } finally {
      setIsSaving(false);
    }
  };

  // Toggle 快捷开关
  const getToggleState = (key: string): boolean => {
    try {
      const cfg = JSON.parse(content) as Record<string, unknown>;
      const toggle = TOGGLES.find((t) => t.key === key);
      return toggle ? toggle.get(cfg) : false;
    } catch {
      return false;
    }
  };

  const handleToggle = (key: string, on: boolean) => {
    try {
      const cfg = JSON.parse(content || "{}") as Record<string, unknown>;
      const toggle = TOGGLES.find((t) => t.key === key);
      if (toggle) {
        toggle.set(cfg, on);
        handleChange(JSON.stringify(cfg, null, 2));
      }
    } catch {
      // JSON 无效时不操作
    }
  };

  const settingsFilePath = `${projectPath}/.claude/settings.json`;

  return (
    <div className="flex flex-col h-full">
      {/* 顶部路径栏 */}
      <div className="flex items-center gap-2 px-6 py-3 border-b border-border/40 bg-muted/10 shrink-0">
        <FolderOpen className="w-4 h-4 text-muted-foreground shrink-0" />
        <span
          className="text-sm text-muted-foreground truncate"
          title={settingsFilePath}
        >
          {abbreviatePath(settingsFilePath, 60)}
        </span>
        <FileJson className="w-4 h-4 text-muted-foreground/50 shrink-0 ml-auto" />
      </div>

      {/* 编辑区 */}
      <div className="flex-1 min-h-0 overflow-y-auto px-6 py-4 space-y-3">
        {isLoading ? (
          <div className="flex items-center justify-center h-40 text-muted-foreground gap-2">
            <Loader2 className="w-4 h-4 animate-spin" />
            <span className="text-sm">
              {t("common.loading", { defaultValue: "加载中..." })}
            </span>
          </div>
        ) : (
          <>
            {/* Toggle 快捷开关 */}
            <div className="flex flex-wrap items-center gap-x-4 gap-y-1">
              {TOGGLES.map((toggle) => (
                <label
                  key={toggle.key}
                  className="inline-flex items-center gap-2 text-sm text-muted-foreground cursor-pointer"
                >
                  <input
                    type="checkbox"
                    checked={getToggleState(toggle.key)}
                    onChange={(e) => handleToggle(toggle.key, e.target.checked)}
                    className="w-4 h-4 text-blue-500 bg-white dark:bg-gray-800 border-border rounded focus:ring-blue-500 focus:ring-2"
                  />
                  <span>{t(toggle.label)}</span>
                </label>
              ))}
            </div>

            {/* JSON 编辑器 */}
            <JsonEditor
              value={content}
              onChange={handleChange}
              placeholder={DEFAULT_SETTINGS}
              darkMode={isDarkMode}
              showValidation
              language="json"
              height="calc(100vh - 300px)"
            />
          </>
        )}
      </div>

      {/* 底部保存栏 */}
      <div className="shrink-0 flex items-center justify-between px-6 py-3 border-t border-border/40 bg-background/80 backdrop-blur">
        <span className="text-xs text-muted-foreground">
          {isDirty
            ? t("projectScope.unsaved", { defaultValue: "有未保存的更改" })
            : t("projectScope.saved", { defaultValue: "已保存" })}
        </span>
        <Button
          onClick={() => void handleSave()}
          disabled={isSaving || isLoading || !isDirty}
          size="sm"
          className="gap-2"
        >
          {isSaving ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <Save className="w-4 h-4" />
          )}
          {t("common.save", { defaultValue: "保存" })}
        </Button>
      </div>
    </div>
  );
}
