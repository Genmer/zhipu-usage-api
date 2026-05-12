# 智谱AI 额度监控

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
