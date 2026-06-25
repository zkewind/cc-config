import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { ModelInputWithFetch } from "@/components/providers/forms/shared/ModelInputWithFetch";
import type { FetchedModel } from "@/lib/api/model-fetch";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) =>
      (params?.defaultValue as string) ?? key,
  }),
}));

vi.mock("@/components/ui/input", () => ({
  Input: (props: React.InputHTMLAttributes<HTMLInputElement>) => (
    <input {...props} />
  ),
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({
    children,
    onClick,
    disabled,
    title,
    type,
  }: {
    children: React.ReactNode;
    onClick?: () => void;
    disabled?: boolean;
    title?: string;
    type?: "button" | "submit" | "reset";
  }) => (
    <button onClick={onClick} disabled={disabled} title={title} type={type}>
      {children}
    </button>
  ),
}));

vi.mock("@/components/ui/dropdown-menu", () => ({
  DropdownMenu: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
  DropdownMenuTrigger: ({
    children,
  }: {
    children: React.ReactNode;
    asChild?: boolean;
  }) => <div data-testid="dropdown-trigger">{children}</div>,
  DropdownMenuContent: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="dropdown-content">{children}</div>
  ),
  DropdownMenuItem: ({
    children,
    onSelect,
  }: {
    children: React.ReactNode;
    onSelect?: () => void;
  }) => (
    <div role="menuitem" onClick={onSelect}>
      {children}
    </div>
  ),
  DropdownMenuLabel: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="dropdown-label">{children}</div>
  ),
  DropdownMenuSeparator: () => <hr />,
}));

vi.mock("lucide-react", () => ({
  ChevronDown: () => <span data-testid="icon-chevron-down" />,
  Download: () => <span data-testid="icon-download" />,
  Loader2: () => <span data-testid="icon-loader" />,
}));

const defaultProps = {
  id: "model-input",
  value: "",
  onChange: vi.fn(),
  fetchedModels: [] as FetchedModel[],
  isLoading: false,
};

describe("ModelInputWithFetch", () => {
  describe("无 onFetch、无数据、非加载中：纯 Input 模式", () => {
    it("渲染单个输入框，无按钮", () => {
      render(<ModelInputWithFetch {...defaultProps} />);

      expect(screen.getByRole("textbox")).toBeInTheDocument();
      expect(screen.queryByRole("button")).not.toBeInTheDocument();
    });

    it("输入框显示 value", () => {
      render(<ModelInputWithFetch {...defaultProps} value="claude-3-sonnet" />);
      expect(screen.getByRole("textbox")).toHaveValue("claude-3-sonnet");
    });

    it("输入框显示 placeholder", () => {
      render(
        <ModelInputWithFetch
          {...defaultProps}
          placeholder="输入模型名称"
        />,
      );
      expect(
        screen.getByPlaceholderText("输入模型名称"),
      ).toBeInTheDocument();
    });

    it("用户输入时调用 onChange", () => {
      const handleChange = vi.fn();
      render(
        <ModelInputWithFetch {...defaultProps} onChange={handleChange} />,
      );
      fireEvent.change(screen.getByRole("textbox"), {
        target: { value: "gpt-4" },
      });
      expect(handleChange).toHaveBeenCalledWith("gpt-4");
    });
  });

  describe("有 onFetch、无数据、非加载中：获取按钮模式", () => {
    it("渲染获取按钮（Download 图标）", () => {
      render(
        <ModelInputWithFetch
          {...defaultProps}
          onFetch={vi.fn()}
        />,
      );

      expect(screen.getByTestId("icon-download")).toBeInTheDocument();
    });

    it("点击按钮调用 onFetch", () => {
      const handleFetch = vi.fn();
      render(
        <ModelInputWithFetch
          {...defaultProps}
          onFetch={handleFetch}
        />,
      );

      const button = screen.getByRole("button");
      fireEvent.click(button);
      expect(handleFetch).toHaveBeenCalledTimes(1);
    });

    it("按钮 title 为 i18n key providerForm.fetchModels", () => {
      render(
        <ModelInputWithFetch
          {...defaultProps}
          onFetch={vi.fn()}
        />,
      );
      // t() mock 返回 key 本身
      expect(screen.getByRole("button")).toHaveAttribute(
        "title",
        "providerForm.fetchModels",
      );
    });

    it("按钮 type 为 button（防止触发表单提交）", () => {
      render(
        <ModelInputWithFetch
          {...defaultProps}
          onFetch={vi.fn()}
        />,
      );
      expect(screen.getByRole("button")).toHaveAttribute("type", "button");
    });
  });

  describe("isLoading=true：加载中模式（Spinner 按钮，禁用）", () => {
    it("渲染 Loader2 图标", () => {
      render(
        <ModelInputWithFetch
          {...defaultProps}
          isLoading={true}
          onFetch={vi.fn()}
        />,
      );
      expect(screen.getByTestId("icon-loader")).toBeInTheDocument();
    });

    it("按钮处于禁用状态", () => {
      render(
        <ModelInputWithFetch
          {...defaultProps}
          isLoading={true}
          onFetch={vi.fn()}
        />,
      );
      expect(screen.getByRole("button")).toBeDisabled();
    });

    it("fetchedModels 非空时下拉优先于 isLoading（组件渲染顺序：fetchedModels > isLoading）", () => {
      const models: FetchedModel[] = [{ id: "gpt-4", ownedBy: "openai" }];
      render(
        <ModelInputWithFetch
          {...defaultProps}
          isLoading={true}
          fetchedModels={models}
          onFetch={vi.fn()}
        />,
      );
      // fetchedModels 分支优先级高于 isLoading，显示下拉
      expect(screen.getByTestId("icon-chevron-down")).toBeInTheDocument();
      expect(screen.queryByTestId("icon-loader")).not.toBeInTheDocument();
    });
  });

  describe("有 fetchedModels：下拉选择模式", () => {
    const models: FetchedModel[] = [
      { id: "gpt-4", ownedBy: "openai" },
      { id: "gpt-3.5-turbo", ownedBy: "openai" },
      { id: "claude-3-sonnet", ownedBy: "anthropic" },
    ];

    it("渲染 ChevronDown 图标（下拉触发器）", () => {
      render(
        <ModelInputWithFetch
          {...defaultProps}
          fetchedModels={models}
        />,
      );
      expect(screen.getByTestId("icon-chevron-down")).toBeInTheDocument();
    });

    it("按供应商分组展示模型 label", () => {
      render(
        <ModelInputWithFetch
          {...defaultProps}
          fetchedModels={models}
        />,
      );
      const labels = screen.getAllByTestId("dropdown-label");
      const labelTexts = labels.map((l) => l.textContent);
      expect(labelTexts).toContain("anthropic");
      expect(labelTexts).toContain("openai");
    });

    it("展示所有模型 id 作为菜单项", () => {
      render(
        <ModelInputWithFetch
          {...defaultProps}
          fetchedModels={models}
        />,
      );
      expect(screen.getByText("gpt-4")).toBeInTheDocument();
      expect(screen.getByText("gpt-3.5-turbo")).toBeInTheDocument();
      expect(screen.getByText("claude-3-sonnet")).toBeInTheDocument();
    });

    it("点击模型项调用 onChange 并传入模型 id", () => {
      const handleChange = vi.fn();
      render(
        <ModelInputWithFetch
          {...defaultProps}
          fetchedModels={models}
          onChange={handleChange}
        />,
      );
      fireEvent.click(screen.getByText("gpt-4"));
      expect(handleChange).toHaveBeenCalledWith("gpt-4");
    });

    it("ownedBy 为 null 时归入 Other 分组", () => {
      const modelsWithNull: FetchedModel[] = [
        { id: "unknown-model", ownedBy: null },
      ];
      render(
        <ModelInputWithFetch
          {...defaultProps}
          fetchedModels={modelsWithNull}
        />,
      );
      expect(screen.getByTestId("dropdown-label")).toHaveTextContent("Other");
    });

    it("输入框仍可手动编辑", () => {
      const handleChange = vi.fn();
      render(
        <ModelInputWithFetch
          {...defaultProps}
          fetchedModels={models}
          onChange={handleChange}
        />,
      );
      fireEvent.change(screen.getByRole("textbox"), {
        target: { value: "my-custom-model" },
      });
      expect(handleChange).toHaveBeenCalledWith("my-custom-model");
    });
  });

  describe("优先级：fetchedModels > isLoading > onFetch > 纯 Input", () => {
    it("有 fetchedModels 时显示下拉，即使传了 onFetch", () => {
      const models: FetchedModel[] = [{ id: "gpt-4", ownedBy: "openai" }];
      render(
        <ModelInputWithFetch
          {...defaultProps}
          fetchedModels={models}
          onFetch={vi.fn()}
        />,
      );
      expect(screen.getByTestId("icon-chevron-down")).toBeInTheDocument();
      expect(screen.queryByTestId("icon-download")).not.toBeInTheDocument();
    });
  });
});
