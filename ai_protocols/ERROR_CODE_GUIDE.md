# Error Code 设计协议 (AI 专用)

> 本文档是错误码体系的执行规约。AI 在新增业务错误、改造错误链路时，必须遵循本协议。

---

## 一、目标

1. 统一业务错误表达：业务失败必须使用 `BizError`。
2. 前端可稳定展示用户文案：前端仅通过 `code` 映射用户可见消息。
3. 分离用户提示与技术细节：响应体里的 `message` 仅用于日志与调试，不作为用户主文案。

---

## 二、统一约定

1. **业务错误（BizError）统一返回 HTTP 400**
2. **系统/内部错误统一返回 HTTP 500**
3. `BizError` 结构保持最小集：`code` + `message`
4. 前端展示文案必须以 `code` 为主，不依赖响应体里的 `message`
5. 新增错误码必须在 `crates/error-code` 中集中定义，禁止手写魔法数字
6. 每个错误码必须能对应一条精确、稳定的用户消息；不同用户消息不能共用同一个错误码

---

## 三、分层职责

### 1) error-code crate

- 位置：`crates/error-code`
- 职责：维护稳定、可复用的业务错误码常量，以及错误码对应的用户消息索引
- 规则：
  - 按业务域拆分模块（例如：`auth`）
  - 常量名需表达业务语义（如 `USER_NOT_FOUND`）
  - 每个码值必须有明确用户语义
  - 可提供 `user_message(code)` 作为前端语言包的基准文案/索引

### 2) service/repo 层

- 业务可预期失败使用 `BizError::new(code, message)`
- `code` 必须来自 `error-code` crate
- 响应体里的 `message` 可包含调试细节，但要避免敏感信息泄露

### 3) web-server 层

- `from_biz_error` 将 `BizError` 映射为 HTTP 400
- 非 `BizError` 异常映射为 HTTP 500
- 响应体至少包含 `code`、`message`

---

## 四、前端用户消息契约

### 1) 基础形态（推荐）

```json
{
  "default": "request failed, please try again later",
  "-1000": "username already exists",
  "-1001": "email already exists",
  "-1002": "user not found"
}
```

### 2) 使用规则

1. 前端根据响应 `code` 查找语言包
2. 命中则展示语言包文案
3. 未命中则展示 `default`
4. 响应体里的 `message` 仅日志记录，不直接展示给终端用户

---

## 五、变更流程

1. 先在 `crates/error-code` 新增常量
2. 在 `service/repo` 引用常量替换手写数字
3. 检查 `web-server` 错误映射是否符合 400/500 约定
4. 补充错误码对应的用户消息，并更新前端语言包（至少补充新 `code`）
5. 运行 `cargo check`

---

## 六、自检清单

- [ ] 是否仍存在手写错误码（如 `-1`）？
- [ ] 新增业务错误是否都来自 `crates/error-code`？
- [ ] `BizError` 是否统一走 HTTP 400？
- [ ] 系统错误是否统一走 HTTP 500？
- [ ] 前端是否只按 `code` 做用户可见文案？
- [ ] 每个错误码是否对应一条精确用户消息？
- [ ] 响应体里的 `message` 是否仅用于日志/调试？
