import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { HealthStatusIndicator } from "@/components/providers/HealthStatusIndicator";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) =>
      (params?.defaultValue as string) ?? key,
  }),
}));

describe("HealthStatusIndicator", () => {
  it("operational 状态: 显示正常标签，绿色圆点", () => {
    const { container } = render(<HealthStatusIndicator status="operational" />);
    expect(screen.getByText("正常")).toBeInTheDocument();
    const dot = container.querySelector(".bg-emerald-500");
    expect(dot).toBeInTheDocument();
    expect(dot).not.toHaveClass("animate-pulse");
  });

  it("degraded 状态: 显示降级标签，黄色圆点", () => {
    const { container } = render(<HealthStatusIndicator status="degraded" />);
    expect(screen.getByText("降级")).toBeInTheDocument();
    const dot = container.querySelector(".bg-yellow-500");
    expect(dot).toBeInTheDocument();
    expect(dot).not.toHaveClass("animate-pulse");
  });

  it("failed 状态: 显示失败标签，红色圆点", () => {
    const { container } = render(<HealthStatusIndicator status="failed" />);
    expect(screen.getByText("失败")).toBeInTheDocument();
    const dot = container.querySelector(".bg-red-500");
    expect(dot).toBeInTheDocument();
    expect(dot).not.toHaveClass("animate-pulse");
  });

  it("checking 状态: 显示检测中标签，蓝色圆点带 animate-pulse", () => {
    const { container } = render(<HealthStatusIndicator status="checking" />);
    expect(screen.getByText("检测中")).toBeInTheDocument();
    const dot = container.querySelector(".bg-blue-400");
    expect(dot).toBeInTheDocument();
    expect(dot).toHaveClass("animate-pulse");
  });

  it("传入 responseTimeMs 时在标签后显示响应时间", () => {
    render(<HealthStatusIndicator status="operational" responseTimeMs={123} />);
    expect(screen.getByText("正常 (123ms)")).toBeInTheDocument();
  });

  it("不传 responseTimeMs 时不显示括号内容", () => {
    render(<HealthStatusIndicator status="operational" />);
    expect(screen.queryByText(/ms/)).not.toBeInTheDocument();
  });

  it("传入 className 时附加到容器", () => {
    const { container } = render(
      <HealthStatusIndicator status="operational" className="my-custom-class" />,
    );
    expect(container.firstChild).toHaveClass("my-custom-class");
  });
});
