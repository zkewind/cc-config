import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import UnifiedMcpPanel from "@/components/mcp/UnifiedMcpPanel";

const toggleMcpAppMock = vi.fn();
const deleteMcpServerMock = vi.fn();
const importMcpMock = vi.fn();

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) =>
      params ? `${key}:${JSON.stringify(params)}` : key,
  }),
}));

vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

vi.mock("@/hooks/useMcp", () => ({
  useAllMcpServers: () => ({
    isLoading: false,
    data: {
      fetch: {
        id: "fetch",
        name: "Fetch Server",
        description: "Fetch pages",
        server: { type: "stdio", command: "uvx" },
        apps: {
          claude: true,
        },
      },
    },
  }),
  useToggleMcpApp: () => ({
    mutateAsync: toggleMcpAppMock,
  }),
  useDeleteMcpServer: () => ({
    mutateAsync: deleteMcpServerMock,
  }),
  useImportMcpFromApps: () => ({
    mutateAsync: importMcpMock,
  }),
}));

vi.mock("@/lib/api", () => ({
  settingsApi: {
    openExternal: vi.fn(),
  },
}));

vi.mock("@/components/mcp/McpFormModal", () => ({
  default: ({ editingId }: any) => (
    <div data-testid="mcp-form-modal">{editingId ?? "new"}</div>
  ),
}));

vi.mock("@/components/ConfirmDialog", () => ({
  ConfirmDialog: ({ isOpen, message, onConfirm }: any) =>
    isOpen ? (
      <div data-testid="confirm-dialog">
        <span>{message}</span>
        <button type="button" onClick={onConfirm}>
          confirm
        </button>
      </div>
    ) : null,
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({ children, ...props }: any) => <button {...props}>{children}</button>,
}));

describe("UnifiedMcpPanel", () => {
  beforeEach(() => {
    toggleMcpAppMock.mockReset();
    deleteMcpServerMock.mockReset();
    importMcpMock.mockReset();
    deleteMcpServerMock.mockResolvedValue(undefined);
    importMcpMock.mockResolvedValue(1);
  });

  it("renders MCP server list and app counts", () => {
    render(<UnifiedMcpPanel onOpenChange={() => {}} />);

    expect(screen.getByText("Fetch Server")).toBeInTheDocument();
    expect(screen.getByText("Fetch pages")).toBeInTheDocument();
    expect(screen.getByText('mcp.serverCount:{"count":1}')).toBeInTheDocument();
    expect(screen.getByText("Claude:")).toBeInTheDocument();
  });

  it("opens edit form and deletes a server", async () => {
    render(<UnifiedMcpPanel onOpenChange={() => {}} />);

    fireEvent.click(screen.getByTitle("common.edit"));
    expect(screen.getByTestId("mcp-form-modal")).toHaveTextContent("fetch");

    fireEvent.click(screen.getByTitle("common.delete"));
    expect(screen.getByTestId("confirm-dialog")).toHaveTextContent("fetch");
    fireEvent.click(screen.getByText("confirm"));

    await waitFor(() => expect(deleteMcpServerMock).toHaveBeenCalledWith("fetch"));
  });
});
