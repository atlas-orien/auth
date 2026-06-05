# AI 驱动开发指南

本项目专为 **AI 编程助手**（Claude Code / Cursor / Codex / GitHub Copilot）设计。

> **核心理念**：人类描述需求，AI 编写代码。

---

## 快速开始

### 方式 1：Claude Code / Gemini CLI 等

```bash
# 在项目目录启动 AI 助手
# 然后输入你的需求，例如：
> 请先阅读 ai_protocols/TABLE_ADDING_PROTOCOL.md，然后帮我添加一个用户表
```

### 方式 2：Cursor 等带上下文的工具

1. 用 Cursor 打开项目
2. 按 `Cmd+K` 或在 Chat 窗口打开对话
3. 粘贴下方模板

### 方式 3：其他 Web 网页端 AI 工具

1. 将 `ai_protocols/TABLE_ADDING_PROTOCOL.md` 内容复制给 AI
2. 描述你的需求
3. AI 会按照协议生成代码

---

## 首次使用（必读）

在让 AI 开发功能之前，建议先让它阅读协议（特别是多轮对话中）：

```
请阅读 ai_protocols/TABLE_ADDING_PROTOCOL.md 文件，了解本项目的架构规范。
阅读完成后，告诉我你理解的要点。
```

AI 应该能总结出：
- 5 层架构及各层职责
- `db_core` 不可修改
- `repo` 的防腐特性及单表原则
- 跨表编排和聚合应在 `service` 层
- Web 接口在 `web-server` 层且绝对不能碰 `sea-orm`

---

## 提示词模板

### 模板 1：新增数据表

```
请严格按照 ai_protocols/TABLE_ADDING_PROTOCOL.md 执行。

需要新增一张表：
- 表名：<table_name>
- 表语义：<一句话描述这张表存储什么事实>
- 字段：
  - <field1>: <type> <约束>
  - <field2>: <type> <约束>
  - ...

完成以下工作：
1. migration 建表
2. 提示我运行 ./scripts/fresh_db.sh 生成 entity
3. 在 repo 中创建 dto 和 service

禁止修改 db_core。
```

**示例**：

```
请严格按照 ai_protocols/TABLE_ADDING_PROTOCOL.md 执行。

需要新增一张表：
- 表名：device
- 表语义：记录医疗设备信息
- 字段：
  - device_id: bigint (主键, 自增)
  - device_code: varchar(100) (唯一, 设备编号)
  - device_name: varchar(255) (设备名称)
  - device_type: varchar(50) (设备类型)
  - status: varchar(20) (状态: active/inactive)
  - created_at: timestamp

完成以下工作：
1. migration 建表
2. 提示我运行 ./scripts/fresh_db.sh 生成 entity
3. repo 中创建 dto 和 service

禁止修改 db_core。
```

---

### 模板 2：新增业务 API

```
请严格按照 ai_protocols/TABLE_ADDING_PROTOCOL.md 执行。

需要在 service crate 中新增业务 API：
- API 名称：<ApiName>
- 功能描述：<描述这个 API 做什么>
- 涉及的表：<table1>, <table2>, ...
- 输入参数：<参数列表>
- 返回数据：<返回内容描述>

在 service 层完成：
1. dto/<domain>.rs - 定义请求和响应结构
2. api/<domain>.rs - 实现 API，编排多个 repo 中的 Service

禁止修改 db_core。
禁止在 service 的 API 中直接操作数据库或引入 sea-orm。
```

**示例**：

```
请严格按照 ai_protocols/TABLE_ADDING_PROTOCOL.md 执行。

需要在 service crate 中新增业务 API：
- API 名称：DeviceApi
- 功能描述：设备管理，包括注册设备和查询设备绑定的观测数据
- 涉及的表：device, observation
- 输入参数：device_code, subject_id
- 返回数据：设备信息 + 该设备记录的最近观测

在 service 层完成：
1. dto/device_biz.rs - 定义请求和响应结构
2. api/device_biz.rs - 实现 API，编排 DeviceService 和 ObservationService

禁止修改 db_core。
禁止在 service 中直接操作数据库。
```

---

### 模板 3：新增 HTTP 接口

```
请严格按照 ai_protocols/TABLE_ADDING_PROTOCOL.md 执行。

需要在 web-server 中新增 HTTP 接口：
- 路径：<HTTP method> /<path>
- 功能：<描述>
- 请求参数：<Query/Body 参数>
- 响应格式：<响应结构>
- 调用的业务逻辑：<service crate 中的 API>

在 web-server 层完成：
1. dto/<domain>.rs - 请求/响应 DTO
2. handlers/<domain>.rs - Handler 实现
3. routes/<domain>.rs - 路由注册
4. 补充相关的 validator 验证规则。

禁止修改 db_core。
禁止在 Handler 中直接使用 repo 的 Service 或引入任何 sea-orm 数据结构。
```

---

### 模板 4：完整功能（表 + API + HTTP）

这是最常用的模板，一次性完成整个功能：

```
请严格按照 ai_protocols/TABLE_ADDING_PROTOCOL.md 执行。

需要实现完整功能：

### 1. 新增表
- 表名：<table_name>
- 字段：
  - <field1>: <type>
  - <field2>: <type>

### 2. 业务 API
- 功能：<描述>
- 输入：<参数>
- 输出：<返回>

### 3. HTTP 接口
- <METHOD> /<path> - <描述>

按 5 层架构顺序完成：
1. migration → 建表
2. 提示我运行脚本生成 entity
3. repo → 建立对应表的 dto 和单表 service
4. service → 新增负责编排和聚合逻辑的 api 与 dto
5. web-server → handler + route + dto，通过调用 service 输出结果

禁止修改 db_core，且只能在 repo 引入 sea-orm。
```

**示例**：

```
请严格按照 ai_protocols/TABLE_ADDING_PROTOCOL.md 执行。

需要实现完整功能：

### 1. 新增表
- 表名：prescription
- 字段：
  - prescription_id: bigint (主键)
  - subject_id: bigint (患者)
  - doctor_name: varchar(100)
  - medication: text (药品信息 JSON)
  - prescribed_at: timestamp
  - created_at: timestamp

### 2. 业务 API
- 功能：开具处方、查询患者处方历史
- 输入：subject_id, doctor_name, medication
- 输出：处方详情、处方列表

### 3. HTTP 接口
- POST /prescriptions - 创建处方
- GET /prescriptions?subject_id=xxx - 查询处方

按 5 层架构顺序完成：
1. migration → 建表
2. 提示我运行脚本生成 entity
3. repo → 建立 prescription 表的 dto 和 service
4. service → 实现 prescription_biz 的 api 与复合 dto
5. web-server → handler + route + dto

禁止修改 db_core。
```

---

### 模板 5：修改或修复 Bug

```
请严格按照 ai_protocols/TABLE_ADDING_PROTOCOL.md 的架构边界执行。

Bug 描述：<描述问题>
复现步骤：<如何复现>
期望行为：<应该是什么样>

修复时遵守：
- 不改变现有 5 层架构的调用链路
- 禁止修改 db_core
- 禁止在 service/web-server 泄漏 sea-orm
```

---

## 工作流程

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  1. 描述需求  │────▶│  2. AI 生成  │────▶│ 3. 运行验证  │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       ▼                   ▼                   ▼
   填写模板            按协议编码           cargo check
   复制给 AI          修改多个文件          cargo run
```

### 验证命令

```bash
# 编译检查
cargo check

# 运行服务
cargo run -p web-server

```

---

## 常见问题

### Q: AI 生成的代码编译不过？

A: 让 AI 修复：
```
cargo check 报错如下：
<粘贴错误信息>

请严格按照协议修复，尤其是检查是否出现了跨层不当的依赖。
```

### Q: AI 修改了 db_core 或在 service 写了 sea_orm 代码？

A: 明确告诉它：
```
你刚才破坏了架构限制（如修改了 db_core，或在 repo 外引入了 sea-orm），这是绝对禁止的。
请撤销这部分的修改，按照协议中的架构图重新实现。
```

### Q: 需要跨表查询怎么办？

A: 跨表逻辑放在 service 层：
```
需要在 service 层实现跨表查询：
- 从 subject repo 查询 subject 信息
- 同时从 observation repo 查询该 subject 的所有 observations
- 将两者数据在 service 的响应 dto 中聚合返回

禁止在 repo 层直接跨表 JOIN。
```

### Q: Entity 生成后字段不对？

A: 检查 migration，然后重新生成：
```bash
# 修改 migration 后
./scripts/fresh_db.sh
```

---

## 进阶用法

### 批量操作

```
请按顺序完成以下任务，每个任务按 ai_protocols/TABLE_ADDING_PROTOCOL.md 执行：

1. 在 repo 中新增 notification 表（通知记录）的基础增删改查
2. 在 service 中添加 NotificationBizApi 用于编排发送逻辑
3. 在 web-server 中添加 POST /notifications 接口

完成一个任务后，告诉我，我确认后你再继续下一个。
```

### 代码审查

```
请审查以下文件是否符合 ai_protocols/TABLE_ADDING_PROTOCOL.md 的架构规定：
- crates/repo/src/table/xxx/service.rs
- crates/service/src/api/xxx.rs

重点检查：
1. service 是否错误地引入了 sea_orm 或者直接操作了数据库。
2. 是否修改了 db_core。
3. 错误和 dto 转换是否符合层级职责。
```

---

## 最佳实践

1. **先让 AI 读协议**：首次使用或更换 Chat 时，先让 AI 阅读并总结协议，这非常有助于统一认知。
2. **一次一个功能**：避免一次性让 AI 做太多事情，防止超出上下文限制导致胡编乱造。
3. **及时验证**：每完成一个步骤就 `cargo check`，以排除级联错误。
4. **保留上下文**：在同一个对话中完成相关功能，如果 AI 开始乱写，立即打断并重申协议规范。
5. **明确约束**：每次 Prompt 都强调 `禁止修改 db_core，禁止在 repo 外使用 sea-orm`。
