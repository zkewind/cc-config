import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { SkillStorageLocationSettings } from "@/components/settings/SkillStorageLocationSettings";
import { SkillSyncMethodSettings } from "@/components/settings/SkillSyncMethodSettings";

const toastSuccessMock = vi.fn();
const toastWarningMock = vi.fn();
const toastErrorMock = vi.fn();
const migrateStorageMock = vi.fn();

vi.mock("sonner", () => ({
  toast: {
    success: (...args: unknown[]) => toastSuccessMock(...args),
    warning: (...args: unknown[]) => toastWarningMock(...args),
    error: (...args: unknown[]) => toastErrorMock(...args),
  },
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) =>
      params ? `${key}:${JSON.stringify(params)}` : key,
  }),
}));

vi.mock("@/lib/api/skills", () => ({
  skillsApi: {
    migrateStorage: (...args: unknown[]) => migrateStorageMock(...args),
  },
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({ children, ...props }: any) => <button {...props}>{children}</button>,
}));

vi.mock("@/components/ui/dialog", () => ({
  Dialog: ({ open, children }: any) => (open ? <div>{children}</div> : null),
  DialogContent: ({ children }: any) => <div>{children}</div>,
  DialogDescription: ({ children }: any) => <p>{children}</p>,
  DialogFooter: ({ children }: any) => <div>{children}</div>,
  DialogHeader: ({ children }: any) => <div>{children}</div>,
  DialogTitle: ({ children }: any) => <h2>{children}</h2>,
}));

describe("skill settings", () => {
  beforeEach(() => {
    toastSuccessMock.mockReset();
    toastWarningMock.mockReset();
    toastErrorMock.mockReset();
    migrateStorageMock.mockReset();
    migrateStorageMock.mockResolvedValue({ migratedCount: 2, errors: [] });
  });

  it("migrates storage location after confirmation when skills exist", async () => {
    const onMigrated = vi.fn();

    render(
      <SkillStorageLocationSettings
        value="cc_switch"
        installedCount={2}
        onMigrated={onMigrated}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "settings.skillStorage.unified" }));
    expect(
      screen.getByText('settings.skillStorage.confirmMessage:{"count":2}'),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "common.confirm" }));

    await waitFor(() => expect(migrateStorageMock).toHaveBeenCalledWith("unified"));
    expect(onMigrated).toHaveBeenCalledWith("unified");
    expect(toastSuccessMock).toHaveBeenCalledWith(
      'settings.skillStorage.migrationSuccess:{"count":2}',
    );
  });

  it("switches sync method between symlink and copy", () => {
    const onChange = vi.fn();

    render(<SkillSyncMethodSettings value="symlink" onChange={onChange} />);

    fireEvent.click(screen.getByRole("button", { name: "settings.skillSync.copy" }));

    expect(onChange).toHaveBeenCalledWith("copy");
    expect(screen.getByText("settings.skillSync.symlinkHint")).toBeInTheDocument();
  });
});
