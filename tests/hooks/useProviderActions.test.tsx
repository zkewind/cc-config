import type { ReactNode } from "react";
import { renderHook, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { useProviderActions } from "@/hooks/useProviderActions";
import type { Provider, UsageScript } from "@/types";

const toastSuccessMock = vi.fn();
const toastErrorMock = vi.fn();
const toastWarningMock = vi.fn();
const addProviderMutateAsync = vi.fn();
const updateProviderMutateAsync = vi.fn();
const deleteProviderMutateAsync = vi.fn();
const switchProviderMutateAsync = vi.fn();
const providersApiUpdateMock = vi.fn();
const providersApiUpdateTrayMenuMock = vi.fn();
const settingsApiGetMock = vi.fn();
const settingsApiApplyMock = vi.fn();

const addProviderMutation = { mutateAsync: addProviderMutateAsync, isPending: false };
const updateProviderMutation = { mutateAsync: updateProviderMutateAsync, isPending: false };
const deleteProviderMutation = { mutateAsync: deleteProviderMutateAsync, isPending: false };
const switchProviderMutation = { mutateAsync: switchProviderMutateAsync, isPending: false };

const useAddProviderMutationMock = vi.fn(() => addProviderMutation);
const useUpdateProviderMutationMock = vi.fn(() => updateProviderMutation);
const useDeleteProviderMutationMock = vi.fn(() => deleteProviderMutation);
const useSwitchProviderMutationMock = vi.fn(() => switchProviderMutation);

vi.mock("sonner", () => ({
  toast: {
    success: (...args: unknown[]) => toastSuccessMock(...args),
    error: (...args: unknown[]) => toastErrorMock(...args),
    warning: (...args: unknown[]) => toastWarningMock(...args),
  },
}));

vi.mock("@/lib/query", () => ({
  useAddProviderMutation: () => useAddProviderMutationMock(),
  useUpdateProviderMutation: () => useUpdateProviderMutationMock(),
  useDeleteProviderMutation: () => useDeleteProviderMutationMock(),
  useSwitchProviderMutation: () => useSwitchProviderMutationMock(),
}));

vi.mock("@/lib/api", () => ({
  providersApi: {
    update: (...args: unknown[]) => providersApiUpdateMock(...args),
    updateTrayMenu: (...args: unknown[]) =>
      providersApiUpdateTrayMenuMock(...args),
  },
  settingsApi: {
    get: (...args: unknown[]) => settingsApiGetMock(...args),
    applyClaudePluginConfig: (...args: unknown[]) =>
      settingsApiApplyMock(...args),
  },
}));

function createWrapper() {
  const queryClient = new QueryClient();
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  return { wrapper, queryClient };
}

function createProvider(overrides: Partial<Provider> = {}): Provider {
  return {
    id: "provider-1",
    name: "Test Provider",
    settingsConfig: {},
    category: "official",
    ...overrides,
  };
}

beforeEach(() => {
  addProviderMutateAsync.mockReset();
  updateProviderMutateAsync.mockReset();
  deleteProviderMutateAsync.mockReset();
  switchProviderMutateAsync.mockReset();
  providersApiUpdateMock.mockReset();
  providersApiUpdateTrayMenuMock.mockReset();
  settingsApiGetMock.mockReset();
  settingsApiApplyMock.mockReset();
  toastSuccessMock.mockReset();
  toastErrorMock.mockReset();
  toastWarningMock.mockReset();
  addProviderMutation.isPending = false;
  updateProviderMutation.isPending = false;
  deleteProviderMutation.isPending = false;
  switchProviderMutation.isPending = false;
});

describe("useProviderActions", () => {
  it("adds a Claude provider", async () => {
    addProviderMutateAsync.mockResolvedValueOnce(undefined);
    const { wrapper } = createWrapper();
    const providerInput = {
      name: "New Provider",
      settingsConfig: { env: {} },
    } as Omit<Provider, "id">;

    const { result } = renderHook(() => useProviderActions("claude"), {
      wrapper,
    });

    await act(async () => {
      await result.current.addProvider(providerInput);
    });

    expect(addProviderMutateAsync).toHaveBeenCalledWith(providerInput);
  });

  it("updates provider and tray menu", async () => {
    updateProviderMutateAsync.mockResolvedValueOnce(undefined);
    providersApiUpdateTrayMenuMock.mockResolvedValueOnce(true);
    const { wrapper } = createWrapper();
    const provider = createProvider();

    const { result } = renderHook(() => useProviderActions("claude"), {
      wrapper,
    });

    await act(async () => {
      await result.current.updateProvider(provider);
    });

    expect(updateProviderMutateAsync).toHaveBeenCalledWith({
      provider,
      originalId: undefined,
    });
    expect(providersApiUpdateTrayMenuMock).toHaveBeenCalledTimes(1);
  });

  it("warns but still switches proxy-required Claude providers when proxy is not running", async () => {
    switchProviderMutateAsync.mockResolvedValueOnce(undefined);
    const { wrapper } = createWrapper();
    const provider = createProvider({
      category: "custom",
      meta: { apiFormat: "openai_chat" },
    });

    const { result } = renderHook(() => useProviderActions("claude", false), {
      wrapper,
    });

    await act(async () => {
      await result.current.switchProvider(provider);
    });

    expect(toastWarningMock).toHaveBeenCalledTimes(1);
    expect(switchProviderMutateAsync).toHaveBeenCalledWith(provider.id);
  });

  it("syncs plugin config when switching Claude provider with integration enabled", async () => {
    switchProviderMutateAsync.mockResolvedValueOnce(undefined);
    settingsApiGetMock.mockResolvedValueOnce({
      enableClaudePluginIntegration: true,
    });
    settingsApiApplyMock.mockResolvedValueOnce(true);
    const { wrapper } = createWrapper();
    const provider = createProvider({ category: "official" });

    const { result } = renderHook(() => useProviderActions("claude"), {
      wrapper,
    });

    await act(async () => {
      await result.current.switchProvider(provider);
    });

    expect(switchProviderMutateAsync).toHaveBeenCalledWith(provider.id);
    expect(settingsApiGetMock).toHaveBeenCalledTimes(1);
    expect(settingsApiApplyMock).toHaveBeenCalledWith({ official: true });
  });

  it("does not call applyClaudePluginConfig when integration is disabled", async () => {
    switchProviderMutateAsync.mockResolvedValueOnce(undefined);
    settingsApiGetMock.mockResolvedValueOnce({
      enableClaudePluginIntegration: false,
    });
    const { wrapper } = createWrapper();

    const { result } = renderHook(() => useProviderActions("claude"), {
      wrapper,
    });

    await act(async () => {
      await result.current.switchProvider(createProvider());
    });

    expect(settingsApiGetMock).toHaveBeenCalledTimes(1);
    expect(settingsApiApplyMock).not.toHaveBeenCalled();
  });

  it("shows error toast when plugin sync fails", async () => {
    switchProviderMutateAsync.mockResolvedValueOnce(undefined);
    settingsApiGetMock.mockResolvedValueOnce({
      enableClaudePluginIntegration: true,
    });
    settingsApiApplyMock.mockRejectedValueOnce(new Error("Sync failed"));
    const { wrapper } = createWrapper();

    const { result } = renderHook(() => useProviderActions("claude"), {
      wrapper,
    });

    await act(async () => {
      await result.current.switchProvider(createProvider());
    });

    expect(toastErrorMock.mock.calls[0]?.[0]).toBe("Sync failed");
  });

  it("deletes a provider", async () => {
    deleteProviderMutateAsync.mockResolvedValueOnce(undefined);
    const { wrapper } = createWrapper();

    const { result } = renderHook(() => useProviderActions("claude"), {
      wrapper,
    });

    await act(async () => {
      await result.current.deleteProvider("provider-2");
    });

    expect(deleteProviderMutateAsync).toHaveBeenCalledWith("provider-2");
  });

  it("saves usage script and refreshes cache", async () => {
    providersApiUpdateMock.mockResolvedValueOnce(true);
    const { wrapper, queryClient } = createWrapper();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    const provider = createProvider();
    const script: UsageScript = {
      enabled: true,
      language: "javascript",
      code: "return { success: true };",
      timeout: 5,
    };

    const { result } = renderHook(() => useProviderActions("claude"), {
      wrapper,
    });

    await act(async () => {
      await result.current.saveUsageScript(provider, script);
    });

    expect(providersApiUpdateMock).toHaveBeenCalledWith(
      { ...provider, meta: { usage_script: script } },
      "claude",
    );
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["providers", "claude"],
    });
    expect(toastSuccessMock).toHaveBeenCalledTimes(1);
  });

  it("tracks pending state", () => {
    addProviderMutation.isPending = true;
    const { wrapper } = createWrapper();

    const { result } = renderHook(() => useProviderActions("claude"), {
      wrapper,
    });

    expect(result.current.isLoading).toBe(true);
  });
});
