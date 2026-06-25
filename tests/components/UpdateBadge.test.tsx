import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { UpdateBadge } from "@/components/UpdateBadge";

const updateMock = vi.hoisted(() =>
  vi.fn((): any => ({
    hasUpdate: false,
    updateInfo: null,
  })),
);

vi.mock("@/contexts/UpdateContext", () => ({
  useUpdate: () => updateMock(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { version?: string }) =>
      key === "settings.updateAvailable"
        ? `Update ${options?.version ?? ""}`
        : key,
  }),
}));

describe("UpdateBadge", () => {
  it("renders nothing when no update is available", () => {
    updateMock.mockReturnValueOnce({ hasUpdate: false, updateInfo: null });

    const { container } = render(<UpdateBadge />);

    expect(container).toBeEmptyDOMElement();
  });

  it("renders available update and calls click handler", () => {
    const onClick = vi.fn();
    updateMock.mockReturnValueOnce({
      hasUpdate: true,
      updateInfo: {
        currentVersion: "1.0.0",
        availableVersion: "1.1.0",
      },
    });

    render(<UpdateBadge onClick={onClick} />);
    const button = screen.getByRole("button", { name: "Update 1.1.0" });

    fireEvent.click(button);

    expect(button).toHaveAttribute("title", "Update 1.1.0");
    expect(onClick).toHaveBeenCalledOnce();
  });
});
