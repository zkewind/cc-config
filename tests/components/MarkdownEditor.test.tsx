import { render, screen, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import MarkdownEditor from "@/components/MarkdownEditor";

vi.mock("codemirror", () => ({
  basicSetup: {},
  EditorView: Object.assign(
    vi.fn().mockImplementation(({ state }) => ({
      state: {
        doc: {
          toString: () => state.doc,
          length: state.doc.length,
        },
        update: vi.fn((transaction) => transaction),
      },
      dispatch: vi.fn(),
      destroy: vi.fn(),
    })),
    {
      baseTheme: () => ({}),
      lineWrapping: {},
      updateListener: { of: () => ({}) },
      theme: () => ({}),
    },
  ),
}));

vi.mock("@codemirror/lang-markdown", () => ({
  markdown: () => ({}),
}));

vi.mock("@codemirror/theme-one-dark", () => ({
  oneDark: {},
}));

vi.mock("@codemirror/state", () => ({
  EditorState: {
    create: ({ doc, extensions }: any) => ({ doc, extensions }),
    readOnly: { of: () => ({}) },
  },
}));

vi.mock("@codemirror/view", () => {
  return {
    placeholder: () => ({}),
  };
});

describe("MarkdownEditor", () => {
  it("renders markdown content in preview mode", () => {
    render(
      <MarkdownEditor
        value={"# Release notes\n\nUse **bold** and `code`."}
        onChange={() => {}}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "markdown.preview" }));

    expect(
      screen.getByRole("heading", { name: "Release notes", level: 1 }),
    ).toBeInTheDocument();
    expect(screen.getByText("bold")).toHaveClass("font-semibold");
    expect(screen.getByText("code")).toHaveClass("font-mono");
  });

  it("returns to editable mode after preview", () => {
    render(<MarkdownEditor value="Text" onChange={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: "markdown.preview" }));
    fireEvent.click(screen.getByRole("button", { name: "markdown.edit" }));

    expect(screen.queryByText("Text")).not.toBeInTheDocument();
  });
});
