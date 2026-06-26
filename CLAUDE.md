```
# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.
```

# CC Config 项目开发指南

## 项目概览

CC Config 是一个基于 **Tauri 2** 的跨平台原生桌面应用程序，旨在统一管理 Claude Code AI编程CLI工具，提供可视化配置和快速切换功能。

### 核心功能
- Claude Code 供应商可视化配置与一键热切换
- 系统托盘快速访问（动态菜单）、静默启动与轻量模式
- 统一 MCP 管理（校验、向导）、提示词与技能管理（含技能仓库）
- WebDAV 云同步（手动 / 自动）、会话管理（读取 Claude 及第三方会话）
- 使用统计、余额查询、订阅额度、端点测速、流式校验
- 深度链接导入（`ccconfig://`）、自有 GitHub 更新源自更新
- 跨平台支持（Windows / macOS / Linux）

## 技术架构

### 前端
- React 18 + TypeScript + Vite
- TailwindCSS + Radix UI（shadcn 风格组件）
- TanStack Query（数据）+ Framer Motion（动画）
- CodeMirror（JSON/Markdown 编辑器）、react-hook-form + zod、recharts、dnd-kit
- i18next（国际化：zh/en）

### 后端
- Rust + Tauri 2 框架
- JSON 文件存储（`json_store`，位于 `~/.cc-config/`）
- reqwest 直连 HTTP 客户端（独立代理服务器已移除）
- rusqlite 仅用于读取第三方会话文件（OpenCode / Hermes）
- rquickjs 用于使用统计脚本；WebDAV 同步、自动启动、深度链接、单实例等系统集成

## 常用开发命令

### 完整开发流程
```bash
# 安装依赖
pnpm install

# 启动完整开发环境
pnpm dev

# 仅启动前端开发服务器
pnpm dev:renderer

# 构建生产版本
pnpm build
```

### 代码质量
```bash
# TypeScript类型检查
pnpm typecheck

# 格式化代码
pnpm format

# 检查格式
pnpm format:check
```

### 测试
```bash
# 运行前端单元测试
pnpm test:unit

# 监视模式测试
pnpm test:unit:watch
```

### Rust后端
```bash
cd src-tauri

# 格式化Rust代码
cargo fmt

# 代码检查
cargo clippy

# 运行所有测试
cargo test

# 运行特定测试
cargo test test_function_name
```

## 项目结构

### 主要目录
```
src/                      # 前端代码
├── components/          # 业务组件（providers/mcp/prompts/skills/sessions/settings/deeplink/env/ui 等）
├── hooks/               # React 钩子
├── lib/                 # 工具库（API、查询、验证等）
├── i18n/                # 国际化（locales/zh.json、locales/en.json）
├── config/              # 前端配置
├── contexts/            # 全局上下文
├── icons/               # 图标资源
├── types/               # TypeScript 类型
└── utils/               # 通用工具

src-tauri/               # 后端代码
├── src/
│   ├── commands/        # Tauri 命令（config/mcp/prompt/skill/session_manager/webdav_sync/balance/subscription/env/deeplink/import_export/model_fetch/stream_check 等）
│   ├── services/        # 业务逻辑（provider/mcp/prompt/skill/webdav/balance/subscription/speedtest/model_fetch/env_manager 等）
│   ├── json_store/      # JSON 文件存储（providers/mcp/skills/prompts/settings/backup/migration/providers_seed）
│   ├── providers/       # 供应商适配（adapter/auth/models/sse/streaming/transform/proxy_error）
│   ├── deeplink/        # ccconfig:// 深度链接解析
│   ├── mcp/             # MCP 同步与校验
│   ├── session_manager/ # 会话读取（Claude / 第三方）
│   ├── tray.rs          # 系统托盘
│   ├── handler_registry.rs # Tauri 命令注册
│   └── usage_script.rs  # 使用统计脚本引擎
└── tests/               # Rust 后端测试

tests/                   # 前端单元测试
docs/                    # 项目文档
assets/                  # 资源文件
```

## 运行时数据

应用运行时数据以 JSON 文件形式存储在用户主目录的 `.cc-config` 文件夹中（基于 `JsonStore`，非 SQLite）：
- **供应商数据**: `~/.cc-config/providers.json` - 各应用的供应商列表与当前选择
- **应用配置**: `~/.cc-config/config.json` - 通用配置片段、seed 标志、项目关联等
- **用户设置**: `~/.cc-config/settings.json` - UI偏好
- **MCP 配置**: `~/.cc-config/mcp.json`
- **技能配置**: `~/.cc-config/skills.json`
- **提示词**: `~/.cc-config/prompts.json`
- **自动备份**: `~/.cc-config/backups/` - 配置备份
- **日志**: `~/.cc-config/logs/`

## 配置文件

### 项目配置
- `package.json` - 前端依赖和脚本
- `tsconfig.json` - TypeScript配置
- `vite.config.ts` - Vite构建配置
- `tailwind.config.js` - TailwindCSS配置

### Tauri配置
- `src-tauri/Cargo.toml` - Rust依赖
- `src-tauri/tauri.conf.json` - Tauri应用配置
- `src-tauri/wix/` - Windows安装包模板

## 开发注意事项

1. **Tauri开发环境** 需要配置好 Rust 和 Node.js
2. **HTTP 客户端** 为 reqwest 直连模式（独立代理服务器已移除），无需额外启动代理进程
3. **配置存储** 基于 JSON 文件（`JsonStore`），无需数据库迁移
4. **热重载** 支持前端代码变更，Rust代码变更需要重启
5. **测试钩子** 用于在测试期间模拟系统功能

## 贡献指南

请参考 `CONTRIBUTING.md` 和项目代码风格。
