import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SessionMessageItem } from "@/components/sessions/SessionMessageItem";
import {
  SessionTocDialog,
  SessionTocSidebar,
} from "@/components/sessions/SessionToc";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (
      key: string,
      options?: {
        defaultValue?: string;
      },
    ) => options?.defaultValue ?? key,
  }),
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({ children, ...props }: any) => <button {...props}>{children}</button>,
}));

vi.mock("@/components/ui/tooltip", () => ({
  Tooltip: ({ children }: any) => <>{children}</>,
  TooltipTrigger: ({ children }: any) => <>{children}</>,
  TooltipContent: ({ children }: any) => <span>{children}</span>,
}));

vi.mock("@/components/ui/scroll-area", () => ({
  ScrollArea: ({ children }: any) => <div>{children}</div>,
}));

vi.mock("@/components/ui/dialog", () => ({
  Dialog: ({ open, children }: any) => (open ? <div>{children}</div> : null),
  DialogTrigger: ({ children }: any) => <>{children}</>,
  DialogContent: ({ children }: any) => <div>{children}</div>,
  DialogHeader: ({ children }: any) => <div>{children}</div>,
  DialogTitle: ({ children }: any) => <h2>{children}</h2>,
  DialogClose: ({ children, ...props }: any) => (
    <button type="button" {...props}>
      {children}
    </button>
  ),
}));

describe("iter-4 session management components", () => {
  it("renders message content, copies full text, and expands long messages", () => {
    const onCopy = vi.fn();
    const longContent = `start ${"x".repeat(3200)} end`;

    render(
      <SessionMessageItem
        message={{ role: "assistant", content: longContent, ts: 1_780_000_000 }}
        isActive
        onCopy={onCopy}
      />,
    );

    expect(screen.getByText(/start/)).toBeInTheDocument();
    expect(screen.queryByText(/end$/)).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /展开/ }));
    expect(screen.getByText(/end$/)).toBeInTheDocument();

    const copyButton = screen
      .getAllByRole("button")
      .find((button) => button.querySelector(".lucide-copy"));
    expect(copyButton).toBeDefined();
    fireEvent.click(copyButton!);

    expect(onCopy).toHaveBeenCalledWith(longContent);
  });

  it("keeps matching long messages expanded while searching", () => {
    render(
      <SessionMessageItem
        message={{
          role: "user",
          content: `needle ${"x".repeat(3200)} visible-tail`,
        }}
        isActive={false}
        searchQuery="visible-tail"
        onCopy={() => {}}
      />,
    );

    expect(screen.getByText(/visible-tail/)).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /展开/ }),
    ).not.toBeInTheDocument();
  });

  it("renders TOC entries and jumps to the selected message index", () => {
    const onItemClick = vi.fn();
    const items = [
      { index: 0, preview: "First prompt" },
      { index: 1, preview: "Second reply" },
      { index: 2, preview: "Third prompt" },
    ];

    render(
      <>
        <SessionTocSidebar items={items} onItemClick={onItemClick} />
        <SessionTocDialog
          items={items}
          onItemClick={onItemClick}
          open
          onOpenChange={() => {}}
        />
      </>,
    );

    fireEvent.click(screen.getAllByRole("button", { name: /Second reply/ })[0]);
    fireEvent.click(screen.getAllByRole("button", { name: /Third prompt/ })[1]);

    expect(onItemClick).toHaveBeenNthCalledWith(1, 1);
    expect(onItemClick).toHaveBeenNthCalledWith(2, 2);
  });

  it("does not render TOC for short sessions", () => {
    const { container } = render(
      <SessionTocSidebar
        items={[
          { index: 0, preview: "one" },
          { index: 1, preview: "two" },
        ]}
        onItemClick={() => {}}
      />,
    );

    expect(container).toBeEmptyDOMElement();
  });
});
