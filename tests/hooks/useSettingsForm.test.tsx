import { renderHook, act, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import i18n from "i18next";
import { useSettingsForm } from "@/hooks/useSettingsForm";

const useSettingsQueryMock = vi.fn();

vi.mock("@/lib/query", () => ({
  useSettingsQuery: (...args: unknown[]) => useSettingsQueryMock(...args),
}));

let changeLanguageSpy: ReturnType<typeof vi.spyOn<any, any>>;

beforeEach(() => {
  useSettingsQueryMock.mockReset();
  window.localStorage.clear();
  (i18n as any).language = "zh";
  changeLanguageSpy = vi
    .spyOn(i18n, "changeLanguage")
    .mockImplementation(async (lang?: string) => {
      (i18n as any).language = lang;
      return i18n.t;
    });
});

afterEach(() => {
  changeLanguageSpy.mockRestore();
});

describe("useSettingsForm Hook", () => {
  it("normalizes Claude settings and syncs language on initialization", async () => {
    useSettingsQueryMock.mockReturnValue({
      data: {
        showInTray: undefined,
        minimizeToTrayOnClose: undefined,
        enableClaudePluginIntegration: undefined,
        claudeConfigDir: "  /Users/demo  ",
        language: "en",
      },
      isLoading: false,
    });

    const { result } = renderHook(() => useSettingsForm());

    await waitFor(() => {
      expect(result.current.settings).not.toBeNull();
    });

    const settings = result.current.settings!;
    expect(settings.showInTray).toBe(true);
    expect(settings.minimizeToTrayOnClose).toBe(true);
    expect(settings.enableClaudePluginIntegration).toBe(false);
    expect(settings.claudeConfigDir).toBe("/Users/demo");
    expect(settings.language).toBe("en");
    expect(result.current.initialLanguage).toBe("en");
    expect(changeLanguageSpy).toHaveBeenCalledWith("en");
  });

  it("prioritizes reading language from local storage", () => {
    useSettingsQueryMock.mockReturnValue({ data: null, isLoading: false });
    window.localStorage.setItem("language", "en");

    const { result } = renderHook(() => useSettingsForm());

    expect(result.current.readPersistedLanguage()).toBe("en");
    expect(changeLanguageSpy).not.toHaveBeenCalled();
  });

  it("updates fields and syncs language when language changes", () => {
    useSettingsQueryMock.mockReturnValue({ data: null, isLoading: false });

    const { result } = renderHook(() => useSettingsForm());

    act(() => {
      result.current.updateSettings({ showInTray: false });
    });

    expect(result.current.settings?.showInTray).toBe(false);

    changeLanguageSpy.mockClear();
    act(() => {
      result.current.updateSettings({ language: "en" });
    });

    expect(result.current.settings?.language).toBe("en");
    expect(changeLanguageSpy).toHaveBeenCalledWith("en");
  });

  it("resets with server data and restores initial language", async () => {
    useSettingsQueryMock.mockReturnValue({
      data: {
        showInTray: true,
        minimizeToTrayOnClose: true,
        enableClaudePluginIntegration: false,
        claudeConfigDir: "/origin",
        language: "en",
      },
      isLoading: false,
    });

    const { result } = renderHook(() => useSettingsForm());

    await waitFor(() => {
      expect(result.current.settings).not.toBeNull();
    });

    changeLanguageSpy.mockClear();
    (i18n as any).language = "zh";

    act(() => {
      result.current.resetSettings({
        showInTray: false,
        minimizeToTrayOnClose: false,
        enableClaudePluginIntegration: true,
        claudeConfigDir: "  /reset  ",
        language: "zh",
      });
    });

    const settings = result.current.settings!;
    expect(settings.showInTray).toBe(false);
    expect(settings.minimizeToTrayOnClose).toBe(false);
    expect(settings.enableClaudePluginIntegration).toBe(true);
    expect(settings.claudeConfigDir).toBe("/reset");
    expect(settings.language).toBe("zh");
    expect(result.current.initialLanguage).toBe("en");
    expect(changeLanguageSpy).toHaveBeenCalledWith("en");
  });

  it("does not call changeLanguage repeatedly when language is consistent", async () => {
    useSettingsQueryMock.mockReturnValue({
      data: {
        showInTray: true,
        minimizeToTrayOnClose: true,
        enableClaudePluginIntegration: false,
        claudeConfigDir: null,
        language: "zh",
      },
      isLoading: false,
    });

    const { result } = renderHook(() => useSettingsForm());

    await waitFor(() => {
      expect(result.current.settings).not.toBeNull();
    });

    changeLanguageSpy.mockClear();
    (i18n as any).language = "zh";

    act(() => {
      result.current.syncLanguage("zh");
    });

    expect(changeLanguageSpy).not.toHaveBeenCalled();
  });
});
