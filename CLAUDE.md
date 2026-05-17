# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在本仓库中工作时提供指导。

## 项目概述

Free Proxy 是一个轻量级 macOS 代理客户端，技术栈为 **Tauri 2（Rust）+ React 19 + TypeScript + Vite**。内嵌 **mihomo（Clash.Meta）** 作为代理内核，以 Tauri sidecar 进程方式管理。

## 常用命令

```bash
pnpm install                             # 安装前端依赖
bash scripts/download-mihomo.sh v1.19.6  # 下载 mihomo 二进制（首次构建前必须执行）
pnpm tauri dev                           # 开发模式（启动 Vite + 编译 Rust + 打开窗口）
pnpm tauri build                         # 生产构建（输出 .dmg/.app）
cargo check --manifest-path src-tauri/Cargo.toml  # 仅检查 Rust 编译
pnpm build                               # 仅构建前端（tsc + vite build）
```

尚未配置测试套件。

## 架构

应用分为两个运行时层，通过两条独立通道通信：

### 双通道 IPC

1. **Tauri invoke（前端 -> Rust）** — 用于生命周期操作：启停代理、管理订阅、切换系统代理、读写配置。定义在 `src-tauri/src/cmd/`，前端通过 `src/services/tauriCommands.ts` 调用。

2. **直连 HTTP/WebSocket（前端 -> mihomo）** — 用于实时数据和节点选择。前端直接连接 mihomo 的 external-controller `127.0.0.1:9090`。定义在 `src/services/mihomoApi.ts`。这样避免高频流量数据经过 Tauri IPC。

### 关键端口

- **7890** — mihomo mixed-port（HTTP + SOCKS5 代理，按连接自动检测协议）。macOS 系统代理指向此端口。
- **9090** — mihomo external-controller（REST API + WebSocket）。`tauri.conf.json` 中的 CSP 允许 `connect-src` 访问此地址。

### 代理内核生命周期

`core/mihomo_manager.rs` 通过 `tauri-plugin-shell` 管理 mihomo sidecar。二进制文件在 `src-tauri/binaries/` 中，遵循 Tauri 的 target-triple 命名约定（`mihomo-aarch64-apple-darwin`、`mihomo-x86_64-apple-darwin`）。mihomo 以 `-d <app_data_dir>/mihomo` 启动，从该目录读取 `config.yaml`。

### 配置生成流程

`core/config_generator.rs` 使用 `serde_yaml::Value` 在代码中动态构建 mihomo 的 `config.yaml`（非字符串模板）。生成内容包括：mixed-port、DNS（fake-ip 模式，国内 DNS 为主、国外 DNS 为 fallback）、proxy-providers（每个订阅对应一个 file 类型 provider）、proxy-groups（PROXY select + auto url-test）及分流规则。

### 订阅策略

订阅由 Rust 后端下载（而非 mihomo 直接拉取）。每个订阅的节点列表保存为本地 YAML 文件 `providers/sub-<uuid>.yaml`，在配置中以 `type: file` proxy-provider 引用。这样应用可以完全控制拉取时机、错误处理和 UI 反馈。

### 系统代理（仅 macOS）

`core/system_proxy.rs` 通过 `networksetup` CLI 命令在活跃的网络服务（自动检测 Wi-Fi/Ethernet）上设置/取消 HTTP、HTTPS、SOCKS 代理。启动时 `lib.rs` 中的 `cleanup_stale_proxy()` 会检查上次会话是否遗留了指向我们端口的系统代理并自动清理。

### 数据持久化

所有持久数据存储在 `~/Library/Application Support/com.freeproxy.app/`：
- `app_config.json` — 用户设置（模式、端口、secret、自定义规则）
- `subscriptions.json` — 订阅元数据
- `mihomo/config.yaml` — 生成的 mihomo 配置
- `mihomo/providers/` — 订阅节点文件

### 前端状态管理

三个 Zustand store（`src/store/`）：`proxyStore`（代理开关、模式、节点组）、`subscriptionStore`（订阅增删改查）、`trafficStore`（WebSocket 实时上下行速率）。

### 托盘行为

系统托盘在 `tray/tray.rs` 中以代码创建（不通过 `tauri.conf.json`）。关闭窗口时隐藏到托盘而非退出。从托盘菜单退出时会先禁用系统代理并终止 mihomo。

## Rust 模块结构

- `src-tauri/src/lib.rs` — Tauri 入口：注册插件、命令、托盘、窗口事件、启动清理
- `src-tauri/src/cmd/` — Tauri 命令处理器（IPC 桥接）。每个文件对应一个领域：proxy、subscription、config、system_proxy、rules
- `src-tauri/src/core/` — 核心业务逻辑：mihomo 进程管理、config YAML 生成、订阅拉取解析、networksetup 代理控制
- `src-tauri/src/model/` — 共享数据结构（AppConfig、AppState、Subscription、ProxyGroup 等）
- `src-tauri/src/utils/` — 路径解析（`paths.rs`）和活跃网络服务检测（`network.rs`）
