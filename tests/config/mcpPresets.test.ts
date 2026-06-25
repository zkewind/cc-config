import { describe, expect, it } from "vitest";
import { getMcpPresetWithDescription, mcpPresets } from "@/config/mcpPresets";

describe("mcpPresets", () => {
  it("contains the built-in common MCP presets required by iter-3", () => {
    expect(mcpPresets.map((preset) => preset.id)).toEqual([
      "fetch",
      "time",
      "memory",
      "sequential-thinking",
      "context7",
    ]);
  });

  it("hydrates an i18n description without changing preset metadata", () => {
    const preset = mcpPresets.find((item) => item.id === "context7");
    expect(preset).toBeDefined();

    const hydrated = getMcpPresetWithDescription(preset!, (key) => `t:${key}`);

    expect(hydrated).toMatchObject({
      id: "context7",
      name: "@upstash/context7-mcp",
      description: "t:mcp.presets.context7.description",
      homepage: "https://context7.com",
    });
    expect(hydrated.server.type).toBe("stdio");
  });
});
