import React, {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import type { AppId } from "@/lib/api";
import { isTextEditableTarget } from "@/utils/domUtils";

export type View =
  | "providers"
  | "settings"
  | "prompts"
  | "skills"
  | "skillsDiscovery"
  | "mcp"
  | "sessions";

const VIEW_STORAGE_KEY = "cc-config-last-view";
const VALID_VIEWS: View[] = [
  "providers",
  "settings",
  "prompts",
  "skills",
  "skillsDiscovery",
  "mcp",
  "sessions",
];

const getInitialView = (): View => {
  const saved = localStorage.getItem(VIEW_STORAGE_KEY) as View | null;
  if (saved && VALID_VIEWS.includes(saved)) {
    return saved;
  }
  return "providers";
};

interface RouterContextValue {
  activeApp: AppId;
  currentView: View;
  settingsDefaultTab: string;
  setCurrentView: (view: View) => void;
  setSettingsDefaultTab: (tab: string) => void;
}

const RouterContext = createContext<RouterContextValue | null>(null);

export function useAppRouter(): RouterContextValue {
  const ctx = useContext(RouterContext);
  if (!ctx) {
    throw new Error("useAppRouter must be used within AppRouter");
  }
  return ctx;
}

interface AppRouterProps {
  children: React.ReactNode;
}

export function AppRouter({ children }: AppRouterProps) {
  const activeApp: AppId = "claude";
  const [currentView, setCurrentView] = useState<View>(getInitialView);
  const [settingsDefaultTab, setSettingsDefaultTab] = useState("general");

  useEffect(() => {
    localStorage.setItem(VIEW_STORAGE_KEY, currentView);
  }, [currentView]);

  const currentViewRef = useRef(currentView);
  useEffect(() => {
    currentViewRef.current = currentView;
  }, [currentView]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "," && (event.metaKey || event.ctrlKey)) {
        event.preventDefault();
        setCurrentView("settings");
        return;
      }

      if (event.key !== "Escape" || event.defaultPrevented) return;
      if (document.body.style.overflow === "hidden") return;

      const view = currentViewRef.current;
      if (view === "providers") return;
      if (isTextEditableTarget(event.target)) return;

      event.preventDefault();
      setCurrentView(view === "skillsDiscovery" ? "skills" : "providers");
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  const value: RouterContextValue = {
    activeApp,
    currentView,
    settingsDefaultTab,
    setCurrentView,
    setSettingsDefaultTab,
  };

  return (
    <RouterContext.Provider value={value}>{children}</RouterContext.Provider>
  );
}
