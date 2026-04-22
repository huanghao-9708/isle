<div align="right">
  <strong>🇨🇳 简体中文</strong> | <a href="./README_en.md">🇺🇸 English</a>
</div>

<div align="center">
  <h1>🏝️ 私屿 (Isle)</h1>
  <p><strong>你的私人音乐库，只属于你。</strong></p>
</div>

<br/>

「私屿」(Isle) 是一款以**数据绝对私有化**为核心，主打「轻量无广告、高颜值跨平台、本地播放 + 用户自有云同步」的纯净本地音乐播放器。

## 🌟 核心哲学与承诺

我们拒绝流媒体时代的隐私窃取与商业打扰。我们向你承诺：

*   **🔒 绝对的数据所有权**: 零数据上报，零追踪代码。服务端永远不触碰、不存储任何用户数据。
*   **🫧 绝对的纯净体验**: 终身无开屏广告、插屏广告，无任何商业化营销及推荐信息流。
*   **🔌 真正的离线可用**: 核心播放与音乐库管理功能 100% 支持始终离线使用，无强制联网授权要求。

## ✨ 核心特性

### 🎧 极致播放引擎
*   **全格式兼容**: 完美原生解码 FLAC, APE, WAV, MP3, M4A, OGG 等部分全部主流无损/有损格式。
*   **极速冷启动**: 得益于 Rust 的底层性能，桌面端冷启动 ≤2秒，即使扫描加载 TB 级超大曲库也能毫秒级响应。

### 🔄 私有云跨端同步
*   **WebDAV 兼容**: 允许将播放列表、进度、收藏等轻量元数据同步至你个人的 NAS 或私有云盘。
*   **端到端加密**: 采用 AES-GCM 算法，同步数据在本地即被加密，全程受你个人的密钥保护。

### 🎨 现代设计美学
*   **高标准设计语言**: 采用 Tailwind CSS v3 与 DaisyUI v4 构筑，自带惊艳的沉浸式「深色模式」和超大专辑封面墙。
*   **跨平台原生体验**: 一套核心代码即可平滑适配 Windows, macOS, Linux, Android 以及 Web 端。

### 🧘 私人专属空间
*   **听歌足迹**: 自动、非阻塞地记录每一首歌的播放痕迹，呈现专属于你的听歌心路。
*   **深度收藏**: 从歌手、专辑到曲目进行多维度收藏整理，构建你的数字音乐资产档案。
## 📸 界面预览

| 音乐库 | 沉浸式播放 | 私人空间 |
| :---: | :---: | :---: |
| ![音乐库](./resource/image/音乐库.png) | ![沉浸式播放](./resource/image/沉浸式播放页面.png) | ![私人空间](./resource/image/私人空间.png) |

## 🛠️ 技术架构

本项目采用 **Cargo Workspace** 多包架构，严格分离核心业务逻辑与各端 UI 表现层，最大化代码复用率：

| 模块分层 | 核心方案 | 职责说明 |
| :--- | :--- | :--- |
| **UI Framework** | [Dioxus 0.7](https://dioxuslabs.com/) | 跨平台 UI 核心，提供响应式状态与组件树流 |
| **Database** | SQLite + SQLx | 高性能本地检索与海量数据索引、历史记录持久化 |
| **Audio Engine** | Symphonia + Rodio | 负责全格式的硬件级音频解码与跨平台音频输出 |
| **Styling** | Tailwind CSS v3 + DaisyUI v4 | 原子化 CSS 与成熟组件库构建的一致性高颜值 UI |
| **Encryption** | AES-GCM (RustCrypto) | 保证跨设备同步（WebDAV 等）的绝对安全需求 |

## 🚀 快速开始

**1. 前置依赖**
* 安装最新版 [Rust Stable](https://www.rust-lang.org/tools/install) (1.75+)
* 安装 Dioxus 脚手架: `cargo install dioxus-cli`
* 安装 Node.js (用于首次构建 Tailwind 样式环境)

**2. 构建与运行**
```bash
git clone https://github.com/your-username/isle.git
cd isle

# 安装前端样式依赖并构建
npm install
npm run build:css

# 启动桌面端预览
cd packages/desktop
dx serve --platform desktop

# 3. 打包发布
# Windows msi 示例
dx bundle --platform desktop --release --package-types msi
```
> **提示:** 修改代码调试 UI 时，可在新窗口运行 `npm run watch:css` 实时编译 Tailwind 样式。

## 📅 演进路线

- [x] **Phase 1 (MVP Foundation)**: 基于 Dioxus 0.7，实现高性能本地音轨大文件扫描与无损播放。
- [x] **Phase 2 (Interaction)**: 落成 “私人空间” （播放历史防抖入库、全局收藏反转交互）。
- [ ] **Phase 3 (Sync & Security)**: 实装基于 WebDAV 的端到端加密、纯元数据跨端无感同步。
- [ ] **Phase 4 (Creation)**: 开放本地 LRC 歌词可视化编辑板及全自定义标签分类体系。

## ⚖️ 开源协议

本项目作为探索极客隐私边界的音乐聚合器，所有核心框架代码永久基于 [MIT License](LICENSE) 开源。

---
<div align="center">
  <b>在算法的海洋中，找寻属于你的那座岛。</b>
</div>
