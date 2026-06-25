import { describe, expect, it } from "vitest";
import { computePathLabels, abbreviatePath } from "@/lib/api/projects";

describe("computePathLabels", () => {
  it("末级唯一时只显示末级目录名", () => {
    const labels = computePathLabels([
      "C:\\proj\\notes-app",
      "D:\\work\\misc\\tools",
    ]);
    expect(labels.get("C:\\proj\\notes-app")).toBe("notes-app");
    expect(labels.get("D:\\work\\misc\\tools")).toBe("tools");
  });

  it("单条路径只显示末级目录名", () => {
    const labels = computePathLabels(["C:\\proj\\only"]);
    expect(labels.get("C:\\proj\\only")).toBe("only");
  });

  it("末级重复、倒数第二级即区分时显示 区分级\\末级（无省略号）", () => {
    const labels = computePathLabels([
      "D:\\work\\demo\\app",
      "E:\\repo\\lab\\app",
    ]);
    expect(labels.get("D:\\work\\demo\\app")).toBe("demo\\app");
    expect(labels.get("E:\\repo\\lab\\app")).toBe("lab\\app");
  });

  it("末级重复且倒数第二级也相同时，向上找到区分级显示 区分级\\...\\末级", () => {
    const labels = computePathLabels([
      "C:\\proj\\teamA\\frontend\\service",
      "C:\\proj\\teamB\\frontend\\service",
    ]);
    expect(labels.get("C:\\proj\\teamA\\frontend\\service")).toBe(
      "teamA\\...\\service",
    );
    expect(labels.get("C:\\proj\\teamB\\frontend\\service")).toBe(
      "teamB\\...\\service",
    );
  });

  it("支持更深的区分级（倒数第四级）", () => {
    const labels = computePathLabels([
      "C:\\a\\X\\common\\shared\\module",
      "C:\\a\\Y\\common\\shared\\module",
    ]);
    expect(labels.get("C:\\a\\X\\common\\shared\\module")).toBe("X\\...\\module");
    expect(labels.get("C:\\a\\Y\\common\\shared\\module")).toBe("Y\\...\\module");
  });

  it("路径短于区分级所需层级时原样显示完整路径", () => {
    // /app 深度 1，取不到区分级 → 原样；/x/app 倒数第二级 x 区分
    const labels = computePathLabels(["/app", "/x/app"]);
    expect(labels.get("/app")).toBe("/app");
    expect(labels.get("/x/app")).toBe("x/app");
  });

  it("Unix 风格路径使用 / 分隔符", () => {
    const labels = computePathLabels([
      "/home/user/teamA/frontend/service",
      "/home/user/teamB/frontend/service",
    ]);
    expect(labels.get("/home/user/teamA/frontend/service")).toBe(
      "teamA/.../service",
    );
    expect(labels.get("/home/user/teamB/frontend/service")).toBe(
      "teamB/.../service",
    );
  });
});

describe("abbreviatePath（保持不变，回归保护）", () => {
  it("长路径仍按 首段+...+末级 缩写", () => {
    const long =
      "C:\\" +
      "a\\b\\c\\d\\e\\f\\g\\h\\i\\j\\k\\l\\m\\n\\o\\p\\q\\r\\s\\target";
    const result = abbreviatePath(long, 10);
    expect(result).toMatch(/^C:\\/);
    expect(result).toContain("...");
    expect(result).toMatch(/target$/);
  });

  it("短路径原样返回", () => {
    expect(abbreviatePath("C:\\short", 36)).toBe("C:\\short");
  });
});
