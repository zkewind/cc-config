import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { EndpointField } from "@/components/providers/forms/shared/EndpointField";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) =>
      (params?.defaultValue as string) ?? key,
  }),
}));

vi.mock("@/components/ui/form", () => ({
  FormLabel: ({ children, htmlFor }: { children: React.ReactNode; htmlFor?: string }) => (
    <label htmlFor={htmlFor}>{children}</label>
  ),
}));

vi.mock("@/components/ui/input", () => ({
  Input: (props: React.InputHTMLAttributes<HTMLInputElement>) => <input {...props} />,
}));

vi.mock("@/components/ui/switch", () => ({
  Switch: ({
    checked,
    onCheckedChange,
    "aria-label": ariaLabel,
  }: {
    checked: boolean;
    onCheckedChange?: (value: boolean) => void;
    "aria-label"?: string;
  }) => (
    <button
      role="switch"
      aria-checked={checked}
      aria-label={ariaLabel}
      onClick={() => onCheckedChange?.(!checked)}
    />
  ),
}));

describe("EndpointField", () => {
  it("渲染标签和输入框", () => {
    render(
      <EndpointField
        id="endpoint"
        label="API 端点"
        value=""
        onChange={vi.fn()}
        placeholder="https://api.example.com"
      />,
    );

    expect(screen.getByLabelText("API 端点")).toBeInTheDocument();
    expect(
      screen.getByPlaceholderText("https://api.example.com"),
    ).toBeInTheDocument();
  });

  it("输入框显示传入的 value", () => {
    render(
      <EndpointField
        id="endpoint"
        label="API 端点"
        value="https://custom.api.com"
        onChange={vi.fn()}
        placeholder=""
      />,
    );

    const input = screen.getByRole("textbox");
    expect(input).toHaveValue("https://custom.api.com");
  });

  it("用户输入时调用 onChange", () => {
    const handleChange = vi.fn();
    render(
      <EndpointField
        id="endpoint"
        label="API 端点"
        value=""
        onChange={handleChange}
        placeholder=""
      />,
    );

    fireEvent.change(screen.getByRole("textbox"), {
      target: { value: "https://new-endpoint.example.com" },
    });

    expect(handleChange).toHaveBeenCalledWith("https://new-endpoint.example.com");
  });

  it("显示 hint 提示文字", () => {
    render(
      <EndpointField
        id="endpoint"
        label="API 端点"
        value=""
        onChange={vi.fn()}
        placeholder=""
        hint="请填写正确的端点地址"
      />,
    );

    expect(screen.getByText("请填写正确的端点地址")).toBeInTheDocument();
  });

  it("showManageButton=true 且提供 onManageClick 时显示管理按钮", () => {
    const handleManage = vi.fn();
    render(
      <EndpointField
        id="endpoint"
        label="API 端点"
        value=""
        onChange={vi.fn()}
        placeholder=""
        showManageButton
        onManageClick={handleManage}
        manageButtonLabel="测速"
      />,
    );

    const btn = screen.getByRole("button", { name: /测速/ });
    expect(btn).toBeInTheDocument();
    fireEvent.click(btn);
    expect(handleManage).toHaveBeenCalledTimes(1);
  });

  it("showManageButton=false 时不显示管理按钮", () => {
    render(
      <EndpointField
        id="endpoint"
        label="API 端点"
        value=""
        onChange={vi.fn()}
        placeholder=""
        showManageButton={false}
        onManageClick={vi.fn()}
      />,
    );

    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("showFullUrlToggle=true 时显示 Switch 并可切换", () => {
    const handleFullUrlChange = vi.fn();
    render(
      <EndpointField
        id="endpoint"
        label="API 端点"
        value=""
        onChange={vi.fn()}
        placeholder=""
        showFullUrlToggle
        isFullUrl={false}
        onFullUrlChange={handleFullUrlChange}
        showManageButton={false}
      />,
    );

    const toggle = screen.getByRole("switch");
    expect(toggle).toHaveAttribute("aria-checked", "false");
    fireEvent.click(toggle);
    expect(handleFullUrlChange).toHaveBeenCalledWith(true);
  });

  it("isFullUrl=true 时显示 fullUrlHint 而非普通 hint", () => {
    render(
      <EndpointField
        id="endpoint"
        label="API 端点"
        value=""
        onChange={vi.fn()}
        placeholder=""
        showFullUrlToggle
        isFullUrl={true}
        onFullUrlChange={vi.fn()}
        hint="普通提示"
        fullUrlHint="完整URL提示"
        showManageButton={false}
      />,
    );

    expect(screen.getByText("完整URL提示")).toBeInTheDocument();
    expect(screen.queryByText("普通提示")).not.toBeInTheDocument();
  });
});
