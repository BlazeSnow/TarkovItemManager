# Tarkov Item Manager

用于规划藏身处升级目标并汇总所需材料的 Web 应用。项目采用 Vue 3 + Vuetify 前端、Rust + Axum API 和 SQLite 持久化，按 AGPL-3.0-or-later 发布。

## 功能

- 本地账户注册、登录和安全会话 Cookie。
- 按设施设定目标等级，展示前置条件状态。
- 根据目标等级汇总升级材料。
- 勾选已拥有材料，并保存到当前用户账户。
- 应用启动时检查数据集版本、关联 ID、重复定义和设施依赖循环。

当前 `dataset/` 中仅包含用于验证完整流程的示例数据，不代表游戏中的完整或最新物品与藏身处需求。

## 开发环境

需要 Node.js 22+、npm 和 Rust stable。所有源码和 JSON 文件均以 UTF-8 处理；读取外部 GBK 数据时应先显式转码。

```bash
cd frontend
npm install
npm run dev
```

另开一个终端启动 API：

```bash
cd backend
cargo run
```

开发时前端地址为 `http://localhost:5173`，Vite 会将 `/api` 转发到 `http://localhost:3000`。

## 环境变量

复制 `.env.example` 为 `.env` 后按需修改。服务端从当前工作目录读取 `.env`，本地开发时建议在 `backend/.env` 放置下列配置：

```dotenv
DATABASE_URL=sqlite:data/tarkov-item-manager.db?mode=rwc
DATASET_DIR=../dataset
APP_ORIGIN=http://localhost:5173
LISTEN_ADDR=0.0.0.0:3000
SESSION_SECRET=replace-with-a-long-random-secret
SECURE_COOKIES=false
```

- `DATABASE_URL`：第一版完整支持 SQLite，例如 `sqlite:data/tarkov-item-manager.db`。PostgreSQL/MySQL 连接字符串会被明确拒绝，等待其迁移和集成测试实现。
- `DATASET_DIR`：五个 JSON 数据集的目录。
- `APP_ORIGIN`：开发环境的前端来源，用于 CORS。
- `SESSION_SECRET`：至少 16 个字符，用于哈希会话令牌。生产环境必须设置为随机高强度值。
- `SECURE_COOKIES`：HTTPS 部署时设为 `true`。
- `SOURCE_URL`：前端的 AGPL 源码链接，前端构建时请使用 `VITE_SOURCE_URL` 设置。

## 数据集

`dataset/schema/` 提供版本 1 JSON Schema。静态数据只描述不可变规则：物品、翻译、设施、升级材料和带等级的前置条件。用户的设施等级选择和材料勾选状态保存在数据库中。

完整数据替换时，应保持以下约束：

- ID 为非空稳定字符串，中文/英文文件使用相同 ID 集合。
- 材料数量与升级等级均为正整数。
- 每个 `{ facilityId, level }` 只定义一次。
- 前置条件必须指向存在的设施等级，且不可形成循环。

## Docker

Docker/Compose 可用时，先提供会话密钥：

```bash
export SESSION_SECRET='replace-with-a-long-random-secret'
docker compose up --build
```

访问 `http://localhost:3000`。SQLite 数据会保存到名为 `tarkov-data` 的 Docker 卷；容器重建不会清除该卷。

## 验证

```bash
cd backend
cargo fmt --check
cargo test

cd ../frontend
npm run build
```

容器镜像构建命令：

```bash
docker build -t tarkov-item-manager .
```
