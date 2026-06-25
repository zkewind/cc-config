import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { useStreamCheck } from "@/hooks/useStreamCheck";
import type { StreamCheckResult } from "@/lib/api/model-test";

const toastSuccessMock = vi.fn();
const toastErrorMock = vi.fn();
const toastWarningMock = vi.fn();
const streamCheckProviderMock = vi.fn();

vi.mock("sonner", () => ({
  toast: {
    success: (...args: unknown[]) => toastSuccessMock(...args),
    error: (...args: unknown[]) => toastErrorMock(...args),
    warning: (...args: unknown[]) => toastWarningMock(...args),
  },
}));

vi.mock("@/lib/api/model-test", () => ({
  streamCheckProvider: (...args: unknown[]) =>
    streamCheckProviderMock(...args),
}));

function makeResult(
  overrides: Partial<StreamCheckResult> = {},
): StreamCheckResult {
  return {
    status: "operational",
    success: true,
    message: "Check succeeded",
    responseTimeMs: 120,
    httpStatus: 200,
    modelUsed: "claude-haiku-4-5-20251001",
    testedAt: Date.now(),
    retryCount: 0,
    ...overrides,
  };
}

beforeEach(() => {
  toastSuccessMock.mockReset();
  toastErrorMock.mockReset();
  toastWarningMock.mockReset();
  streamCheckProviderMock.mockReset();
});

describe("useStreamCheck", () => {
  it("returns checkProvider and isChecking helpers", () => {
    const { result } = renderHook(() => useStreamCheck("claude"));
    expect(typeof result.current.checkProvider).toBe("function");
    expect(typeof result.current.isChecking).toBe("function");
  });

  it("calls streamCheckProvider with Claude appId and providerId", async () => {
    streamCheckProviderMock.mockResolvedValueOnce(makeResult());
    const { result } = renderHook(() => useStreamCheck("claude"));

    await act(async () => {
      await result.current.checkProvider("provider-1", "Anthropic");
    });

    expect(streamCheckProviderMock).toHaveBeenCalledWith("claude", "provider-1");
  });

  it("shows success toast when status is operational", async () => {
    streamCheckProviderMock.mockResolvedValueOnce(
      makeResult({ status: "operational", responseTimeMs: 200 }),
    );
    const { result } = renderHook(() => useStreamCheck("claude"));

    await act(async () => {
      await result.current.checkProvider("p1", "MyProvider");
    });

    expect(toastSuccessMock).toHaveBeenCalledTimes(1);
    expect(toastErrorMock).not.toHaveBeenCalled();
    expect(toastWarningMock).not.toHaveBeenCalled();
  });

  it("shows warning toast when status is degraded", async () => {
    streamCheckProviderMock.mockResolvedValueOnce(
      makeResult({ status: "degraded", responseTimeMs: 8000 }),
    );
    const { result } = renderHook(() => useStreamCheck("claude"));

    await act(async () => {
      await result.current.checkProvider("p1", "SlowProvider");
    });

    expect(toastWarningMock).toHaveBeenCalledTimes(1);
    expect(toastSuccessMock).not.toHaveBeenCalled();
  });

  it("shows error toast when errorCategory is modelNotFound", async () => {
    streamCheckProviderMock.mockResolvedValueOnce(
      makeResult({
        status: "failed",
        success: false,
        errorCategory: "modelNotFound",
        modelUsed: "bad-model",
        httpStatus: 404,
      }),
    );
    const { result } = renderHook(() => useStreamCheck("claude"));

    await act(async () => {
      await result.current.checkProvider("p1", "BadProvider");
    });

    expect(toastErrorMock).toHaveBeenCalledTimes(1);
    expect(toastSuccessMock).not.toHaveBeenCalled();
  });

  it("returns null and shows error toast when invoke throws", async () => {
    streamCheckProviderMock.mockRejectedValueOnce(new Error("IPC error"));
    const { result } = renderHook(() => useStreamCheck("claude"));

    let returned: StreamCheckResult | null | undefined;
    await act(async () => {
      returned = await result.current.checkProvider("p1", "CrashedProvider");
    });

    expect(returned).toBeNull();
    expect(toastErrorMock).toHaveBeenCalledTimes(1);
    expect(result.current.isChecking("p1")).toBe(false);
  });
});
