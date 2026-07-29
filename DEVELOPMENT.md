# Tarkov Item Manager

用于记录藏身处当前设施等级，并汇总升级至满级所需剩余材料的 Web 应用。项目采用 Vue 3 + Vuetify 前端、Rust + Axum API 和 SQLite 持久化，按 AGPL-3.0-or-later 发布。

## 功能

- 本地账户注册、登录和安全会话 Cookie。
- 记录每个设施当前已拥有等级，并展示前置条件状态。
- 自动汇总从当前等级升级至各设施满级所需的剩余材料。
- 记录商人和技能等级，并据此显示每级升级的可用或受阻状态。
- 区分带勾与非带勾材料，按剩余升级需求汇总展示。
- 应用启动时检查数据集版本、关联 ID、重复定义和设施依赖循环。

当前 `dataset/` 包含完整的 PVE 藏身处数据快照；PVP 数据、后续游戏版本或来源修订可能与其不同。

## 开发环境

需要 Node.js 22+（含 Corepack）、pnpm 和 Rust stable。所有源码和 JSON 文件均以 UTF-8 处理；读取外部 GBK 数据时应先显式转码。

```bash
cd frontend
pnpm install
pnpm build
pnpm dev
```

另开一个终端启动 API：

```bash
cd backend
cargo run
```

开发时前端地址为 `http://localhost:5173`，Vite 会将 `/api` 转发到 `http://localhost:3000`。前端变更后需重新运行 `pnpm build`，供 Rust 将生产资源嵌入二进制。

## 环境变量

复制 `.env.example` 为 `.env` 后按需修改。服务端从当前工作目录读取 `.env`，本地开发时建议在 `backend/.env` 放置下列配置：

```dotenv
DATABASE_URL=sqlite:data/tarkov-item-manager.db?mode=rwc
APP_ORIGIN=http://localhost:5173
LISTEN_ADDR=0.0.0.0:3000
SESSION_SECRET=replace-with-a-long-random-secret
SECURE_COOKIES=false
```

- `DATABASE_URL`：第一版完整支持 SQLite，例如 `sqlite:data/tarkov-item-manager.db`。PostgreSQL/MySQL 连接字符串会被明确拒绝，等待其迁移和集成测试实现。
- `DATASET_DIR`：可选的外部 PVE 快照目录；未设置时使用编译进程序的数据集。显式设置后若目录无效或数据校验失败，应用会拒绝启动。
- `APP_ORIGIN`：开发环境的前端来源，用于 CORS。
- `SESSION_SECRET`：至少 16 个字符，用于哈希会话令牌。生产环境必须设置为随机高强度值。
- `SECURE_COOKIES`：HTTPS 部署时设为 `true`。
- `DESKTOP_APP`：默认 `true`，桌面应用模式下服务成功绑定端口后会打开本机默认浏览器；开发脚本和 Docker 环境显式设为 `false`。

## 数据集

`dataset/` 当前采用扁平数值 ID 主表：`items.json`、`facilities.json` 和 `merchants.json` 均为 `{ "ID": number, "name": string }` 数组；`skills.json` 为 `{ "ID": number, "name": string, "maxLevel": number }` 数组，保存完整技能目录。根 `hideout.json` 是 PVE 快照 manifest，包含 `schemaVersion`、模式、来源元数据和按数值设施 ID 排序的 `upgradeFiles`。每个 `dataset/hideout/<facilityID>.json` 是一个升级数组，保存该设施的全部升级记录。每条升级记录使用 `facilityID`、`level`、材料 `requirements`（`itemID`、`quantity`、`foundInRaid`）、设施/商人/技能/任务/版本包前置条件，以及 `constructionTimeSeconds`。

PVE 材料、数量、带勾、建造时间和页面提供的设施/商人/技能条件来自 `eftarkov.com` 的设施详情页；Fandom MediaWiki API 用于来源补充与交叉核对。PVE 页面未提供结构化任务和版本包条件，因此对应数组当前为空。后端启动时加载并验证编译进程序的数值 PVE 数据契约；显式设置 `DATASET_DIR` 可严格覆盖为外部快照。用户的设施、商人和技能等级保存在数据库中；材料清单按未完成升级自动计算至各设施满级，不保存材料拥有或完成状态。

完整数据替换时，应保持以下约束：

- ID 为连续非负整数；所有 `facilityID`、`itemID` 和 `merchantID` 必须解析到对应主表，所有技能前置名称必须存在于 `skills.json`。
- `skills.json` 的名称不可重复，`maxLevel` 必须为正整数，技能前置等级不可超过该技能上限。
- 材料数量和设施等级为正整数，建造时间为非负整数秒数。
- `foundInRaid` 必须是布尔值，且只属于单条升级材料需求。
- 每个 `{ facilityID, level }` 只定义一次。
- 设施前置条件必须指向存在的升级等级，且不可形成循环。
- 来源名称归并必须记录在 `hideout.json` 的 `sources.notes` 中。

## Docker

Docker/Compose 可用时，先提供会话密钥：

```bash
export SESSION_SECRET='replace-with-a-long-random-secret'
docker compose up --build
```

访问 `http://localhost:3000`。SQLite 数据会保存到名为 `tarkov-data` 的 Docker 卷；容器重建不会清除该卷。

## 本地发布

先使用 `./tag.ps1` 创建并推送版本标签，然后运行：

```powershell
.\pubdev.ps1
```

`pubdev.ps1` 会显示最近可达的 Git tag。输入精确的小写 `y` 后，脚本生成只包含 `tarkov-item-manager.exe` 的 `TarkovItemManager-<tag>.zip`；正式版本包同名时不会覆盖。输入其他内容，或没有可达 tag 时，脚本生成并覆盖 `TarkovItemManager-dev.zip`。前端资源和 PVE 数据集已嵌入可执行文件；直接启动 exe 会在本机浏览器打开应用。归档不包含 `pubdev/.env`、`pubdev/data/` 中的会话密钥、账户或用户进度。

## 验证

```bash
cd backend
cargo fmt --check
cargo test

cd ../frontend
pnpm build
```

容器镜像构建命令：

```bash
docker build -t tarkov-item-manager .
```
