# 智谱AI 额度监控

官方查询API 输入API-KEY查询

## 更新公告 / Changelog

### 2026-05-28 v3.2.4

- 显示套餐类型标签（LITE绿/PRO蓝/MAX红）/ Show plan badge (LITE green / PRO blue / MAX red)

### 2026-05-27 v3.2.3

- 设置持久化：刷新间隔、卡片切换间隔重启后保留 / Settings persistence: refresh & card switch intervals survive restarts
- 版本检查改用 Rust 后端请求，解决打包后 CORS 问题 / Version check via Rust backend, fixed CORS in production build

### 2026-05-27 v3.2.2

- GitHub 图标 hover 弹出菜单，支持跳转 GitHub/Gitee 仓库 / GitHub icon hover menu with GitHub/Gitee links
- 显示当前版本与 GitHub 最新 Release 版本对比 / Shows current version vs latest GitHub Release
- 圆形进度条位置微调 / Adjusted circular progress indicator position

### 2026-05-27 v3.2.1

- 新增卡片右下角圆形进度条，显示另一张卡片的额度百分比 / Added circular progress indicator showing the other card's quota percentage
- 添加系统托盘支持，隐藏 Dock/任务栏图标 / Added system tray support, hidden from Dock/taskbar
- 卡片切换支持禁用（设置中勾选"不自动切换卡片"）/ Added option to disable auto card switching
- reqwest 切换到 rustls-tls，移除 openssl 依赖 / Switched to rustls-tls, removed openssl dependency
- 登录改为异步调用，避免界面卡顿 / Login changed to async to prevent UI freezing
- 版本号自动同步脚本 / Added version sync script

### 2026-05-07 v3.1.0

- 修复每周额度显示 0% 的问题 / Fixed weekly quota showing 0%
- 修复登录时 tokio panic 的问题 / Fixed tokio panic on login

![输入图片说明](image.png)
![输入图片说明](image2.png)

macOS 桌面悬浮窗，实时显示智谱AI Coding Plan的额度使用情况。

## 功能

- 📊 **额度监控** — 实时显示每5小时/每周额度使用百分比
- 🔄 **自动刷新** — 可配置定时自动刷新数据
- 📌 **窗口置顶** — 悬浮窗始终在最前方，可切换置顶状态
- 🔐 **账号管理** — 记住登录账号，一键切换登录
- ⚙️ **可配置** — 刷新间隔、卡片切换间隔自定义

## 下载安装

| 平台 | 下载 |
|---|---|
| macOS (Apple Silicon) | [GitHub Releases](https://github.com/Genmer/zhipu-usage-api/releases) 下载 `.dmg` |
| macOS (Intel) | [GitHub Releases](https://github.com/Genmer/zhipu-usage-api/releases) 下载 `.dmg` |
| Windows | [GitHub Releases](https://github.com/Genmer/zhipu-usage-api/releases) 下载 `.exe` 或 `.msi` |

> Windows 版本（.exe / .msi）请在 [GitHub Releases](https://github.com/Genmer/zhipu-usage-api/releases) 页面下载。

## 截图

### 登录页
- 已记住账号列表，点击直接登录
- 支持微信扫码或账号密码登录

### 使用页
- 大字显示当前额度百分比
- 进度条可视化（蓝色/黄色/红色）
- 显示重置时间倒计时
- 左右切换查看不同周期额度

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | React 18 + Tailwind CSS |
| 桌面框架 | Tauri v2 |
| 后端 | Rust |
| 图标 | Lucide React |

## 开发

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 打包构建
npm run tauri build
```

## 系统要求

- macOS / Windows
- Node.js 18+
- Rust 1.77+

## License

MIT
