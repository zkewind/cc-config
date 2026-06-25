import { beforeEach, describe, expect, it, vi } from "vitest";
import { checkForUpdate, relaunchApp } from "@/lib/updater";

const appMocks = vi.hoisted(() => ({
  getVersion: vi.fn(),
}));

const updaterMocks = vi.hoisted(() => ({
  check: vi.fn(),
}));

const processMocks = vi.hoisted(() => ({
  relaunch: vi.fn(),
}));

vi.mock("@tauri-apps/api/app", () => ({
  getVersion: appMocks.getVersion,
}));

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: updaterMocks.check,
}));

vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: processMocks.relaunch,
}));

describe("updater api", () => {
  beforeEach(() => {
    appMocks.getVersion.mockReset();
    updaterMocks.check.mockReset();
    processMocks.relaunch.mockReset();
  });

  it("returns up-to-date when plugin has no update", async () => {
    appMocks.getVersion.mockResolvedValue("1.0.0");
    updaterMocks.check.mockResolvedValue(null);

    await expect(checkForUpdate({ timeout: 1234 })).resolves.toEqual({
      status: "up-to-date",
    });

    expect(updaterMocks.check).toHaveBeenCalledWith({ timeout: 1234 });
  });

  it("maps update info and progress events", async () => {
    const rawUpdate = {
      version: "1.1.0",
      notes: "Release notes",
      date: "2026-06-07",
      downloadAndInstall: vi.fn(async (onProgress) => {
        onProgress({ event: "Started", data: { contentLength: 100 } });
        onProgress({ event: "Progress", data: { chunkLength: 40 } });
        onProgress({ event: "Finished" });
      }),
    };
    appMocks.getVersion.mockResolvedValue("1.0.0");
    updaterMocks.check.mockResolvedValue(rawUpdate);

    const result = await checkForUpdate();
    expect(result.status).toBe("available");
    if (result.status !== "available") throw new Error("expected update");

    expect(result.info).toEqual({
      currentVersion: "1.0.0",
      availableVersion: "1.1.0",
      notes: "Release notes",
      pubDate: "2026-06-07",
    });

    const progress: unknown[] = [];
    await result.update.downloadAndInstall((event) => progress.push(event));

    expect(progress).toEqual([
      { event: "Started", total: 100, downloaded: 0 },
      { event: "Progress", downloaded: 40 },
      { event: "Finished" },
    ]);
  });

  it("relaunches through process plugin", async () => {
    processMocks.relaunch.mockResolvedValue(undefined);

    await relaunchApp();

    expect(processMocks.relaunch).toHaveBeenCalledOnce();
  });
});
