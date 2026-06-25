import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RepoManager } from "@/components/skills/RepoManager";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) =>
      params ? `${key}:${JSON.stringify(params)}` : key,
  }),
}));

vi.mock("@/lib/api", () => ({
  settingsApi: {
    openExternal: vi.fn(),
  },
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({ children, ...props }: any) => <button {...props}>{children}</button>,
}));

vi.mock("@/components/ui/input", () => ({
  Input: (props: any) => <input {...props} />,
}));

vi.mock("@/components/ui/label", () => ({
  Label: ({ children, ...props }: any) => <label {...props}>{children}</label>,
}));

vi.mock("@/components/ui/dialog", () => ({
  Dialog: ({ open, children }: any) => (open ? <div>{children}</div> : null),
  DialogContent: ({ children }: any) => <div>{children}</div>,
  DialogDescription: ({ children }: any) => <p>{children}</p>,
  DialogHeader: ({ children }: any) => <div>{children}</div>,
  DialogTitle: ({ children }: any) => <h2>{children}</h2>,
}));

const repos = [
  { owner: "owner", name: "repo", branch: "main", enabled: true },
];

const skills = [
  {
    key: "skill:owner:repo",
    directory: "skill",
    name: "Skill",
    description: "",
    repoOwner: "owner",
    repoName: "repo",
    repoBranch: "main",
  },
];

describe("RepoManager", () => {
  it("adds, removes, and refreshes skill repositories", async () => {
    const onAdd = vi.fn().mockResolvedValue(undefined);
    const onRemove = vi.fn().mockResolvedValue(undefined);
    const onRefresh = vi.fn().mockResolvedValue(undefined);

    render(
      <RepoManager
        open={true}
        onOpenChange={() => {}}
        repos={repos as any}
        skills={skills as any}
        onAdd={onAdd}
        onRemove={onRemove}
        onRefresh={onRefresh}
      />,
    );

    expect(screen.getByText("owner/repo")).toBeInTheDocument();
    expect(
      screen.getByText('skills.repo.skillCount:{"count":1}'),
    ).toBeInTheDocument();

    fireEvent.change(screen.getByPlaceholderText("skills.repo.urlPlaceholder"), {
      target: { value: "https://github.com/new/repo.git" },
    });
    fireEvent.change(
      screen.getByPlaceholderText("skills.repo.branchPlaceholder"),
      {
        target: { value: "develop" },
      },
    );
    fireEvent.click(screen.getByRole("button", { name: /skills.repo.add/ }));

    await waitFor(() =>
      expect(onAdd).toHaveBeenCalledWith({
        owner: "new",
        name: "repo",
        branch: "develop",
        enabled: true,
      }),
    );

    fireEvent.click(screen.getByRole("button", { name: "common.refresh" }));
    await waitFor(() => expect(onRefresh).toHaveBeenCalledTimes(1));

    fireEvent.click(screen.getByTitle("common.delete"));
    expect(onRemove).toHaveBeenCalledWith("owner", "repo");
  });

  it("shows an error for invalid repository URLs", () => {
    render(
      <RepoManager
        open={true}
        onOpenChange={() => {}}
        repos={[]}
        skills={[]}
        onAdd={vi.fn()}
        onRemove={vi.fn()}
        onRefresh={vi.fn()}
      />,
    );

    fireEvent.change(screen.getByPlaceholderText("skills.repo.urlPlaceholder"), {
      target: { value: "not/a/valid/repo" },
    });
    fireEvent.click(screen.getByRole("button", { name: /skills.repo.add/ }));

    expect(screen.getByText("skills.repo.invalidUrl")).toBeInTheDocument();
  });
});
