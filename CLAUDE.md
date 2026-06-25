```
# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.
```

# CC Config 项目开发指南

## 项目概览

CC Config 是一个基于 **Tauri 2** 的跨平台原生桌面应用程序，旨在统一管理 Claude Code AI编程CLI工具，提供可视化配置和快速切换功能。

### 核心功能
- 支持 Claude Code AI编程工具
- 提供50+内置提供商预设，可视化配置
- 系统托盘快速访问和 Claude Code 热切换
- 统一MCP管理、云同步、代理与故障转移
- 使用统计、会话管理、跨平台支持

## 技术架构

### 前端
- React 18 + TypeScript + Vite
- TailwindCSS + shadcn/ui组件库
- React Query (数据管理) + Framer Motion (动画)
- i18next (国际化：zh/en/ja)

### 后端
- Rust + Tauri 2 框架
- SQLite数据库 + rusqlite
- Axum代理服务器 + Tower/Hyper
- 系统级集成（自动启动、注册表、深度链接）

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
├── components/          # 业务组件
├── hooks/              # React钩子
├── lib/                # 工具库（API、错误处理、查询、验证）
├── locales/            # 国际化翻译
├── config/             # 内置提供商预设
├── contexts/           # 全局上下文
└── types/              # TypeScript类型

src-tauri/              # 后端代码
├── src/
│   ├── commands/       # Tauri API接口
│   ├── services/       # 业务逻辑
│   ├── database/       # SQLite数据访问
│   ├── proxy/          # 代理服务
│   ├── session_manager/# 会话管理
│   ├── deeplink/       # 深度链接
│   └── mcp/            # MCP同步
└── tests/              # Rust后端测试

tests/                  # 前端单元测试
docs/                   # 项目文档
assets/                 # 资源文件
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
2. **代理服务** 会在开发时自动启动（端口10889）
3. **配置存储** 基于 JSON 文件（`JsonStore`），无需数据库迁移
4. **热重载** 支持前端代码变更，Rust代码变更需要重启
5. **测试钩子** 用于在测试期间模拟系统功能

## 贡献指南

请参考 `CONTRIBUTING.md` 和项目代码风格。
