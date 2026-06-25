import React, { useEffect, useMemo, useRef, useState } from "react";
import { EditorView, basicSetup } from "codemirror";
import { markdown } from "@codemirror/lang-markdown";
import { oneDark } from "@codemirror/theme-one-dark";
import { EditorState } from "@codemirror/state";
import { placeholder as placeholderExt } from "@codemirror/view";
import { useTranslation } from "react-i18next";

interface MarkdownEditorProps {
  value: string;
  onChange?: (value: string) => void;
  placeholder?: string;
  darkMode?: boolean;
  readOnly?: boolean;
  className?: string;
  minHeight?: string;
  maxHeight?: string;
}

const MarkdownEditor: React.FC<MarkdownEditorProps> = ({
  value,
  onChange,
  placeholder: placeholderText = "",
  darkMode = false,
  readOnly = false,
  className = "",
  minHeight = "300px",
  maxHeight,
}) => {
  const { t } = useTranslation();
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const [mode, setMode] = useState<"edit" | "preview">(
    readOnly ? "preview" : "edit",
  );
  const previewNodes = useMemo(() => renderMarkdownPreview(value), [value]);

  useEffect(() => {
    if (!editorRef.current || mode !== "edit") return;

    const baseTheme = EditorView.baseTheme({
      "&": {
        height: "100%",
        minHeight,
        maxHeight: maxHeight || "none",
      },
      ".cm-scroller": {
        overflow: "auto",
        fontFamily:
          "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace",
        fontSize: "14px",
      },
      "&light .cm-content, &dark .cm-content": {
        padding: "12px 0",
      },
      "&light .cm-editor, &dark .cm-editor": {
        backgroundColor: "transparent",
      },
      "&.cm-focused": {
        outline: "none",
      },
    });

    const extensions = [
      basicSetup,
      markdown(),
      baseTheme,
      EditorView.lineWrapping,
      EditorState.readOnly.of(readOnly),
    ];

    if (!readOnly) {
      extensions.push(
        placeholderExt(placeholderText),
        EditorView.updateListener.of((update) => {
          if (update.docChanged && onChange) {
            onChange(update.state.doc.toString());
          }
        }),
      );
    } else {
      extensions.push(
        EditorView.theme({
          ".cm-cursor, .cm-dropCursor": { border: "none" },
          ".cm-activeLine": { backgroundColor: "transparent !important" },
          ".cm-activeLineGutter": { backgroundColor: "transparent !important" },
        }),
      );
    }

    if (darkMode) {
      extensions.push(oneDark);
    } else {
      extensions.push(
        EditorView.theme(
          {
            "&": {
              backgroundColor: "transparent",
            },
            ".cm-content": {
              color: "#374151",
            },
            ".cm-gutters": {
              backgroundColor: "#f9fafb",
              color: "#9ca3af",
              borderRight: "1px solid #e5e7eb",
            },
            ".cm-activeLineGutter": {
              backgroundColor: "#e5e7eb",
            },
          },
          { dark: false },
        ),
      );
    }

    const state = EditorState.create({
      doc: value,
      extensions,
    });

    const view = new EditorView({
      state,
      parent: editorRef.current,
    });

    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
  }, [darkMode, readOnly, minHeight, maxHeight, placeholderText, mode]);

  useEffect(() => {
    if (
      mode === "edit" &&
      viewRef.current &&
      viewRef.current.state.doc.toString() !== value
    ) {
      const transaction = viewRef.current.state.update({
        changes: {
          from: 0,
          to: viewRef.current.state.doc.length,
          insert: value,
        },
      });
      viewRef.current.dispatch(transaction);
    }
  }, [value, mode]);

  return (
    <div
      className={`border rounded-md overflow-hidden ${
        darkMode ? "border-gray-800" : "border-gray-200"
      } ${className}`}
    >
      {!readOnly && (
        <div
          className={`flex items-center justify-end gap-1 border-b px-2 py-1 ${
            darkMode
              ? "border-gray-800 bg-gray-900/60"
              : "border-gray-200 bg-gray-50"
          }`}
        >
          <ModeButton active={mode === "edit"} onClick={() => setMode("edit")}>
            {t("markdown.edit")}
          </ModeButton>
          <ModeButton
            active={mode === "preview"}
            onClick={() => setMode("preview")}
          >
            {t("markdown.preview")}
          </ModeButton>
        </div>
      )}

      {mode === "edit" ? (
        <div ref={editorRef} />
      ) : (
        <div
          className="space-y-3 overflow-auto px-4 py-3 text-sm text-foreground"
          style={{ minHeight, maxHeight }}
        >
          {previewNodes.length > 0 ? (
            previewNodes
          ) : (
            <p className="text-muted-foreground">{placeholderText}</p>
          )}
        </div>
      )}
    </div>
  );
};

interface ModeButtonProps {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}

function ModeButton({ active, onClick, children }: ModeButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`rounded px-2 py-1 text-xs transition-colors ${
        active
          ? "bg-background text-foreground shadow-sm"
          : "text-muted-foreground hover:text-foreground"
      }`}
    >
      {children}
    </button>
  );
}

function renderMarkdownPreview(markdownText: string): React.ReactNode[] {
  const lines = markdownText.replace(/\r\n/g, "\n").split("\n");
  const nodes: React.ReactNode[] = [];
  let paragraph: string[] = [];

  const flushParagraph = () => {
    if (paragraph.length === 0) return;
    nodes.push(
      <p key={`p-${nodes.length}`} className="leading-6">
        {renderInlineMarkdown(paragraph.join(" "))}
      </p>,
    );
    paragraph = [];
  };

  lines.forEach((line) => {
    const trimmed = line.trim();
    if (!trimmed) {
      flushParagraph();
      return;
    }

    const headingMatch = /^(#{1,3})\s+(.+)$/.exec(trimmed);
    if (headingMatch) {
      flushParagraph();
      const level = headingMatch[1].length;
      const content = renderInlineMarkdown(headingMatch[2]);
      if (level === 1) {
        nodes.push(
          <h1 key={`h-${nodes.length}`} className="text-xl font-semibold">
            {content}
          </h1>,
        );
      } else if (level === 2) {
        nodes.push(
          <h2 key={`h-${nodes.length}`} className="text-lg font-semibold">
            {content}
          </h2>,
        );
      } else {
        nodes.push(
          <h3 key={`h-${nodes.length}`} className="text-base font-semibold">
            {content}
          </h3>,
        );
      }
      return;
    }

    const listMatch = /^[-*]\s+(.+)$/.exec(trimmed);
    if (listMatch) {
      flushParagraph();
      nodes.push(
        <div key={`li-${nodes.length}`} className="flex gap-2 leading-6">
          <span className="text-muted-foreground">-</span>
          <span>{renderInlineMarkdown(listMatch[1])}</span>
        </div>,
      );
      return;
    }

    paragraph.push(trimmed);
  });

  flushParagraph();
  return nodes;
}

function renderInlineMarkdown(text: string): React.ReactNode[] {
  const parts = text.split(/(`[^`]+`|\*\*[^*]+\*\*)/g);
  return parts
    .filter((part) => part.length > 0)
    .map((part, index) => {
      if (part.startsWith("`") && part.endsWith("`")) {
        return (
          <code
            key={index}
            className="rounded bg-muted px-1 py-0.5 font-mono text-xs"
          >
            {part.slice(1, -1)}
          </code>
        );
      }
      if (part.startsWith("**") && part.endsWith("**")) {
        return (
          <strong key={index} className="font-semibold">
            {part.slice(2, -2)}
          </strong>
        );
      }
      return <React.Fragment key={index}>{part}</React.Fragment>;
    });
}

export default MarkdownEditor;
