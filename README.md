# AI Novel Agent Studio v0.4.0

桌面端 AI 小说创作工作台 + 嵌入式多模型 API 网关系统。

> **技术栈**: Tauri 2.0 + React 19 + TypeScript + Rust (Axum + Tokio)

---

## 🏗️ 项目架构

```
AI-Novel-Agent-Studio/
├── apps/desktop/          # React 19 前端 (TipTap 富文本编辑器 + TailwindCSS + Zustand)
└── src-tauri/             # Tauri 2.0 应用内核 (Rust Workspace)
    └── crates/
        ├── gateway/       # 多模型 API 网关 (OpenAI / Anthropic / Gemini / Ollama)
        ├── novel-core/    # 小说数据层 (章节 / 卷 / 设定库)
        ├── agent-core/    # AI Agent 系统 (摘要 / 压缩 / 审查 / 视觉 / Bash)
        └── rag-engine/    # 向量检索引擎 (LanceDB + SQLite FTS5 混合检索)
```

---

## 🚀 快速启动

```bash
# 1. 安装依赖
pnpm install

# 2. 前端开发 (Vite HMR)
pnpm dev

# 3. 启动 Tauri 桌面端 (内嵌 Rust 网关)
pnpm dev:tauri
```

---

## 📡 Gateway — 多模型 API 网关

`crates/gateway` 是一个嵌入式 Rust HTTP 网关，将外部 AI API 统一为 OpenAI 兼容接口。

### 支持的 Provider

| Provider         | 流式 | 非流式 | Tools | Vision | Thinking    |
| ---------------- | ---- | ------ | ----- | ------ | ----------- |
| **OpenAI**       | ✅   | ✅     | ✅    | ✅     | —           |
| **Anthropic**    | ✅   | ✅     | ✅    | ✅     | ✅ Extended |
| **Gemini**       | ✅   | ✅     | ✅    | ✅     | ✅ Thinking |
| **Ollama**       | ✅   | ✅     | —     | —      | —           |
| **自定义中转站** | ✅   | ✅     | ✅    | ✅     | ✅          |

### 网关能力

| 模块              | 功能                                                                 |
| ----------------- | -------------------------------------------------------------------- |
| `client.rs`       | **GatewayClient** — 面向 Agent 的进程内模型调用 API (不走 HTTP 中转) |
| `dispatcher.rs`   | Provider 路由 → Fallback 链降级 → Candidate 优先级                   |
| `guards.rs`       | CircuitBreaker 熔断 + 并发信号量限流 + SSE 超时守护                  |
| `embedding.rs`    | Embedding 向量缓存 (SHA-256 指纹去重)                                |
| `rerank.rs`       | Jina/Cohere Rerank 适配                                              |
| `reasoning.rs`    | `<think>` 标签提取 + 推理内容 SSE chunk 构建                         |
| `privacy_mask.rs` | API Key / Bearer Token 自动脱敏                                      |
| `i18n.rs`         | zh-CN / en-US / ja-JP 三语错误信息                                   |
| `registry.rs`     | 模型注册表 (TPS/Cost 自动计算)                                       |

### 网关 HTTP API

```bash
POST /v1/chat/completions    # OpenAI 兼容 Chat (流式/非流式)
POST /v1/messages            # Anthropic Messages 原生协议
POST /v1beta/models/{path}   # Gemini 原生协议
POST /v1/responses           # OpenAI Responses API
POST /v1/embeddings          # Embedding 向量
POST /v1/rerank              # Rerank 重排序
POST /v1/images/generations  # 图片生成
POST /v1/images/edits        # 图片编辑
POST /v1/images/variations   # 图片变体
GET  /v1/models              # 模型列表
GET  /v1/models/registry     # 模型注册表
POST /v1/gateway/config      # 网关配置
POST /v1/gateway/test-model  # 模型连通测试
POST /v1/gateway/fetch-upstream-models  # 拉取上游模型列表
```

### GatewayClient 使用示例

```rust
use gateway::client::{GatewayClient, ChatOptions};
use gateway::types::ChatMessage;

// 流式调用
let stream = GatewayClient::chat_stream(
    "gpt-4o",
    &[ChatMessage { role: "user".into(), content: "你好".into(), ..Default::default() }],
    ChatOptions::default(),
).await?;

// 非流式调用 — 直接获取完整文本
let reply = GatewayClient::chat_complete("gpt-4o", &messages, ChatOptions::default()).await?;
```

---

## 🤖 agent-core — AI Agent 系统

对标 [snow-cli](https://github.com/MayDay-wpf/snow-cli) Agent 架构，基于 `GatewayClient` 实现。

| Agent                    | 对标 snow-cli               | 功能                               | 使用的模型      |
| ------------------------ | --------------------------- | ---------------------------------- | --------------- |
| `SummaryAgent`           | `summaryAgent.ts`           | 对话标题 + 摘要生成 (≤50/150 字符) | `basicModel`    |
| `CompactAgent`           | `compactAgent.ts`           | 网页/大文本内容提取压缩            | `basicModel`    |
| `ReviewAgent`            | `reviewAgent.ts`            | Git Diff / 代码片段审查            | `advancedModel` |
| `VisionAgent`            | `visionAgent.ts`            | 图片 → 文字描述 (视觉回退)         | `visionModel`   |
| `BashOutputSummaryAgent` | `bashOutputSummaryAgent.ts` | 终端命令输出压缩 (错误优先)        | `basicModel`    |

### Agent 使用示例

```rust
use agent_core::{SummaryAgent, Agent, AgentConfig};

let mut agent = SummaryAgent::new();
let config = AgentConfig::new("gpt-4o-mini");

let summary = agent.generate_summary(
    &config,
    "帮我写一段修仙小说的开头",
    "好的，以下是修仙小说的开篇：……",
).await?;

println!("Title: {}", summary.title);
println!("Summary: {}", summary.summary);
```

### Agent Trait 接口

```rust
pub trait Agent: Send + Sync {
    fn initialize(&mut self, config: &AgentConfig) -> bool;
    fn clear_cache(&mut self);
    fn is_available(&mut self, config: &AgentConfig) -> bool;
    fn name(&self) -> &'static str;
}
```

---

## 📖 novel-core — 小说数据层

`crates/novel-core` 提供小说创作的核心数据结构：

- `models/` — Chapter, Volume, Project, CharacterCard, WorldSetting
- `db/` — SQLite 持久化 (rusqlite + r2d2 连接池)
- 卷-章节树形结构 + 设定库关联查询

---

## 🔍 rag-engine — 向量检索引擎

`crates/rag-engine` 提供混合检索能力：

- **LanceDB** — 向量语义索引 (Embedding 相似度搜索)
- **SQLite FTS5** — 全文关键词检索
- 混合排序 (语义 + 关键词融合)

---

## 🛡️ Gateway 保障机制

| 机制               | 说明                                       |
| ------------------ | ------------------------------------------ |
| **CircuitBreaker** | 同一 Provider 连续 3 次失败 → 60s 冷却熔断 |
| **Failover**       | 模型匹配 → Fallback 链 → 任意可用 Provider |
| **Concurrency**    | Provider 级信号量限流 (防 429)             |
| **Retry**          | 指数退避重试 (429/502/503/504)             |
| **SSE Idle Guard** | 流式超时守护 (180s)                        |

---

## 📦 构建

```bash
# 检查所有 Rust crate
cargo check

# 仅检查 gateway
cargo check -p gateway

# 仅检查 agent-core
cargo check -p agent-core

# Release 构建
cargo build --release

# Tauri 打包
pnpm tauri build
```

---

## 📝 配置文件

| 文件                    | 用途                                          |
| ----------------------- | --------------------------------------------- |
| `config/providers.json` | Provider 配置 (API Key / Base URL / 模型列表) |
| `config/models.json`    | 模型注册表 (pricing / context window 等)      |
| `prompts/writer.md`     | 写手 Agent 系统提示词                         |
| `prompts/editor.md`     | 编辑 Agent 系统提示词                         |
| `prompts/reviewer.md`   | 审稿 Agent 系统提示词                         |
| `prompts/summarizer.md` | 总结 Agent 系统提示词                         |

---

## 📄 License

MIT
