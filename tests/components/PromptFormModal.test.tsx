import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import PromptFormModal from "@/components/prompts/PromptFormModal";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      if (key === "prompts.contentPlaceholder") {
        return `placeholder:${params?.filename}`;
      }
      return key;
    },
  }),
}));

vi.mock("@/components/MarkdownEditor", () => ({
  default: ({ value, onChange, placeholder }: any) => (
    <textarea
      aria-label="markdown-editor"
      value={value}
      placeholder={placeholder}
      onChange={(event) => onChange(event.target.value)}
    />
  ),
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
  Dialog: ({ children }: any) => <div>{children}</div>,
  DialogContent: ({ children }: any) => <div>{children}</div>,
  DialogFooter: ({ children }: any) => <div>{children}</div>,
  DialogHeader: ({ children }: any) => <div>{children}</div>,
  DialogTitle: ({ children }: any) => <h2>{children}</h2>,
}));

describe("PromptFormModal", () => {
  it("uses the Claude prompt filename in the markdown placeholder", () => {
    render(
      <PromptFormModal
        appId="claude"
        onSave={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    expect(
      screen.getByPlaceholderText("placeholder:CLAUDE.md"),
    ).toBeInTheDocument();
  });

  it("saves a trimmed prompt and preserves enabled state while editing", async () => {
    const onSave = vi.fn().mockResolvedValue(undefined);
    const onClose = vi.fn();

    render(
      <PromptFormModal
        appId="claude"
        editingId="prompt-1"
        initialData={{
          id: "prompt-1",
          name: "Old",
          description: "Old description",
          content: "Old content",
          enabled: true,
          createdAt: 100,
          updatedAt: 100,
        }}
        onSave={onSave}
        onClose={onClose}
      />,
    );

    fireEvent.change(screen.getByPlaceholderText("prompts.namePlaceholder"), {
      target: { value: "  New name  " },
    });
    fireEvent.change(
      screen.getByPlaceholderText("prompts.descriptionPlaceholder"),
      {
        target: { value: "  New description  " },
      },
    );
    fireEvent.change(screen.getByLabelText("markdown-editor"), {
      target: { value: "  New content  " },
    });
    fireEvent.click(screen.getByRole("button", { name: "common.save" }));

    await waitFor(() => expect(onSave).toHaveBeenCalledTimes(1));
    expect(onSave).toHaveBeenCalledWith(
      "prompt-1",
      expect.objectContaining({
        id: "prompt-1",
        name: "New name",
        description: "New description",
        content: "New content",
        enabled: true,
        createdAt: 100,
      }),
    );
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
