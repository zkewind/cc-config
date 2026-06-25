import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SkillCard } from "@/components/skills/SkillCard";

const openExternalMock = vi.fn();

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

vi.mock("@/lib/api", () => ({
  settingsApi: {
    openExternal: (...args: unknown[]) => openExternalMock(...args),
  },
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({ children, ...props }: any) => <button {...props}>{children}</button>,
}));

vi.mock("@/components/ui/badge", () => ({
  Badge: ({ children, ...props }: any) => <span {...props}>{children}</span>,
}));

vi.mock("@/components/ui/card", () => ({
  Card: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  CardContent: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  CardDescription: ({ children, ...props }: any) => <p {...props}>{children}</p>,
  CardFooter: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  CardHeader: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  CardTitle: ({ children, ...props }: any) => <h3 {...props}>{children}</h3>,
}));

describe("SkillCard", () => {
  it("renders skill metadata and installs an available skill", async () => {
    const onInstall = vi.fn().mockResolvedValue(undefined);

    render(
      <SkillCard
        skill={{
          key: "skill:owner:repo",
          directory: "skill-dir",
          name: "Skill Name",
          description: "Useful skill",
          repoOwner: "owner",
          repoName: "repo",
          repoBranch: "main",
          readmeUrl: "https://example.com/readme",
          installed: false,
        }}
        installs={1234}
        onInstall={onInstall}
        onUninstall={vi.fn()}
      />,
    );

    expect(screen.getByText("Skill Name")).toBeInTheDocument();
    expect(screen.getByText("Useful skill")).toBeInTheDocument();
    expect(screen.getByText("owner/repo")).toBeInTheDocument();
    expect(screen.getByText("1,234")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /skills.view/ }));
    expect(openExternalMock).toHaveBeenCalledWith("https://example.com/readme");

    fireEvent.click(screen.getByRole("button", { name: /skills.install/ }));
    await waitFor(() => expect(onInstall).toHaveBeenCalledWith("skill-dir"));
  });

  it("uninstalls an installed skill", async () => {
    const onUninstall = vi.fn().mockResolvedValue(undefined);

    render(
      <SkillCard
        skill={{
          key: "skill:owner:repo",
          directory: "skill-dir",
          name: "Skill Name",
          description: "",
          repoOwner: "owner",
          repoName: "repo",
          repoBranch: "main",
          installed: true,
        }}
        onInstall={vi.fn()}
        onUninstall={onUninstall}
      />,
    );

    expect(screen.getByText("skills.installed")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /skills.uninstall/ }));

    await waitFor(() => expect(onUninstall).toHaveBeenCalledWith("skill-dir"));
  });
});
