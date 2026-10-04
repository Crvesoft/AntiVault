# AntiVault 🛡️

<div align="center">

<img src="./public/logo.png" alt="AntiVault Logo" width="100" height="100" />

### Antigravity 多账号管理与额度监控工具
**All-in-One Multi-Account & Quota Manager for Google Antigravity**

[![Tauri v2](https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri&logoColor=white)](https://tauri.app/)
[![React 19](https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=black)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.x-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Tailwind CSS v4](https://img.shields.io/badge/Tailwind-v4-38B2AC?logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)
[![Platform](https://img.shields.io/badge/Platform-Windows-0078D6?logo=windows&logoColor=white)](https://www.microsoft.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

</div>

---

## 📖 项目简介 (Introduction)

**AntiVault** 是一款专为 **Google Antigravity** 开发者打造的现代化桌面原生管理工具。针对多账号切换繁琐、额度消耗不透明、本地凭证易冲突等痛点，AntiVault 提供了**一键无感切换账号**、**多模型额度实时追踪**、**每分钟自动静默刷新**、**拖拽卡片自定义排序**等核心功能，帮助开发者轻松掌控多账号开发环境。

---

## ✨ 核心特性 (Key Features)

### 1. ⚡ 无感一键切换账号 (Seamless Account Switch)
- **多端状态统一注入**：支持同时切换 Windows 凭据管理器（`gemini:antigravity`）与 Antigravity IDE 本地数据库（`state.vscdb`），确保 CLI 与 IDE 凭证完全同步。
- **协议级双层注入**：通过内置 Protobuf 序列化构造 `antigravityUnifiedStateSync.oauthToken`，保证新凭证写入后立即生效。
- **数据库冷备份保障**：切换前自动检测 Antigravity 运行状态，智能关闭外部进程并对 `state.vscdb` 制作带时间戳的完整安全备份。
- **可选自动重启**：切换凭据后可一键自动唤醒或重启 Antigravity IDE 客户端。

### 2. 📊 实时模型额度追踪 (Live Quota & Usage Tracking)
- **多模型额度可视**：实时展示 Claude 3.7 Sonnet、Claude 3.5 Sonnet、Gemini 2.5 Pro、Gemini 2.5 Flash 等模型的当前可用配额百分比与 Reset 重置倒计时。
- **订阅方案自动识别**：智能识别 Google Cloud Code 账号的订阅等级（`FREE` / `PRO` / `ULTRA`）及关联项目 ID。
- **动态健康色彩反馈**：采用直观的进度条色彩（>50% 翡翠绿、20%~50% 琥珀黄、<20% 警示红），一目了然。

### 3. 🔄 自动静默刷新 (Silent Auto Refresh)
- **每分钟后台静默同步**：每隔 1 分钟在后台静默轮询最新配额数据，无需手动频繁点击。
- **零弹窗无感体验**：静默轮询不触发打扰性通知，不锁定前端交互界面。
- **防竞态保护**：在账号切换或授权登录时自动挂起后台轮询，防止凭证竞争。

### 4. 🔀 账号卡片拖拽排序 (Drag-and-Drop Reordering)
- **自由拖拽布局**：支持按住卡片左上角拖拽手柄或直接拖动卡片调整排列顺序。
- **毫秒级乐观更新**：拖放瞬间前端即时重排，零操作延迟。
- **SQLite 顺序持久化**：卡片顺序自动记录在本地数据库的 `sort_order` 字段中，软件重启依然维持自定义顺序。

### 5. 🔐 硬件级凭据安全 (Secure Credential Storage)
- **Windows Credential Manager 加密存储**：核心 Refresh Token 存放于 Windows 系统密钥环（DPAPI 加密），杜绝明文文件泄露风险。
- **SQLite WAL 本地存储**：账号元信息、配额缓存、排序权重均保存在本地 SQLite 数据库中，高速、稳定、可靠。

### 6. 🔍 便捷账号管理与检索 (Fast Search & Import)
- **Google OAuth 一键授权**：内置本地 HTTP 回调与 PKCE（S256）验证机制，点击即可拉起浏览器安全授权。
- **本地凭证扫描导入**：一键扫描本地已登录的 Antigravity 凭据并直接导入。
- **手动 Token 导入**：支持粘贴 Refresh Token 并自定义备注别名。
- **分类过滤与搜索**：支持根据关键词搜索邮箱/昵称，以及按 PRO / ULTRA / FREE 分类快速过滤。

### 7. 🎨 深浅双主题与现代化 UI (Dark/Light Mode)
- 原生支持 **浅色（Light）** 与 **深色（Dark）** 主题自适应无缝切换。
- 精致的微质感 Glassmorphism 现代桌面界面设计，高对比度字体与流畅视觉动效。

---

## 🛠️ 技术架构 (Tech Stack)

| 层次 | 技术选型 | 说明 |
| :--- | :--- | :--- |
| **桌面底座** | [Tauri 2.0](https://tauri.app/) (Rust) | 超低内存占用、高运行性能、原生系统 API 桥接 |
| **前端界面** | [React 19](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) | 强类型现代化组件架构 |
| **工程构建** | [Vite 8](https://vitejs.dev/) | 毫秒级 HMR 与极速打包 |
| **样式体系** | [Tailwind CSS v4](https://tailwindcss.com/) | 现代化实用类 CSS 系统 |
| **状态管理** | [Zustand](https://github.com/pmndrs/zustand) | 轻量响应式全局状态树 |
| **安全存储** | `keyring-rs` (Windows Credential Manager) | 操作系统级凭据加密存储 |
| **数据引擎** | `rusqlite` (SQLite WAL) | 嵌入式高性能轻量数据库 |
| **系统交互** | `sysinfo` + `dirs` | 进程检测、生命周期管理与目录定位 |

---

## 🚀 开发与构建 (Development & Build)

### 前置环境需求
1. [Node.js](https://nodejs.org/) (建议 LTS 18 或更高版本)
2. [Rust 工具链](https://www.rust-lang.org/) (`rustup` / `cargo`)
3. Windows 10/11 操作系统，已安装 Microsoft C++ Build Tools

### 1. 安装项目依赖
```powershell
npm install
```

### 2. 启动本地开发预览
```powershell
npm run tauri dev
```

### 3. 构建发布安装包
```powershell
npm run tauri build
```
打包成功后，可在以下路径获取安装文件：
- **NSIS 安装包**：`src-tauri/target/release/bundle/nsis/AntiVault_0.1.0_x64-setup.exe`
- **MSI 安装包**：`src-tauri/target/release/bundle/msi/AntiVault_0.1.0_x64_en-US.msi`
- **免安装可执行文件**：`src-tauri/target/release/antivault.exe`

---

## 📂 项目结构 (Project Structure)

```text
AntiVault/
├── src/                          # 前端源码 (React 19 + TypeScript)
│   ├── assets/                   # 静态图标与资源
│   ├── components/               # UI 组件库
│   │   ├── AccountCard.tsx       # 账号卡片组件（支持拖拽排序）
│   │   ├── AddAccountModal.tsx   # 添加账号弹窗（OAuth / 本地扫描）
│   │   ├── AntiVaultLogo.tsx     # 品牌矢量 Logo
│   │   ├── DeleteConfirmModal.tsx# 移除账号确认弹窗
│   │   ├── QuotaDetailModal.tsx  # 模型详细配额弹窗
│   │   ├── StatusBar.tsx         # 底部状态栏
│   │   ├── SwitchConfirmModal.tsx# 账号切换确认弹窗
│   │   ├── TitleBar.tsx          # 自定义沉浸式标题栏（主题切换/窗口控制）
│   │   └── Toast.tsx             # 消息提示组件
│   ├── services/                 # 前端 API 桥接层
│   ├── stores/                   # Zustand 全局状态管理
│   ├── utils/                    # 配额格式化与辅助工具
│   ├── App.tsx                   # 根应用组件
│   └── index.css                 # Tailwind 样式入口
├── src-tauri/                    # 后端源码 (Rust + Tauri 2.0)
│   ├── src/
│   │   ├── commands/             # Tauri 前后端通信接口命令
│   │   │   ├── account.rs        # 账号增删查改与排序持久化
│   │   │   ├── oauth.rs          # Google OAuth 流程
│   │   │   ├── quota.rs          # 配额查询与全量刷新
│   │   │   └── system.rs         # Antigravity 状态检测与数据库切换注入
│   │   ├── storage/              # SQLite 数据库模型与迁移
│   │   ├── utils/                # 凭据管理、HTTP 客户端、Protobuf 构造等
│   │   ├── lib.rs                # Tauri 运行器与命令注册
│   │   └── main.rs               # 主程序入口
│   ├── Cargo.toml                # Rust 依赖清单
│   └── tauri.conf.json           # Tauri 应用配置
└── package.json                  # 前端依赖配置
```

---

## 🔒 隐私与安全声明 (Privacy & Security)

- **完全本地运行**：AntiVault 绝不包含任何第三方追踪或远程分析脚本，不会收集或上传用户的任何凭证与使用数据。
- **凭据隔离加密**：敏感的 Google Refresh Token 仅加密存储于本地 Windows Credential Manager，任何明文信息均不落盘。
- **官方接口直连**：配额查询均直接调用 Google 官方 Cloud Code API 端点，中间无任何中转代理服务器。

---

## 📄 开源许可证 (License)

本项目采用 [MIT License](LICENSE) 许可证开源。
