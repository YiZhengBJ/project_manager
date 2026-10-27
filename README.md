# Project Manager · v1.0

本地项目管理桌面应用，采用 Rust + Tauri 2 + 原生 HTML/CSS/JavaScript。前端保留已确认的界面，不依赖外部网页或在线服务。

## 功能

- 左侧 Projects 支持新增、切换和删除项目，删除前需确认；支持空工作空间。
- 项目事项脑图：任意节点新增下级，滚轮缩放、左键拖动、居中复位。
- 滚轮事件按动画帧合并并平滑缩放，反向滚动即时响应；统一读取卡片尺寸后排版，减少强制重排，并固定鼠标所在卡片的位置。
- 卡片内编辑名称和多位负责人；可勾选新增或移除，至少保留一位；使用上下箭头调整顺序，首位为首要责任人。长标题完整换行。
- 缩放低于 55% 时脑图只保留事项名称，放大至 65% 后恢复详情；卡片宽度、字体和留白随缩放动态调整。
- 标题使用自然换行与自适应高度，展开按钮独立放在卡片外侧，避免文字裁切或被控件遮挡；居中会按动态布局重新计算。
- 画布默认宽高比 2:1，随页面宽度变化；可拖动底边或用上下方向键调整高度，记住比例。
- 项目事项结构采用居中双向扩展树；按子树高度预留空间，支持长标题、目标与风险。
- 项目居中的双向思维导图：一级分支按实际高度均衡分配到左右，下级沿所属方向向外展开，分支配色与柔和曲线保持一致；缩小时按标题宽度显示简洁节点。
- 拖动画布空白处移动视野；拖动卡片非编辑区域到目标卡片上或附近，可将整棵子树移为目标的下级。目标会高亮提示，根节点与自身下级不可作为非法移动来源或目标。
- 拖动期间禁止文字选择；按 Esc 或点击鼠标右键取消拖动，恢复拖动前的画布位置，卡片从属关系保持原样。
- 事项详情支持删除事项：先展示本节点及全部下级的树形结构与数量，经用户确认后删除整棵分支及记录；删除根节点会同时删除项目。
- 负责人管理支持新增、移除与事项转交，负责人名单独立保存。
- 点击绿点/灰点切换进行中/未进行；未进行递归应用到所有下级并隐藏完成度。
- 事项详情按日期编辑 TODO 与进展，默认列出今天和昨天；昨日未完成 TODO 可一键顺延到今日，追加到已有计划且继续可编辑，重复点击不重复追加。
- 事项记录默认隐藏历史区间内没有内容的日期，可切换“显示空白日期”补填遗漏的 TODO 和进展；补填后该日期会作为已有记录持续显示。
- 应用保持打开时会在上海时区跨日后自动切换到新日期；电脑休眠后重新唤醒或应用重新获得焦点也会立即校准日期。手动查看历史日期时不会被自动跳回今天。
- 顺延标记随记录保存；日报昨日进展排除已顺延到今日的记录，每条日报记录缩进展示完整事项路径。
- 每日 TODO 仅按首要责任人汇总，多人协作任务不会重复出现在其他负责人的 TODO 或日报中；纯文本日报分为昨日 TODO 进展、今日 TODO 两段。
- 每个事项支持可选目标和风险，有内容时展示在脑图卡片中。
- 任务表格按叶子任务展示；上级路径用 `/` 连接，同路径单元格合并。
- 历史 TODO 与进展按日期倒序，默认展示 5 条，more 每次追加 5 条；可直接编辑并自动保存。
- 表格支持新增今日 TODO，与事项详情和每日 TODO 共用记录。
- 表格设置可隐藏未进行、已完成任务，自定义显示列；默认隐藏完成度、目标、风险。
- Excel 导出按当前表格筛选和显示列生成 `.xlsx`，导出全部历史记录，保留合并单元格、日期倒序和换行样式。
- 桌面 Excel 导出存放在系统下载目录下的 `Project Manager/`，重名自动追加编号；浏览器模式直接下载。
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

新版本数据文件包含 `projects` 和 `activeProjectId`，每个项目独立保存 `items` 和 `members`；事项 `owners` 保存多位负责人，`owner` 保留首位负责人以兼容旧数据；兼容读取旧版单项目和事项数组。保存使用临时文件写入、同步及替换；上一份文件备份为 `project.json.bak`。关闭窗口前等待保存队列完成。加载失败时停止编辑，避免以示例数据覆盖原文件。

首次打开应用创建示例项目。浏览器原型的 localStorage 属于不同应用环境，不会自动写入桌面版。浏览器预览模式仍使用 localStorage。日报日期按 Asia/Shanghai 时区计算。

## 代码结构

```text
frontend/                 HTML、CSS、交互脚本和桌面数据适配
frontend/excel.js          离线 OOXML/ZIP 导出器
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
