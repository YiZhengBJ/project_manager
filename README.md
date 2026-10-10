# Project Manager · v1.0

本地项目管理桌面应用，采用 Rust + Tauri 2 + 原生 HTML/CSS/JavaScript。前端保留已确认的界面，不依赖外部网页或在线服务。

## 功能

- 左侧 Projects 支持新增、切换和删除项目，删除前需确认；支持空工作空间。
- 项目事项脑图：任意节点新增下级，滚轮缩放、左键拖动、居中复位。
- 卡片内编辑名称和负责人；长标题完整换行。
- 负责人管理支持新增、移除与事项转交，负责人名单独立保存。
- 点击绿点/灰点切换进行中/未进行；未进行递归应用到所有下级并隐藏完成度。
- 事项详情按日期编辑 TODO 与进展，默认列出今天和昨天。
- 每日 TODO 按负责人汇总；纯文本日报分为昨日 TODO 进展、今日 TODO 两段。
- 每个事项支持可选目标和风险，有内容时展示在脑图卡片中。
- 任务表格按叶子任务展示；上级路径用 `/` 连接，同路径单元格合并。
- 历史 TODO 与进展按日期倒序，默认展示 5 条，more 每次追加 5 条；可直接编辑并自动保存。
- 表格支持新增今日 TODO，与事项详情和每日 TODO 共用记录。
- 表格设置可隐藏未进行、已完成任务，自定义显示列；默认隐藏完成度、目标、风险。
- Rust 保存本地项目文件，复制日报使用系统剪贴板。

## 开发环境

需要 Rust 1.90+、Node.js 20+、Python 3.9+。macOS 需安装 Xcode Command Line Tools：

```sh
xcode-select --install
```

安装前端构建工具（项目自带 pnpm 锁文件）：

```sh
pnpm install --frozen-lockfile
pnpm dev
```

也可使用 npm：

```sh
npm install
npm run dev
```

前端只有标准浏览器 API，不需要框架编译。Python 开发服务器绑定 `127.0.0.1:1420`；构建时复制资源到 `dist/`，Tauri 将其嵌入应用。

## 打包

```sh
pnpm build
# 或 npm run build
```

macOS 输出目录：

- `src-tauri/target/release/bundle/macos/Project Manager.app`
- `src-tauri/target/release/bundle/dmg/`：DMG 安装包。

打包先由 Tauri 生成 `.app`，再使用 `hdiutil` 生成含 Applications 链接的 DMG，不依赖 Finder 自动化授权。只生成应用可运行 `pnpm build:app`。

直接运行构建后的 `.app`，不再需要 Node、Python、开发服务器或网络。当前配置面向 macOS；其他系统打包需将 `bundle.targets` 改为对应系统支持的格式。

## 检查

```sh
python3 scripts/frontend.py build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
```

## 数据

桌面版项目文件存放于 Tauri 的应用数据目录。在 macOS 上为：

```text
~/Library/Application Support/local.project-manager.desktop/project.json
```

新版本数据文件包含 `projects` 和 `activeProjectId`，每个项目独立保存 `items` 和 `members`；兼容读取旧版单项目和事项数组。保存使用临时文件写入、同步及替换；上一份文件备份为 `project.json.bak`。关闭窗口前等待保存队列完成。加载失败时停止编辑，避免以示例数据覆盖原文件。

首次打开应用创建示例项目。浏览器原型的 localStorage 属于不同应用环境，不会自动写入桌面版。浏览器预览模式仍使用 localStorage。日报日期按 Asia/Shanghai 时区计算。

## 代码结构

```text
frontend/                 HTML、CSS、交互脚本和桌面数据适配
scripts/frontend.py       前端构建 / 开发服务器
scripts/package_macos.py  无需 Finder 自动化的 DMG 打包
src-tauri/src/            Rust 本地存储、校验、剪贴板和窗口入口
src-tauri/capabilities/    Tauri 窗口权限配置
src-tauri/icons/          应用图标源资源
src-tauri/tauri.conf.json  应用窗口和打包配置
```

`.gitignore` 已排除 Rust `target/`、前端 `dist/`、Tauri 生成目录、依赖、缓存和打包目录；Cargo.lock 和 pnpm-lock.yaml 应纳入版本管理。

Tauri 配置与剪贴板实现参考：[Tauri 配置文档](https://v2.tauri.app/reference/config/)、[剪贴板插件](https://v2.tauri.app/plugin/clipboard/)。

本机的 `.cargo/config.toml` 使用 rsproxy 镜像加速首次依赖下载，已加入忽略规则；删除该本地文件即可恢复默认 crates.io，不影响锁文件和应用代码。
