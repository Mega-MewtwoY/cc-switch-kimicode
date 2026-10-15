# CC Switch（Kimi Code 定制版）

从现有 3.20.4 定制源码升级，已合并上游 4.0.9 及 2026-10-12 核对的 main 分支更新（28dc5198）。保留 Kimi Code CLI、Kimi 桌面端、豆包、自定义用量统计和 Codex 多账号额度等现有功能。上游项目为 https://github.com/farion1231/cc-switch ，定制项目为 https://github.com/Mega-MewtwoY/cc-switch-kimicode 。

本版本不提供 ZCode 应用入口、原生配置、MCP/技能同步或会话扫描，也不在供应商预设中提供 GLM 模型。历史用量和模型定价继续由独立数据表保存；历史数据中的应用标识及数据库迁移兼容逻辑保留。

## 打开应用

供应商卡片右侧直接显示启动按钮，「编辑」移入右侧「⋯」菜单。启动按钮使用项目现有的终端或桌面窗口图标，悬停可查看名称。

| 入口 | 启动目标 |
| --- | --- |
| Claude Code | Claude Code CLI |
| Codex | Codex 桌面端 |
| Kimi Code | Kimi Code.app；未安装时沿用原来的 CLI 回退 |
| Kimi Work&Chat | Kimi.app（聊天与工作客户端） |
| 豆包 | 豆包桌面端 |

Claude Code、Codex 和 Kimi Code 沿用原来的工作目录选择流程；Kimi Work&Chat 和豆包直接打开应用。

## 构建

使用 `rust-toolchain.toml` 指定的 Rust 工具链、支持 Vite 7 的 Node.js 和 pnpm。macOS 需要 Xcode Command Line Tools。

```sh
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test:unit
pnpm tauri build --bundles app --config '{"bundle":{"createUpdaterArtifacts":false}}'
```

macOS 应用生成于 `src-tauri/target/release/bundle/macos/CC Switch.app`。开发时使用 `pnpm dev`。

## 数据与目录

- 应用配置及用量数据库：`~/.cc-switch/`
- Kimi Code 配置：`~/.kimi-code/`
- `src/`：前端源码
- `src-tauri/`：Rust 后端、平台配置和测试
- `tests/`：前端测试

仓库仅保存源码、测试、资源和文档，不包含本地会话、备份、依赖安装目录或编译产物。对于原本只预填 GLM 模型的通用供应商，添加时需自行填写其他受支持的模型。

## Codex 多账号登录与额度

多个官方订阅卡片按用户和 workspace 区分登录。切换和额度查询共用本机的 `auth.json` 与 `~/.cc-switch/codex-login-stash.json`；配置卡内的旧登录快照只用于识别账号和首次迁移，不能独立刷新。非当前账号刷新后将新令牌原子写回本机暂存，并与供应商切换串行，切回时恢复同一份最新登录。

需要在第三方配置下继续保留官方登录时，开启「设置 → 应用配置 → Codex → 非接管切换时保留官方登录」。各官方卡的订阅用量可独立自动查询，建议设为每 1 分钟刷新，也可以点击额度立即查询。当前 live 登录的令牌刷新由 Codex 负责，避免两个进程同时轮换同一令牌。
