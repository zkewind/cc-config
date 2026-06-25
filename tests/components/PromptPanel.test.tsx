import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import PromptPanel from "@/components/prompts/PromptPanel";

const reloadMock = vi.fn();
const savePromptMock = vi.fn();
const deletePromptMock = vi.fn();
const toggleEnabledMock = vi.fn();

let promptsState: Record<string, any> = {};
let loadingState = false;

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) =>
      params ? `${key}:${JSON.stringify(params)}` : key,
  }),
}));

vi.mock("@/hooks/usePromptActions", () => ({
  usePromptActions: () => ({
    prompts: promptsState,
    loading: loadingState,
    reload: reloadMock,
    savePrompt: savePromptMock,
    deletePrompt: deletePromptMock,
    toggleEnabled: toggleEnabledMock,
  }),
}));

vi.mock("@/components/prompts/PromptFormPanel", () => ({
  default: ({ editingId, initialData, onSave, onClose }: any) => (
    <div data-testid="prompt-form-panel">
      <span>{editingId ?? "new"}</span>
      <span>{initialData?.content}</span>
      <button
        type="button"
        onClick={() =>
          onSave("prompt-1", {
            id: "prompt-1",
            name: "Saved",
            content: "Saved content",
            enabled: true,
          })
        }
      >
        save-from-form
      </button>
      <button type="button" onClick={onClose}>
        close-form
      </button>
    </div>
  ),
}));

vi.mock("@/components/ConfirmDialog", () => ({
  ConfirmDialog: ({ isOpen, title, message, onConfirm, onCancel }: any) =>
    isOpen ? (
      <div data-testid="confirm-dialog">
        <span>{title}</span>
        <span>{message}</span>
        <button type="button" onClick={onConfirm}>
          confirm
        </button>
        <button type="button" onClick={onCancel}>
          cancel
        </button>
      </div>
    ) : null,
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({ children, ...props }: any) => <button {...props}>{children}</button>,
}));

describe("PromptPanel", () => {
  beforeEach(() => {
    reloadMock.mockReset();
    savePromptMock.mockReset();
    deletePromptMock.mockReset();
    toggleEnabledMock.mockReset();
    promptsState = {
      "prompt-1": {
        id: "prompt-1",
        name: "Review Style",
        description: "Code review voice",
        content: "Be concise",
        enabled: true,
      },
      "prompt-2": {
        id: "prompt-2",
        name: "Docs Style",
        content: "Write docs",
        enabled: false,
      },
    };
    loadingState = false;
  });

  it("loads and renders prompt list details when opened", async () => {
    render(
      <PromptPanel open={true} onOpenChange={() => {}} appId="claude" />,
    );

    await waitFor(() => expect(reloadMock).toHaveBeenCalledTimes(1));
    expect(screen.getByText("Review Style")).toBeInTheDocument();
    expect(screen.getByText("Code review voice")).toBeInTheDocument();
    expect(
      screen.getByText((content) =>
        content.includes('prompts.enabledName:{"name":"Review Style"}'),
      ),
    ).toBeInTheDocument();
  });

  it("toggles, edits, saves, and deletes prompts", async () => {
    render(
      <PromptPanel open={true} onOpenChange={() => {}} appId="claude" />,
    );

    fireEvent.click(
      screen.getAllByRole("switch").find((node) => {
        return node.getAttribute("aria-checked") === "false";
      })!,
    );
    expect(toggleEnabledMock).toHaveBeenCalledWith("prompt-2", true);

    fireEvent.click(screen.getAllByTitle("common.edit")[0]);
    expect(screen.getByTestId("prompt-form-panel")).toHaveTextContent(
      "Be concise",
    );
    fireEvent.click(screen.getByText("save-from-form"));
    expect(savePromptMock).toHaveBeenCalledWith(
      "prompt-1",
      expect.objectContaining({ name: "Saved" }),
    );

    fireEvent.click(screen.getAllByTitle("common.delete")[0]);
    expect(screen.getByTestId("confirm-dialog")).toHaveTextContent(
      "Review Style",
    );
    await act(async () => {
      fireEvent.click(screen.getByText("confirm"));
    });
    expect(deletePromptMock).toHaveBeenCalledWith("prompt-1");
  });
});
