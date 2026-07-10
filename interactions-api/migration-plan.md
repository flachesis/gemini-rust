# Gemini-Rust Interactions API 遷移計畫

> **策略**：並存（Interactions API + 既有 generateContent API）
> **相容性**：平滑遷移（既有 API 標記 `#[deprecated]`，保留可用）
> **範圍**：完整實作所有 Interactions API 功能，全部範例重寫

---

## 目錄

1. [策略總覽](#策略總覽)
2. [目錄結構變更](#目錄結構變更)
3. [Phase 1：核心型別系統](#phase-1核心型別系統-srcinteractionsmodelrs)
4. [Phase 2：InteractionBuilder](#phase-2interactionbuilder-srcinteractionsbuilderrs)
5. [Phase 3：HTTP Client 擴充](#phase-3http-client-擴充-srcclientrs)
6. [Phase 4：SSE Streaming](#phase-4sse-streaming-srcinteractionsstreamrs)
7. [Phase 5：InteractionHandle](#phase-5interactionhandle-srcinteractionshandlers)
8. [Phase 6：功能實作](#phase-6功能實作)
9. [Phase 7：便利方法](#phase-7便利方法-impl-interaction)
10. [Phase 8：範例全部重寫](#phase-8範例全部重寫)
11. [Phase 9：棄用標記與文件](#phase-9棄用標記與文件)
12. [實作順序](#實作順序建議分-7-個-prcommit)
13. [型別對照表](#型別對照表)
14. [端點對照表](#端點對照表)

---

## 策略總覽

### 範式轉變

```
generateContent（舊）                      Interactions API（新）
─────────────────────────                  ─────────────────────────────
Contents (messages) ──→ Candidates (parts)  Interaction (input) ──→ Steps (typed actions)
POST models/{model}:generateContent          POST /v1beta/interactions
```

### 核心差異

| 面向 | generateContent | Interactions API |
|------|----------------|-----------------|
| 請求結構 | `contents: Vec<Content>` (role + parts) | `input: string \| Content[] \| Step[]` |
| 回應結構 | `candidates: Vec<Candidate>` (parts) | `steps: Vec<Step>` (typed enum) |
| 多輪狀態 | 客戶端管理完整 history | server-side `previous_interaction_id` |
| Streaming | SSE 回傳多個 `GenerationResponse` chunk | SSE 回傳 step 生命週期事件 (`step.start/delta/stop`) |
| 函式呼叫 | `Part::FunctionCall` / `Part::FunctionResponse` | `Step::FunctionCall` / `Step::FunctionResult` |
| 執行步驟可觀察性 | 無 | `thought`, `google_search_call`, `code_execution_call` 等獨立 step |
| 背景執行 | 不支援 | `background=true` + `interactions.get` 輪詢 |
| Managed Agents | 不支援 | `agent` 參數 (Deep Research, Antigravity) |
| Server-side state | 不支援 | `previous_interaction_id` + `store=true` |
| 環境 | 不支援 | `environment: "remote"` (sandbox) |
| Service tier | 不支援 | `flex` / `standard` / `priority` |
| Webhook | 不支援 | `webhook_config` |

### Interactions API 不支援的功能（繼續使用 generateContent）

- **Batch API** — 批次處理
- **Explicit caching** — 顯式快取（Interactions API 有隱式快取 via `previous_interaction_id`）
- **Custom safety settings** — 自訂安全設定
- **Video metadata** — 影片中繼資料（clipping、frame rate）
- **Automatic function calling** — 自動函式呼叫

---

## 目錄結構變更

```
src/
├── client.rs           ← 擴充：新增 interaction 相關方法
├── interactions/       ← 【新增】Interactions API 模組
│   ├── mod.rs          ← 模組入口 + re-exports
│   ├── model.rs        ← Interaction, Step, Content, Tool, ResponseFormat 等型別
│   ├── builder.rs      ← InteractionBuilder（流暢 API）
│   ├── handle.rs       ← InteractionHandle（get / cancel / delete / poll）
│   ├── stream.rs       ← SSE event 解析 + InteractionStream
│   └── serde.rs        ← polymorphic 型別的 serde 輔助
├── generation/         ← 【保留】標記 #[deprecated]
├── models.rs           ← 【保留】共用型別
├── tools/              ← 【保留】FunctionDeclaration 等仍被 interactions 使用
├── batch/              ← 【保留】Interactions API 不支援
├── cache/              ← 【保留】explicit caching 仍需要
├── embedding/          ← 【保留】
├── files/              ← 【保留】
├── file_search/        ← 【保留】
├── safety/             ← 【保留】Interactions API 不支援 custom safety
├── common/             ← 【保留】
├── lib.rs              ← 擴充：新增 interactions module + re-exports
├── prelude.rs          ← 擴充：新增 interactions 常用型別
└── tests.rs            ← 擴充：新增 interactions 測試
```

---

## Phase 1：核心型別系統 (`src/interactions/model.rs`)

### 1.1 Content 型別

Interactions API 的 Content 使用 `type` discriminator 的 polymorphic 結構，與既有 `Part` enum 完全不同。

```rust
use serde::{Deserialize, Serialize};

/// 輸入用 Content — type-tagged polymorphic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InteractionContent {
    Text {
        text: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        annotations: Vec<Annotation>,
    },
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<ImageMimeType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        resolution: Option<MediaResolution>,
    },
    Audio {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<AudioMimeType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        channels: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sample_rate: Option<i32>,
    },
    Document {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<DocumentMimeType>,
    },
    Video {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<VideoMimeType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        resolution: Option<MediaResolution>,
    },
}
```

**與既有型別的對應關係**：

| generateContent | Interactions API |
|-----------------|-----------------|
| `Part::Text { text, thought, thought_signature }` | `Step::Thought { signature, summary }` + `InteractionContent::Text { text }` |
| `Part::InlineData { inline_data: Blob }` | `InteractionContent::Image/Audio/Video/Document { data, mime_type }` |
| `Part::FunctionCall { function_call }` | `Step::FunctionCall { name, arguments, id }` |
| `Part::FunctionResponse { function_response }` | `Step::FunctionResult { name, call_id, result, is_error }` |
| `Part::FileData { file_data }` | `InteractionContent::Image/Audio/Video { uri, mime_type }` |
| `Candidate` | `Step::ModelOutput { content }` |
| `GenerationResponse.candidates[0]` | `Interaction.steps[]` 中 filter `type == "model_output"` |

### 1.2 Annotation 型別

```rust
/// 引用標注 — type-tagged polymorphic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotation {
    UrlCitation {
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_index: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        end_index: Option<i64>,
    },
    FileCitation {
        #[serde(skip_serializing_if = "Option::is_none")]
        document_uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        custom_metadata: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        page_number: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        media_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_index: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        end_index: Option<i64>,
    },
    PlaceCitation {
        #[serde(skip_serializing_if = "Option::is_none")]
        place_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        review_snippets: Option<ReviewSnippet>,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_index: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        end_index: Option<i64>,
    },
}
```

### 1.3 Step 型別

Step 是 Interactions API 的核心 — 每個 step 代表互動歷史中的一個型別化動作。

```rust
/// 互動中的一個步驟 — type-tagged polymorphic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Step {
    /// 使用者輸入
    UserInput {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<InteractionContent>,
    },

    /// 模型輸出
    ModelOutput {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<InteractionContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<Status>,
    },

    /// 模型思考
    Thought {
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        summary: Vec<ThoughtSummaryContent>,
    },

    /// 函式呼叫（模型要求呼叫函式）
    FunctionCall {
        name: String,
        arguments: serde_json::Value,
        id: String,
    },

    /// 函式結果（使用者回傳函式執行結果）
    FunctionResult {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        call_id: String,
        result: StepResult,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },

    /// 程式碼執行呼叫
    CodeExecutionCall {
        arguments: CodeExecutionCallArguments,
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    /// 程式碼執行結果
    CodeExecutionResult {
        result: String,
        call_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    /// URL 上下文呼叫
    UrlContextCall {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
        arguments: UrlContextCallArguments,
    },

    /// URL 上下文結果
    UrlContextResult {
        result: UrlContextResult,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
        call_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    /// Google 搜尋呼叫
    GoogleSearchCall {
        arguments: GoogleSearchCallArguments,
        #[serde(skip_serializing_if = "Option::is_none")]
        search_type: Option<GoogleSearchType>,
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    /// Google 搜尋結果
    GoogleSearchResult {
        result: GoogleSearchResultItem,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
        call_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    /// Google Maps 呼叫
    GoogleMapsCall {
        #[serde(skip_serializing_if = "Option::is_none")]
        arguments: Option<GoogleMapsCallArguments>,
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    /// Google Maps 結果
    GoogleMapsResult {
        result: GoogleMapsResultItem,
        call_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    /// 檔案搜尋呼叫
    FileSearchCall {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    /// 檔案搜尋結果
    FileSearchResult {
        call_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    /// MCP Server 工具呼叫
    McpServerToolCall {
        name: String,
        server_name: String,
        arguments: serde_json::Value,
        id: String,
    },

    /// MCP Server 工具結果
    McpServerToolResult {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        server_name: Option<String>,
        call_id: String,
        result: StepResult,
    },
}
```

#### Step 子型別

```rust
/// 函式結果可以是 text/image content 陣列、JSON object、或字串
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StepResult {
    ContentArray(Vec<StepResultContent>),
    Object(serde_json::Value),
    String(String),
}

/// StepResult 的 content 限制為 ImageContent 或 TextContent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepResultContent {
    Text { text: String },
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<ImageMimeType>,
    },
}

/// 思考摘要內容 — polymorphic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThoughtSummaryContent {
    Text { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeExecutionCallArguments {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<CodeLanguage>,  // python
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UrlContextCallArguments {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoogleSearchCallArguments {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoogleMapsCallArguments {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoogleSearchResultItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_suggestions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoogleMapsResultItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub places: Option<GoogleMapsResultPlaces>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoogleMapsResultPlaces {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_snippets: Option<ReviewSnippet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget_context_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UrlContextResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<UrlContextStatus>,  // success, error, paywall, unsafe
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Status {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewSnippet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_id: Option<String>,
}
```

### 1.4 Interaction 資源

```rust
/// Interaction 資源 — 代表一次完整的互動
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Interaction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(default)]
    pub status: InteractionStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,  // "interaction"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<Step>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<InteractionUsage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<InteractionTool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_interaction_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_modalities: Option<Vec<ResponseModality>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_config: Option<AgentConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<InteractionError>,
}

/// 互動狀態
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InteractionStatus {
    #[default]
    InProgress,
    RequiresAction,
    Completed,
    Failed,
    Cancelled,
    Incomplete,
    BudgetExceeded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InteractionError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
```

### 1.5 Tool 型別

```rust
/// 工具宣告 — type-tagged polymorphic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InteractionTool {
    /// 自訂函式
    Function {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<serde_json::Value>,
    },

    /// 程式碼執行
    CodeExecution,

    /// URL 上下文
    UrlContext,

    /// 電腦操作
    ComputerUse {
        #[serde(skip_serializing_if = "Option::is_none")]
        environment: Option<ComputerUseEnvironment>,  // browser, mobile, desktop
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        excluded_predefined_functions: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        enable_prompt_injection_detection: Option<bool>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        disabled_safety_policies: Vec<ComputerUseSafetyPolicy>,
    },

    /// MCP Server
    McpServer {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_tools: Option<AllowedTools>,
    },

    /// Google 搜尋
    GoogleSearch {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        search_types: Vec<GoogleSearchType>,  // web_search, image_search, enterprise_web_search
    },

    /// 檔案搜尋
    FileSearch {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        file_search_store_names: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        top_k: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata_filter: Option<String>,
    },

    /// Google Maps
    GoogleMaps {
        #[serde(skip_serializing_if = "Option::is_none")]
        enable_widget: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        latitude: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        longitude: Option<f64>,
    },

    /// 檔案擷取
    Retrieval {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        retrieval_types: Vec<RetrievalType>,  // rag_store, exa_ai_search, parallel_ai_search
        #[serde(skip_serializing_if = "Option::is_none")]
        exa_ai_search_config: Option<ExaAISearchConfig>,
        #[serde(skip_serializing_if = "Option::is_none")]
        parallel_ai_search_config: Option<ParallelAISearchConfig>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rag_store_config: Option<RagStoreConfig>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AllowedTools {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ToolChoiceMode>,  // auto, any, none, validated
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<String>,
}
```

### 1.6 ResponseFormat 型別

```rust
/// 回應格式設定 — type-tagged polymorphic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    Text {
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<TextMimeType>,  // application/json, text/plain
        #[serde(skip_serializing_if = "Option::is_none")]
        schema: Option<serde_json::Value>,
    },
    Audio {
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<AudioOutputMimeType>,  // audio/mp3, audio/ogg_opus, audio/l16, audio/wav, audio/alaw, audio/mulaw
        #[serde(skip_serializing_if = "Option::is_none")]
        delivery: Option<DeliveryMode>,  // inline, uri
        #[serde(skip_serializing_if = "Option::is_none")]
        sample_rate: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        bit_rate: Option<i32>,
    },
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<ImageOutputMimeType>,  // image/jpeg
        #[serde(skip_serializing_if = "Option::is_none")]
        delivery: Option<DeliveryMode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        aspect_ratio: Option<AspectRatio>,  // 1:1, 2:3, 3:2, 3:4, 4:3, 4:5, 5:4, 9:16, 16:9, 21:9, 1:8, 8:1, 1:4, 4:1
        #[serde(skip_serializing_if = "Option::is_none")]
        image_size: Option<ImageSize>,  // 512, 1K, 2K, 4K
    },
    Video {
        #[serde(skip_serializing_if = "Option::is_none")]
        delivery: Option<DeliveryMode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        gcs_uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        aspect_ratio: Option<VideoAspectRatio>,  // 16:9, 9:16
        #[serde(skip_serializing_if = "Option::is_none")]
        duration: Option<String>,
    },
}
```

### 1.7 GenerationConfig（Interactions 版本）

```rust
/// 模型互動的設定參數（與 agent_config 互斥，僅適用於 model）
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InteractionGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<InteractionThinkingLevel>,  // minimal, low, medium, high

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_summaries: Option<ThinkingSummaries>,  // auto, none

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech_config: Option<InteractionSpeechConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_config: Option<VideoConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoiceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InteractionThinkingLevel {
    Minimal,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingSummaries {
    Auto,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InteractionSpeechConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VideoConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<VideoTask>,  // text_to_video, image_to_video, reference_to_video, edit
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolChoiceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<AllowedTools>,
}
```

### 1.8 Usage 型別

```rust
/// Token 使用量統計
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InteractionUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_input_tokens: Option<i64>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_tokens_by_modality: Vec<ModalityTokens>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cached_tokens: Option<i64>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cached_tokens_by_modality: Vec<ModalityTokens>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_output_tokens: Option<i64>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_tokens_by_modality: Vec<ModalityTokens>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tool_use_tokens: Option<i64>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_use_tokens_by_modality: Vec<ModalityTokens>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_thought_tokens: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_tool_count: Option<GroundingToolCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModalityTokens {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<ResponseModality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroundingToolCount {
    #[serde(rename = "type")]
    pub grounding_type: Option<GroundingType>,  // google_search, google_maps, retrieval
    pub count: Option<i64>,
}
```

### 1.9 AgentConfig 型別

```rust
/// Agent 設定（與 generation_config 互斥，僅適用於 agent）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum AgentConfig {
    /// 動態 Agent
    Dynamic,

    /// Deep Research Agent
    DeepResearch {
        #[serde(skip_serializing_if = "Option::is_none")]
        thinking_summaries: Option<ThinkingSummaries>,
        #[serde(skip_serializing_if = "Option::is_none")]
        visualization: Option<Visualization>,  // off, auto
        #[serde(skip_serializing_if = "Option::is_none")]
        collaborative_planning: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        enable_bigquery_tool: Option<bool>,
    },
}
```

### 1.10 其他列舉與輔助型別

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseModality {
    Text,
    Image,
    Audio,
    Video,
    Document,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTier {
    Flex,
    Standard,
    Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryMode {
    Inline,
    Uri,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MediaResolution {
    Low,
    Medium,
    High,
    UltraHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImageMimeType {
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/webp")]
    Webp,
    #[serde(rename = "image/heic")]
    Heic,
    #[serde(rename = "image/heif")]
    Heif,
    #[serde(rename = "image/gif")]
    Gif,
    #[serde(rename = "image/bmp")]
    Bmp,
    #[serde(rename = "image/tiff")]
    Tiff,
}

// ... 其他 MIME type enums（AudioMimeType, VideoMimeType, DocumentMimeType 等）
// ... AspectRatio, ImageSize, VideoAspectRatio, VideoTask, CodeLanguage,
//     GoogleSearchType, RetrievalType, ComputerUseEnvironment, ComputerUseSafetyPolicy,
//     UrlContextStatus, ToolChoiceMode, Visualization

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WebhookConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uris: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_metadata: Option<serde_json::Value>,
}

/// 環境設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EnvironmentConfig {
    Remote {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        sources: Vec<EnvironmentSource>,
        #[serde(skip_serializing_if = "Option::is_none")]
        environment_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        network: Option<EnvironmentNetwork>,
    },
}

/// 環境來源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EnvironmentSource {
    Gcs {
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
    },
    Inline {
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        encoding: Option<String>,
    },
    Repository {
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
    },
    SkillRegistry {
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
    },
}
```

### 1.11 CreateInteractionRequest

```rust
/// 建立 Interaction 的請求體
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInteractionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    pub input: InteractionInput,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<InteractionTool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<InteractionGenerationConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_config: Option<AgentConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_interaction_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<EnvironmentConfigOrString>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_content: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub response_modalities: Vec<ResponseModality>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_config: Option<WebhookConfig>,
}

/// input 可以是字串、單一 Content、Content 陣列、Step 陣列、或 Turn 陣列
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InteractionInput {
    Text(String),
    Content(InteractionContent),
    ContentArray(Vec<InteractionContent>),
    StepArray(Vec<Step>),
}
```

---

## Phase 2：InteractionBuilder (`src/interactions/builder.rs`)

```rust
use std::sync::Arc;
use tracing::instrument;

use crate::{
    client::{Error as ClientError, GeminiClient},
    interactions::{
        model::*,
        stream::InteractionStream,
    },
};

/// Interaction 請求的流暢建構器
#[derive(Clone)]
pub struct InteractionBuilder {
    client: Arc<GeminiClient>,
    model: Option<String>,
    agent: Option<String>,
    input: Option<InteractionInput>,
    system_instruction: Option<String>,
    tools: Vec<InteractionTool>,
    response_format: Option<ResponseFormat>,
    stream: bool,
    store: Option<bool>,
    background: bool,
    generation_config: Option<InteractionGenerationConfig>,
    agent_config: Option<AgentConfig>,
    previous_interaction_id: Option<String>,
    environment: Option<EnvironmentConfigOrString>,
    cached_content: Option<String>,
    response_modalities: Vec<ResponseModality>,
    service_tier: Option<ServiceTier>,
    webhook_config: Option<WebhookConfig>,
}

impl InteractionBuilder {
    pub(crate) fn new(client: Arc<GeminiClient>) -> Self { ... }

    // ===== Model / Agent =====

    /// 設定模型（與 agent 互斥）
    pub fn with_model(mut self, model: impl Into<String>) -> Self { ... }

    /// 設定 agent（與 model 互斥）
    pub fn with_agent(mut self, agent: impl Into<String>) -> Self { ... }

    // ===== Input =====

    /// 設定輸入為純文字
    pub fn with_input(mut self, input: impl Into<String>) -> Self { ... }

    /// 設定輸入為 Content 陣列（多模態）
    pub fn with_content_input(mut self, content: Vec<InteractionContent>) -> Self { ... }

    /// 設定輸入為 Step 陣列（stateless 多輪）
    pub fn with_step_input(mut self, steps: Vec<Step>) -> Self { ... }

    /// 新增文字輸入（便利方法）
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.input = Some(InteractionInput::Text(text.into()));
        self
    }

    /// 新增圖片輸入
    pub fn with_image(mut self, data: impl Into<String>, mime_type: ImageMimeType) -> Self { ... }

    /// 新增音訊輸入
    pub fn with_audio(mut self, data: impl Into<String>, mime_type: AudioMimeType) -> Self { ... }

    /// 新增影片輸入
    pub fn with_video(mut self, uri: impl Into<String>) -> Self { ... }

    /// 新增文件輸入
    pub fn with_document(mut self, data: impl Into<String>, mime_type: DocumentMimeType) -> Self { ... }

    // ===== System Instruction =====

    /// 設定系統指令
    pub fn with_system_instruction(mut self, instruction: impl Into<String>) -> Self { ... }

    // ===== Tools =====

    /// 新增工具
    pub fn with_tool(mut self, tool: InteractionTool) -> Self { ... }

    /// 新增函式工具
    pub fn with_function(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self { ... }

    /// 啟用 Google 搜尋
    pub fn with_google_search(mut self) -> Self { ... }

    /// 啟用程式碼執行
    pub fn with_code_execution(mut self) -> Self { ... }

    /// 啟用 URL 上下文
    pub fn with_url_context(mut self) -> Self { ... }

    /// 啟用 Google Maps
    pub fn with_google_maps(mut self) -> Self { ... }

    /// 啟用檔案搜尋
    pub fn with_file_search(mut self, store_names: Vec<String>) -> Self { ... }

    /// 啟用 MCP Server
    pub fn with_mcp_server(mut self, name: impl Into<String>, url: impl Into<String>) -> Self { ... }

    /// 啟用電腦操作
    pub fn with_computer_use(mut self, environment: ComputerUseEnvironment) -> Self { ... }

    // ===== Response Format =====

    /// 設定回應格式
    pub fn with_response_format(mut self, format: ResponseFormat) -> Self { ... }

    /// 設定 JSON structured output
    pub fn with_json_schema(mut self, schema: serde_json::Value) -> Self { ... }

    // ===== Generation Config =====

    pub fn with_temperature(mut self, temperature: f64) -> Self { ... }
    pub fn with_top_p(mut self, top_p: f64) -> Self { ... }
    pub fn with_seed(mut self, seed: i64) -> Self { ... }
    pub fn with_stop_sequences(mut self, sequences: Vec<String>) -> Self { ... }
    pub fn with_max_output_tokens(mut self, tokens: i64) -> Self { ... }
    pub fn with_thinking_level(mut self, level: InteractionThinkingLevel) -> Self { ... }
    pub fn with_thinking_summaries(mut self, summaries: ThinkingSummaries) -> Self { ... }
    pub fn with_presence_penalty(mut self, penalty: f64) -> Self { ... }
    pub fn with_frequency_penalty(mut self, penalty: f64) -> Self { ... }
    pub fn with_tool_choice(mut self, choice: ToolChoiceConfig) -> Self { ... }
    pub fn with_speech_config(mut self, config: InteractionSpeechConfig) -> Self { ... }
    pub fn with_video_config(mut self, config: VideoConfig) -> Self { ... }

    // ===== Agent Config =====

    pub fn with_agent_config(mut self, config: AgentConfig) -> Self { ... }

    // ===== Interaction Options =====

    /// 設定 previous_interaction_id（server-side state）
    pub fn with_previous_interaction(mut self, id: impl Into<String>) -> Self { ... }

    /// 設定 store=false（stateless 模式）
    pub fn with_store(mut self, store: bool) -> Self { ... }

    /// 啟用背景執行
    pub fn with_background(mut self) -> Self { ... }

    /// 設定環境
    pub fn with_environment(mut self, env: EnvironmentConfig) -> Self { ... }

    /// 設定 cached_content
    pub fn with_cached_content(mut self, cached_content: impl Into<String>) -> Self { ... }

    /// 設定回應模式
    pub fn with_response_modalities(mut self, modalities: Vec<ResponseModality>) -> Self { ... }

    /// 設定 service tier
    pub fn with_service_tier(mut self, tier: ServiceTier) -> Self { ... }

    /// 設定 webhook
    pub fn with_webhook_config(mut self, config: WebhookConfig) -> Self { ... }

    // ===== Build & Execute =====

    /// 建構請求
    pub fn build(self) -> Result<CreateInteractionRequest, ClientError> { ... }

    /// 執行（非串流）
    #[instrument(skip_all, fields(
        model = self.model.as_deref().unwrap_or(""),
        agent = self.agent.as_deref().unwrap_or(""),
        tools.count = self.tools.len(),
        system.instruction.present = self.system_instruction.is_some(),
        background = self.background,
        previous.interaction.present = self.previous_interaction_id.is_some(),
    ))]
    pub async fn execute(self) -> Result<Interaction, ClientError> { ... }

    /// 執行（串流）
    #[instrument(skip_all, fields(
        model = self.model.as_deref().unwrap_or(""),
        agent = self.agent.as_deref().unwrap_or(""),
        tools.count = self.tools.len(),
    ))]
    pub async fn execute_stream(self) -> Result<InteractionStream, ClientError> { ... }
}
```

### ContentBuilder → InteractionBuilder 方法對照

| 現有 ContentBuilder | 新 InteractionBuilder |
|---------------------|----------------------|
| `.with_user_message(text)` | `.with_text(text)` |
| `.with_system_prompt(text)` | `.with_system_instruction(text)` |
| `.with_tool(tool)` | `.with_tool(InteractionTool::...)` |
| `.with_function(func)` | `.with_function(name, description, params)` |
| `.with_temperature(t)` | `.with_temperature(t)` |
| `.with_response_schema(s)` | `.with_json_schema(s)` |
| `.with_code_execution()` | `.with_code_execution()` |
| `.with_thinking_level(level)` | `.with_thinking_level(level)` |
| `.execute()` → `GenerationResponse` | `.execute()` → `Interaction` |
| `.execute_stream()` → `GenerationStream` | `.execute_stream()` → `InteractionStream` |
| `.count_tokens()` | *(Interactions API 無此端點)* |
| *(不存在)* | `.with_previous_interaction(id)` |
| *(不存在)* | `.with_background()` |
| *(不存在)* | `.with_agent(name)` |
| *(不存在)* | `.with_store(false)` |
| *(不存在)* | `.with_environment(env)` |
| *(不存在)* | `.with_service_tier(tier)` |
| *(不存在)* | `.with_webhook_config(config)` |
| *(不存在)* | `.with_google_search()` |
| *(不存在)* | `.with_url_context()` |
| *(不存在)* | `.with_mcp_server(name, url)` |
| *(不存在)* | `.with_computer_use(env)` |

---

## Phase 3：HTTP Client 擴充 (`src/client.rs`)

在 `GeminiClient` 上新增內部方法：

```rust
impl GeminiClient {
    /// 建立 interaction（非串流）
    #[instrument(skip_all, fields(
        model = request.model.as_deref().unwrap_or(""),
        agent = request.agent.as_deref().unwrap_or(""),
        tools.count = request.tools.len(),
        background = request.background.unwrap_or(false),
        previous.interaction.present = request.previous_interaction_id.is_some(),
        status.code,
        usage.total_tokens,
    ))]
    pub(crate) async fn create_interaction(
        &self,
        request: CreateInteractionRequest,
    ) -> Result<Interaction, Error> {
        let url = self.build_url_with_suffix("interactions")?;
        let response: Interaction = self.post_json(url, &request).await?;

        // 記錄 usage
        if let Some(usage) = &response.usage {
            Span::current()
                .record("status.code", response.status.as_ref())
                .record("usage.total_tokens", usage.total_tokens);
        }

        Ok(response)
    }

    /// 建立 interaction（串流）
    #[instrument(skip_all, fields(
        model = request.model.as_deref().unwrap_or(""),
        agent = request.agent.as_deref().unwrap_or(""),
        tools.count = request.tools.len(),
    ))]
    pub(crate) async fn create_interaction_stream(
        &self,
        request: CreateInteractionRequest,
    ) -> Result<InteractionStream, Error> {
        let mut url = self.build_url_with_suffix("interactions")?;
        url.query_pairs_mut().append_pair("alt", "sse");

        let stream = self
            .perform_request(
                |c| c.post(url).json(&request),
                async |r| Ok(r.bytes_stream()),
            )
            .await?;

        Ok(Box::pin(
            stream
                .eventsource()
                .map(|event| event.context(BadPartSnafu))
                .and_then(|event| async move {
                    // SSE event 包含 event_type 和 data
                    let sse_event: InteractionSseEvent =
                        serde_json::from_str(&event.data).context(DeserializeSnafu)?;
                    Ok(sse_event.into())
                }),
        ))
    }

    /// 取得 interaction
    #[instrument(skip_all, fields(
        interaction.id = id,
    ))]
    pub(crate) async fn get_interaction(
        &self,
        id: &str,
    ) -> Result<Interaction, Error> {
        let url = self.build_url_with_suffix(&format!("interactions/{id}"))?;
        self.get_json(url).await
    }

    /// 取得 interaction（串流 resume）
    #[instrument(skip_all, fields(
        interaction.id = id,
    ))]
    pub(crate) async fn get_interaction_stream(
        &self,
        id: &str,
        last_event_id: Option<&str>,
    ) -> Result<InteractionStream, Error> {
        let mut url = self.build_url_with_suffix(&format!("interactions/{id}"))?;
        url.query_pairs_mut().append_pair("stream", "true");
        if let Some(event_id) = last_event_id {
            url.query_pairs_mut().append_pair("last_event_id", event_id);
        }
        // ... SSE stream setup
    }

    /// 取消 interaction
    #[instrument(skip_all, fields(
        interaction.id = id,
    ))]
    pub(crate) async fn cancel_interaction(
        &self,
        id: &str,
    ) -> Result<Interaction, Error> {
        let url = self.build_url_with_suffix(&format!("interactions/{id}/cancel"))?;
        self.perform_request(
            |c| c.post(url).json(&json!({})),
            async |r| r.json().await.context(DecodeResponseSnafu),
        )
        .await
    }

    /// 刪除 interaction
    #[instrument(skip_all, fields(
        interaction.id = id,
    ))]
    pub(crate) async fn delete_interaction(
        &self,
        id: &str,
    ) -> Result<(), Error> {
        let url = self.build_url_with_suffix(&format!("interactions/{id}"))?;
        self.perform_request(|c| c.delete(url), async |_r| Ok(())).await
    }
}
```

在 `Gemini` 上新增公開方法：

```rust
impl Gemini {
    /// 開始建構一個 interaction 請求
    pub fn create_interaction(&self) -> InteractionBuilder {
        InteractionBuilder::new(self.client.clone())
    }

    /// 取得 interaction handle（不發送請求）
    pub fn interaction(&self, id: &str) -> InteractionHandle {
        InteractionHandle::new(id.to_string(), self.client.clone())
    }

    /// 取得 interaction 詳細資訊
    pub async fn get_interaction(&self, id: &str) -> Result<Interaction, Error> {
        self.client.get_interaction(id).await
    }
}
```

---

## Phase 4：SSE Streaming (`src/interactions/stream.rs`)

### 4.1 Stream 型別

```rust
use std::pin::Pin;
use futures::{Stream, StreamExt, TryStreamExt};
use eventsource_stream::{Eventsource, EventStreamError};

use crate::client::Error;

/// Interaction 串流 — yield InteractionEvent
pub type InteractionStream =
    Pin<Box<dyn Stream<Item = Result<InteractionEvent, Error>> + Send>>;
```

### 4.2 SSE Event 型別

```rust
/// SSE 事件 — 從 event_type 欄位判定型別
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "event_type")]
pub enum InteractionEvent {
    /// Interaction 建立事件
    #[serde(rename = "interaction.created")]
    InteractionCreated {
        interaction: StreamInteraction,
        event_id: Option<String>,
        metadata: Option<StreamMetadata>,
    },

    /// Interaction 完成事件
    #[serde(rename = "interaction.completed")]
    InteractionCompleted {
        interaction: StreamInteraction,
        event_id: Option<String>,
        metadata: Option<StreamMetadata>,
    },

    /// Interaction 狀態更新
    #[serde(rename = "interaction.status_update")]
    InteractionStatusUpdate {
        interaction_id: String,
        status: InteractionStatus,
        event_id: Option<String>,
        metadata: Option<StreamMetadata>,
    },

    /// 錯誤事件
    #[serde(rename = "error")]
    Error {
        error: InteractionError,
        event_id: Option<String>,
        metadata: Option<StreamMetadata>,
    },

    /// Step 開始
    #[serde(rename = "step.start")]
    StepStart {
        index: usize,
        step: Step,
        event_id: Option<String>,
        metadata: Option<StepDeltaMetadata>,
    },

    /// Step 增量更新
    #[serde(rename = "step.delta")]
    StepDelta {
        index: usize,
        delta: StepDeltaData,
        event_id: Option<String>,
        metadata: Option<StepDeltaMetadata>,
    },

    /// Step 結束
    #[serde(rename = "step.stop")]
    StepStop {
        index: usize,
        #[serde(default)]
        usage: Option<InteractionUsage>,
        step_usage: Option<InteractionUsage>,
        event_id: Option<String>,
        metadata: Option<StreamMetadata>,
    },
}

/// 串流中的部分 Interaction 資源（可能省略部分欄位）
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamInteraction {
    pub id: Option<String>,
    pub object: Option<String>,
    pub model: Option<String>,
    pub agent: Option<String>,
    #[serde(default)]
    pub status: InteractionStatus,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub service_tier: Option<ServiceTier>,
    pub usage: Option<InteractionUsage>,
    #[serde(default)]
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamMetadata {
    pub total_usage: Option<InteractionUsage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StepDeltaMetadata {
    pub total_usage: Option<InteractionUsage>,
}
```

### 4.3 StepDeltaData 型別

```rust
/// Step 增量資料 — type-tagged polymorphic
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepDeltaData {
    /// 文字增量
    Text { text: String },

    /// 圖片增量
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<ImageMimeType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        resolution: Option<MediaResolution>,
    },

    /// 音訊增量
    Audio {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<AudioMimeType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sample_rate: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        channels: Option<i32>,
    },

    /// 文件增量
    Document {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<DocumentMimeType>,
    },

    /// 影片增量
    Video {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<VideoMimeType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        resolution: Option<MediaResolution>,
    },

    /// 思考摘要增量
    ThoughtSummary {
        content: Option<InteractionContent>,
    },

    /// 思考簽章增量
    ThoughtSignature {
        signature: Option<String>,
    },

    /// 文字標注增量
    TextAnnotationDelta {
        annotations: Vec<Annotation>,
    },

    /// 函式引數增量
    ArgumentsDelta {
        arguments: Option<String>,
    },

    /// 程式碼執行呼叫增量
    CodeExecutionCall {
        arguments: CodeExecutionCallArguments,
    },

    /// 程式碼執行結果增量
    CodeExecutionResult {
        result: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },

    /// URL 上下文呼叫增量
    UrlContextCall {
        arguments: UrlContextCallArguments,
    },

    /// URL 上下文結果增量
    UrlContextResult {
        result: UrlContextResult,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },

    /// Google 搜尋呼叫增量
    GoogleSearchCall {
        arguments: GoogleSearchCallArguments,
    },

    /// Google 搜尋結果增量
    GoogleSearchResult {
        result: GoogleSearchResultItem,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },

    /// Google Maps 呼叫增量
    GoogleMapsCall {
        arguments: Option<GoogleMapsCallArguments>,
    },

    /// Google Maps 結果增量
    GoogleMapsResult {
        result: Option<GoogleMapsResultItem>,
    },

    /// 檔案搜尋呼叫增量
    FileSearchCall,

    /// 檔案搜尋結果增量
    FileSearchResult {
        result: FileSearchResult,
    },

    /// MCP Server 工具呼叫增量
    McpServerToolCall {
        name: String,
        server_name: String,
        arguments: serde_json::Value,
    },

    /// MCP Server 工具結果增量
    McpServerToolResult {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        server_name: Option<String>,
        result: StepResult,
    },

    /// 函式結果增量
    FunctionResult {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
        call_id: String,
        result: StepResult,
    },
}
```

### 4.4 SSE 事件解析

SSE 回應格式為標準的 `event: <type>\ndata: <json>` 格式。每筆 data 的 JSON 包含 `event_type` 欄位作為 discriminator。

```rust
// SSE 串流解析邏輯：
// 1. 使用 eventsource_stream 解析 SSE 協議
// 2. 從 event.data JSON 中解析出 InteractionEvent（透過 event_type discriminator）
// 3. 注意：SSE event 的 event 欄位也包含型別名稱，可作為備援驗證

// 範例 SSE 事件序列：
// event: interaction.created
// data: {"event_type":"interaction.created","interaction":{"id":"v1_...","status":"in_progress"}}
//
// event: step.start
// data: {"event_type":"step.start","index":0,"step":{"type":"thought"}}
//
// event: step.delta
// data: {"event_type":"step.delta","index":0,"delta":{"type":"thought_signature","signature":"..."}}
//
// event: step.stop
// data: {"event_type":"step.stop","index":0}
//
// event: step.start
// data: {"event_type":"step.start","index":1,"step":{"type":"model_output"}}
//
// event: step.delta
// data: {"event_type":"step.delta","index":1,"delta":{"type":"text","text":"AI "}}
//
// event: step.delta
// data: {"event_type":"step.delta","index":1,"delta":{"type":"text","text":"works "}}
//
// event: step.stop
// data: {"event_type":"step.stop","index":1}
//
// event: interaction.completed
// data: {"event_type":"interaction.completed","interaction":{"id":"v1_...","status":"completed","usage":{...}}}
```

---

## Phase 5：InteractionHandle (`src/interactions/handle.rs`)

```rust
use std::{sync::Arc, time::Duration};
use tracing::instrument;
use tokio::time::sleep;

use crate::client::{Error, GeminiClient};
use crate::interactions::model::*;

/// Interaction 的 handle，可用於 get / cancel / delete / poll
#[derive(Clone)]
pub struct InteractionHandle {
    id: String,
    client: Arc<GeminiClient>,
}

impl InteractionHandle {
    pub(crate) fn new(id: String, client: Arc<GeminiClient>) -> Self {
        Self { id, client }
    }

    /// 取得 interaction ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 取得 interaction 完整資訊
    #[instrument(skip(self))]
    pub async fn get(&self) -> Result<Interaction, Error> {
        self.client.get_interaction(&self.id).await
    }

    /// 取得 interaction（串流模式，可從 last_event_id resume）
    #[instrument(skip(self), fields(last_event_id = last_event_id.as_deref().unwrap_or(""))]
    pub async fn get_stream(
        &self,
        last_event_id: Option<&str>,
    ) -> Result<InteractionStream, Error> {
        self.client.get_interaction_stream(&self.id, last_event_id).await
    }

    /// 取消 interaction（僅適用於背景執行中）
    #[instrument(skip(self))]
    pub async fn cancel(&self) -> Result<Interaction, Error> {
        self.client.cancel_interaction(&self.id).await
    }

    /// 刪除 interaction
    #[instrument(skip(self))]
    pub async fn delete(&self) -> Result<(), Error> {
        self.client.delete_interaction(&self.id).await
    }

    /// 輪詢直到 interaction 完成
    ///
    /// 持續呼叫 `get()` 直到狀態變為 completed / failed / cancelled。
    /// 適用於背景執行的 interaction。
    #[instrument(skip(self), fields(poll.interval = ?interval))]
    pub async fn poll_until_completed(
        &self,
        interval: Duration,
    ) -> Result<Interaction, Error> {
        loop {
            let interaction = self.get().await?;

            match interaction.status {
                InteractionStatus::Completed
                | InteractionStatus::Failed
                | InteractionStatus::Cancelled
                | InteractionStatus::Incomplete
                | InteractionStatus::BudgetExceeded => {
                    return Ok(interaction);
                }
                InteractionStatus::InProgress | InteractionStatus::RequiresAction => {
                    sleep(interval).await;
                }
            }
        }
    }

    /// 用於多輪對話 — 取得此 interaction 的 ID 作為下一輪的 previous_interaction_id
    pub fn as_previous_interaction_id(&self) -> &str {
        &self.id
    }
}
```

---

## Phase 6：功能實作

### 6.1 Function Calling 迴圈

#### Stateful 模式（推薦，使用 `previous_interaction_id`）

```rust
/// 自動函式呼叫迴圈（stateful 模式）
///
/// 持續建立 interaction 直到模型不再要求函式呼叫。
/// 使用 `previous_interaction_id` 維護 server-side 狀態。
///
/// # 參數
/// * `tools` - 可用的函式工具宣告
/// * `input` - 初始使用者輸入
/// * `handler` - 函式執行回呼，接收 (name, arguments) 回傳 result value
pub async fn run_function_loop<F, Fut>(
    &self,
    tools: Vec<InteractionTool>,
    input: impl Into<String>,
    handler: F,
) -> Result<Interaction, Error>
where
    F: Fn(String, serde_json::Value) -> Fut,
    Fut: std::future::Future<Output = serde_json::Value>,
{
    let mut previous_id: Option<String> = None;
    let mut user_input: InteractionInput = InteractionInput::Text(input.into());
    let mut interaction: Interaction;

    loop {
        let mut builder = self.create_interaction()
            .with_tools(tools.clone());

        if let Some(prev_id) = &previous_id {
            builder = builder.with_previous_interaction(prev_id.clone());
        }

        builder = match user_input {
            InteractionInput::Text(t) => builder.with_text(t),
            InteractionInput::ContentArray(c) => builder.with_content_input(c),
            InteractionInput::StepArray(s) => builder.with_step_input(s),
            _ => builder,
        };

        interaction = builder.execute().await?;

        // 收集 function_call steps
        let function_results: Vec<Step> = interaction.steps
            .iter()
            .filter_map(|step| {
                if let Step::FunctionCall { name, arguments, id } = step {
                    let result = handler(name.clone(), arguments.clone());
                    // Note: handler 是 async，這裡簡化了；實際需要 async handler
                    Some(Step::FunctionResult {
                        name: Some(name.clone()),
                        call_id: id.clone(),
                        result: StepResult::String(serde_json::to_string(&result).unwrap()),
                        is_error: None,
                    })
                } else {
                    None
                }
            })
            .collect();

        if function_results.is_empty() {
            break;
        }

        previous_id = interaction.id.clone();
        user_input = InteractionInput::StepArray(function_results);
    }

    Ok(interaction)
}
```

#### Stateless 模式（使用 `store=false` + 手動 history）

```rust
/// 自動函式呼叫迴圈（stateless 模式）
///
/// 客戶端管理完整對話歷史。每次請求都帶上完整 step 歷史。
/// 需要保留所有模型生成的 step（包含 thought 和 function_call）。
pub async fn run_function_loop_stateless<F, Fut>(
    &self,
    tools: Vec<InteractionTool>,
    input: impl Into<String>,
    handler: F,
) -> Result<Interaction, Error>
where
    F: Fn(String, serde_json::Value) -> Fut,
    Fut: std::future::Future<Output = serde_json::Value>,
{
    let mut history: Vec<Step> = vec![
        Step::UserInput {
            content: vec![InteractionContent::Text { text: input.into(), annotations: vec![] }],
        }
    ];

    let mut interaction: Interaction;

    loop {
        interaction = self.create_interaction()
            .with_store(false)
            .with_tools(tools.clone())
            .with_step_input(history.clone())
            .execute()
            .await?;

        // 將模型生成的 steps 加入歷史
        let mut function_results = Vec::new();
        for step in &interaction.steps {
            history.push(step.clone());

            if let Step::FunctionCall { name, arguments, id } = step {
                let result = handler(name.clone(), arguments.clone()).await;
                let fn_result = Step::FunctionResult {
                    name: Some(name.clone()),
                    call_id: id.clone(),
                    result: StepResult::String(serde_json::to_string(&result).unwrap()),
                    is_error: None,
                };
                function_results.push(fn_result.clone());
                history.push(fn_result);
            }
        }

        if function_results.is_empty() {
            break;
        }
    }

    Ok(interaction)
}
```

### 6.2 Background Execution

```rust
// 啟動背景任務
let interaction = gemini.create_interaction()
    .with_model("gemini-3.5-flash")
    .with_input("Write a detailed analysis...")
    .with_background(true)
    .execute()
    .await?;
// → Interaction { status: InProgress, id: "v1_abc123" }

// 輪詢直到完成
let handle = gemini.interaction(interaction.id.as_ref().unwrap());
let result = handle.poll_until_completed(Duration::from_secs(5)).await?;
// → Interaction { status: Completed, steps: [...] }

// 或手動輪詢
loop {
    let result = handle.get().await?;
    match result.status {
        InteractionStatus::Completed => { /* 處理結果 */ break; }
        InteractionStatus::Failed => { /* 處理錯誤 */ break; }
        _ => tokio::time::sleep(Duration::from_secs(5)).await,
    }
}
```

### 6.3 Managed Agents

#### Deep Research

```rust
let interaction = gemini.create_interaction()
    .with_agent("deep-research-preview-04-2026")
    .with_input("Research the current state of quantum computing")
    .with_agent_config(AgentConfig::DeepResearch {
        thinking_summaries: Some(ThinkingSummaries::Auto),
        visualization: Some(Visualization::Auto),
        collaborative_planning: Some(false),
        enable_bigquery_tool: Some(false),
    })
    .with_background(true)  // Deep Research 通常需要較長時間
    .execute()
    .await?;

let handle = gemini.interaction(interaction.id.as_ref().unwrap());
let result = handle.poll_until_completed(Duration::from_secs(10)).await?;
println!("{}", result.output_text());
```

#### Antigravity Agent

```rust
let interaction = gemini.create_interaction()
    .with_agent("antigravity-preview-05-2026")
    .with_input("Write a Python script that generates Fibonacci numbers")
    .with_environment(EnvironmentConfig::Remote {
        sources: vec![
            EnvironmentSource::Inline {
                target: Some(".agents/AGENTS.md".to_string()),
                content: Some("You are a helpful coding assistant.".to_string()),
                encoding: None,
            },
        ],
        environment_id: None,
        network: None,
    })
    .execute()
    .await?;
println!("Environment ID: {:?}", interaction.environment_id);
println!("{}", interaction.output_text());

// 重用環境
let interaction2 = gemini.create_interaction()
    .with_agent("antigravity-preview-05-2026")
    .with_input("Now modify the script to also save primes")
    .with_environment(interaction.environment_id.clone().unwrap())  // 重用
    .with_previous_interaction(interaction.id.as_ref().unwrap())
    .execute()
    .await?;
```

### 6.4 All Tools 對照

每個 tool 對應 `InteractionTool` variant + 對應的 Step/StepDelta 型別：

| Tool | InteractionTool variant | Call Step | Result Step | Delta variants |
|------|------------------------|-----------|-------------|----------------|
| 自訂函式 | `Function` | `FunctionCall` | `FunctionResult` | `ArgumentsDelta`, `FunctionResult` |
| 程式碼執行 | `CodeExecution` | `CodeExecutionCall` | `CodeExecutionResult` | `CodeExecutionCall`, `CodeExecutionResult` |
| Google 搜尋 | `GoogleSearch` | `GoogleSearchCall` | `GoogleSearchResult` | `GoogleSearchCall`, `GoogleSearchResult` |
| URL 上下文 | `UrlContext` | `UrlContextCall` | `UrlContextResult` | `UrlContextCall`, `UrlContextResult` |
| Google Maps | `GoogleMaps` | `GoogleMapsCall` | `GoogleMapsResult` | `GoogleMapsCall`, `GoogleMapsResult` |
| 檔案搜尋 | `FileSearch` | `FileSearchCall` | `FileSearchResult` | `FileSearchCall`, `FileSearchResult` |
| MCP Server | `McpServer` | `McpServerToolCall` | `McpServerToolResult` | `McpServerToolCall`, `McpServerToolResult` |
| 電腦操作 | `ComputerUse` | *(computer use steps)* | *(computer use steps)* | *(computer use deltas)* |
| 檔案擷取 | `Retrieval` | *(retrieval steps)* | *(retrieval steps)* | *(retrieval deltas)* |

### 6.5 Structured Output

```rust
// JSON Schema structured output
let interaction = gemini.create_interaction()
    .with_model("gemini-3.5-flash")
    .with_input("Give me a recipe for banana bread")
    .with_response_format(ResponseFormat::Text {
        mime_type: Some(TextMimeType::ApplicationJson),
        schema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "recipe_name": { "type": "string" },
                "ingredients": {
                    "type": "array",
                    "items": { "type": "string" }
                },
                "prep_time_minutes": { "type": "integer" }
            },
            "required": ["recipe_name", "ingredients"]
        })),
    })
    .execute()
    .await?;

let recipe: serde_json::Value = serde_json::from_str(&interaction.output_text())?;
```

### 6.6 多輪對話

#### Stateful（推薦）

```rust
// 第一輪
let interaction1 = gemini.create_interaction()
    .with_model("gemini-3.5-flash")
    .with_input("I have 2 dogs in my house.")
    .execute()
    .await?;
println!("Response 1: {}", interaction1.output_text());

// 第二輪 — 使用 previous_interaction_id
let interaction2 = gemini.create_interaction()
    .with_model("gemini-3.5-flash")
    .with_input("How many paws are in my house?")
    .with_previous_interaction(interaction1.id.as_ref().unwrap())
    .execute()
    .await?;
println!("Response 2: {}", interaction2.output_text());
```

#### Stateless

```rust
let mut history = vec![
    Step::UserInput {
        content: vec![InteractionContent::Text {
            text: "I have 2 dogs in my house.".to_string(),
            annotations: vec![],
        }],
    }
];

// 第一輪
let interaction1 = gemini.create_interaction()
    .with_model("gemini-3.5-flash")
    .with_store(false)
    .with_step_input(history.clone())
    .execute()
    .await?;

// 將模型 steps 加入歷史
for step in &interaction1.steps {
    history.push(step.clone());
}

// 第二輪
history.push(Step::UserInput {
    content: vec![InteractionContent::Text {
        text: "How many paws are in my house?".to_string(),
        annotations: vec![],
    }],
});

let interaction2 = gemini.create_interaction()
    .with_model("gemini-3.5-flash")
    .with_store(false)
    .with_step_input(history)
    .execute()
    .await?;
```

---

## Phase 7：便利方法 (`impl Interaction`)

```rust
impl Interaction {
    /// 取得串接的最終文字輸出（模擬 SDK 的 output_text 屬性）
    ///
    /// 從最後一個 model_output step 中提取所有 text content 並串接。
    pub fn output_text(&self) -> String {
        self.steps
            .iter()
            .rev()
            .find(|s| matches!(s, Step::ModelOutput { .. }))
            .and_then(|s| {
                if let Step::ModelOutput { content, .. } = s {
                    Some(
                        content
                            .iter()
                            .filter_map(|c| {
                                if let InteractionContent::Text { text, .. } = c {
                                    Some(text.clone())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(""),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    /// 取得所有 function_call steps
    pub fn function_calls(&self) -> Vec<&Step> {
        self.steps
            .iter()
            .filter(|s| matches!(s, Step::FunctionCall { .. }))
            .collect()
    }

    /// 取得所有 model_output steps
    pub fn model_outputs(&self) -> Vec<&Step> {
        self.steps
            .iter()
            .filter(|s| matches!(s, Step::ModelOutput { .. }))
            .collect()
    }

    /// 取得所有 thought steps
    pub fn thoughts(&self) -> Vec<&Step> {
        self.steps
            .iter()
            .filter(|s| matches!(s, Step::Thought { .. }))
            .collect()
    }

    /// 是否需要使用者執行函式
    pub fn requires_action(&self) -> bool {
        self.status == InteractionStatus::RequiresAction
    }

    /// 是否已完成
    pub fn is_completed(&self) -> bool {
        self.status == InteractionStatus::Completed
    }

    /// 取得輸出圖片（最後一張模型生成的圖片）
    pub fn output_image(&self) -> Option<&InteractionContent> {
        self.steps
            .iter()
            .rev()
            .find_map(|s| {
                if let Step::ModelOutput { content, .. } = s {
                    content.iter().find(|c| {
                        matches!(c, InteractionContent::Image { .. })
                    })
                } else {
                    None
                }
            })
    }

    /// 取得輸出音訊
    pub fn output_audio(&self) -> Option<&InteractionContent> {
        self.steps
            .iter()
            .rev()
            .find_map(|s| {
                if let Step::ModelOutput { content, .. } = s {
                    content.iter().find(|c| {
                        matches!(c, InteractionContent::Audio { .. })
                    })
                } else {
                    None
                }
            })
    }

    /// 取得輸出影片
    pub fn output_video(&self) -> Option<&InteractionContent> {
        self.steps
            .iter()
            .rev()
            .find_map(|s| {
                if let Step::ModelOutput { content, .. } = s {
                    content.iter().find(|c| {
                        matches!(c, InteractionContent::Video { .. })
                    })
                } else {
                    None
                }
            })
    }

    /// 取得所有引用標注（URL / File / Place citations）
    pub fn citations(&self) -> Vec<&Annotation> {
        self.steps
            .iter()
            .filter_map(|s| {
                if let Step::ModelOutput { content, .. } = s {
                    Some(content.iter().flat_map(|c| {
                        if let InteractionContent::Text { annotations, .. } = c {
                            annotations.iter().collect::<Vec<_>>()
                        } else {
                            vec![]
                        }
                    }))
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }

    /// 取得 total_tokens
    pub fn total_tokens(&self) -> Option<i64> {
        self.usage.as_ref()?.total_tokens
    }

    /// 取得 interaction ID
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// 轉為 handle（用於後續 get/cancel/delete/poll）
    pub fn into_handle(self, client: &crate::client::Gemini) -> Option<InteractionHandle> {
        self.id.as_ref().map(|id| {
            crate::interactions::InteractionHandle::new(id.clone(), client.client.clone())
        })
    }
}
```

---

## Phase 8：範例全部重寫

### 範例分組對照

| 分類 | 舊範例 | 新範例 | 備註 |
|------|--------|--------|------|
| **基礎文字** | `basic_generation.rs`, `simple.rs` | `interaction_basic.rs` | 最簡單的 interaction |
| **串流** | `basic_streaming.rs`, `streaming.rs` | `interaction_streaming.rs` | SSE step 事件處理 |
| **多輪對話** | *(分散在各範例)* | `interaction_multi_turn.rs` | Stateful + Stateless |
| **多模態輸入** | `image_*.rs`, `mp4_describe.rs`, `file_input.rs`, `blob.rs` | `interaction_multimodal.rs` | 圖片/音訊/影片/文件 |
| **圖片生成** | `image_generation.rs`, `simple_image_generation.rs`, `image_editing.rs` | `interaction_image_gen.rs` | |
| **TTS** | `multi_speaker_tts.rs`, `simple_speech_generation.rs` | `interaction_tts.rs` | |
| **Structured output** | `json_schema.rs`, `structured_response.rs` | `interaction_structured.rs` | response_format |
| **Function calling** | `complex_function.rs`, `tools.rs` | `interaction_function_calling.rs` | Stateful loop |
| **Thinking** | `thinking_*.rs`, `gemini_3_all_thinking_levels.rs` | `interaction_thinking.rs` | thinking_level |
| **Google 搜尋** | `google_search.rs`, `curl_google_search.rs`, `google_search_with_functions.rs` | `interaction_google_search.rs` | 含 citations |
| **Google Maps** | `google_maps_grounding.rs`, `simple_maps_example.rs`, `advanced_maps_configuration.rs` | `interaction_google_maps.rs` | |
| **程式碼執行** | `gemini_3_code_execution.rs` | `interaction_code_execution.rs` | |
| **URL 上下文** | `url_context.rs` | `interaction_url_context.rs` | |
| **檔案搜尋** | `file_search_basic.rs`, `file_search_import.rs`, `file_search_metadata.rs` | `interaction_file_search.rs` | |
| **背景執行** | *(不存在)* | `interaction_background.rs` | 新增 |
| **Deep Research** | *(不存在)* | `interaction_deep_research.rs` | 新增 |
| **Antigravity** | *(不存在)* | `interaction_antigravity.rs` | 新增 |
| **進階** | `advanced.rs`, `curl_equivalent.rs`, `generation_config.rs` | `interaction_advanced.rs` | |
| **錯誤處理** | `error_handling.rs` | `interaction_error_handling.rs` | |
| **自訂模型/URL** | `custom_models.rs`, `custom_base_url.rs`, `http_client_builder.rs` | `interaction_custom_client.rs` | |
| **Tracing** | `tracing_telemetry.rs` | `interaction_tracing.rs` | |

### 保留不變的範例（Interactions API 不支援）

| 範例 | 原因 |
|------|------|
| `batch_*.rs` (5個) | Interactions API 不支援 Batch API |
| `cache_basic.rs` | Interactions API 不支援 explicit caching |
| `safety_settings.rs` | Interactions API 不支援 custom safety settings |
| `embedding.rs`, `batch_embedding.rs` | Embedding 不在 Interactions API 範疇 |
| `files_*.rs` (3個) | Files API 獨立於 Interactions |
| `count_tokens.rs` | Interactions API 無此端點 |
| `thought_signature_*.rs`, `text_thought_signature_example.rs` | Thought signature 仍可用但屬 generateContent 特性 |
| `gemini_3_thinking_and_media.rs`, `gemini_pro_example.rs` | 可選擇保留或遷移 |
| `test_api.rs` | 測試用 |

---

## Phase 9：棄用標記與文件

### 9.1 棄用標記

在以下型別和方法加上 `#[deprecated]`：

```rust
// src/generation/builder.rs
#[deprecated(
    since = "0.x.0",
    note = "Use crate::interactions::InteractionBuilder instead. \
            See migration guide: interactions-api/migration-plan.md"
)]
pub struct ContentBuilder { ... }

// src/generation/model.rs
#[deprecated(note = "Use crate::interactions::model::Interaction instead")]
pub struct GenerationResponse { ... }

#[deprecated(note = "Use crate::interactions::model::CreateInteractionRequest instead")]
pub struct GenerateContentRequest { ... }

// src/client.rs — Gemini 方法
#[deprecated(note = "Use Gemini::create_interaction() instead")]
pub fn generate_content(&self) -> ContentBuilder { ... }
```

### 9.2 文件更新

1. **`README.md`** — 新增 Interactions API 快速開始段落
2. **`MIGRATION.md`** — 新增遷移指南（從 generateContent 到 Interactions API）
3. **`AGENTS.md`** — 新增 interactions 模組的 logging 規範
4. **`lib.rs`** — 更新模組文件，加入 interactions 模組說明
5. **`prelude.rs`** — 新增 interactions 常用型別 re-export

### 9.3 lib.rs 更新

```rust
/// Interactions API — Gemini 的最新互動介面
///
/// Interactions API 是使用 Gemini 模型和 agents 的最簡單且最佳方式。
/// 它提供統一的介面適用於所有使用場景，包括單輪文字生成、多模態理解、
/// 結構化輸出、工具編排、以及 agentic 工作流程。
///
/// 主要優勢：
/// - 模型和 agents 的統一介面
/// - Server-side 狀態管理（`previous_interaction_id`）
/// - 可觀察的執行步驟
/// - 背景執行
/// - 更高的快取命中率
pub mod interactions;

// re-exports
pub use interactions::{
    builder::InteractionBuilder,
    handle::InteractionHandle,
    model::*,
    stream::{InteractionStream, InteractionEvent, StepDeltaData},
};
```

### 9.4 prelude.rs 更新

```rust
pub use crate::interactions::{
    InteractionBuilder, InteractionHandle, Interaction, InteractionStatus,
    InteractionInput, InteractionContent, InteractionTool, Step, ResponseFormat,
    InteractionGenerationConfig, InteractionUsage, InteractionThinkingLevel,
    AgentConfig, InteractionStream, InteractionEvent, StepDeltaData,
    InteractionTool as Tool,  // 便利別名
};
```

---

## 實作順序（建議分 7 個 PR/commit）

| 步驟 | 內容 | 預估程式碼量 | 相依 |
|------|------|------------|------|
| **1** | Phase 1: 核心型別系統 (`interactions/model.rs` + `serde.rs`) | ~1000 行 | 無 |
| **2** | Phase 3: Client 擴充 (`client.rs` 新增方法) | ~200 行 | Step 1 |
| **3** | Phase 2: InteractionBuilder (`interactions/builder.rs`) | ~500 行 | Step 1, 2 |
| **4** | Phase 4: SSE Streaming (`interactions/stream.rs`) | ~400 行 | Step 1, 2 |
| **5** | Phase 5+6+7: Handle + Function calling loop + Background + Agents + 便利方法 | ~500 行 | Step 1-4 |
| **6** | Phase 8: 範例全部重寫 | ~2000 行 (50 檔案) | Step 1-5 |
| **7** | Phase 9: 棄用標記 + 文件 | 散佈各檔案 | Step 1-6 |

---

## 型別對照表

### 請求型別

| generateContent | Interactions API |
|-----------------|-----------------|
| `GenerateContentRequest` | `CreateInteractionRequest` |
| `Content (parts + role)` | `InteractionInput` (string / Content[] / Step[]) |
| `GenerationConfig` | `InteractionGenerationConfig` |
| `ContentBuilder` | `InteractionBuilder` |
| `Tool (FunctionDeclaration, etc.)` | `InteractionTool` |
| `ToolConfig / FunctionCallingConfig` | `ToolChoiceConfig` |
| `ThinkingConfig (budget + level)` | `InteractionThinkingLevel` (level only) |
| `response_mime_type + response_schema` | `ResponseFormat::Text { mime_type, schema }` |
| `response_modalities` | `ResponseFormat` (Text/Image/Audio/Video) |
| `system_instruction: Content` | `system_instruction: String` |
| `cached_content` | `cached_content` (保留) |
| `safety_settings` | *(不支援)* |
| *(不存在)* | `previous_interaction_id` |
| *(不存在)* | `store` |
| *(不存在)* | `background` |
| *(不存在)* | `agent` |
| *(不存在)* | `agent_config` |
| *(不存在)* | `environment` |
| *(不存在)* | `service_tier` |
| *(不存在)* | `webhook_config` |

### 回應型別

| generateContent | Interactions API |
|-----------------|-----------------|
| `GenerationResponse` | `Interaction` |
| `candidates: Vec<Candidate>` | `steps: Vec<Step>` (filter `model_output`) |
| `Candidate.content.parts` | `ModelOutputStep.content` |
| `Candidate.finish_reason` | `Interaction.status` |
| `UsageMetadata` | `InteractionUsage` |
| `PromptFeedback` | *(不存在 — 無 safety)* |
| `GroundingMetadata` | `GoogleSearchResultStep` / `GoogleMapsResultStep` |
| `GenerationStream` | `InteractionStream` |
| `GenerationResponse (stream chunk)` | `InteractionEvent` (step.start/delta/stop) |
| `response.text()` | `interaction.output_text()` |
| `response.function_calls()` | `interaction.function_calls()` |

### Part / Step 對照

| generateContent `Part` | Interactions API `Step` |
|------------------------|------------------------|
| `Part::Text { text, thought: false }` | `Step::ModelOutput { content: [Text] }` |
| `Part::Text { text, thought: true }` | `Step::Thought { summary: [Text] }` |
| `Part::Text { thought_signature }` | `Step::Thought { signature }` |
| `Part::InlineData { inline_data }` | `Step::ModelOutput { content: [Image/Audio/Video] }` |
| `Part::FunctionCall` | `Step::FunctionCall` |
| `Part::FunctionResponse` | `Step::FunctionResult` |
| `Part::FileData` | `InteractionContent::Image/Audio/Video { uri }` |
| `Part::ExecutableCode` | `Step::CodeExecutionCall` |
| `Part::CodeExecutionResult` | `Step::CodeExecutionResult` |
| `Part::ToolCall` | `Step::GoogleSearchCall` / `GoogleMapsCall` / etc. |
| `Part::ToolResponse` | `Step::GoogleSearchResult` / `GoogleMapsResult` / etc. |

---

## 端點對照表

| 功能 | generateContent 端點 | Interactions API 端點 |
|------|---------------------|----------------------|
| 生成內容 | `POST models/{model}:generateContent` | `POST interactions` |
| 串流生成 | `POST models/{model}:streamGenerateContent?alt=sse` | `POST interactions?alt=sse` |
| 計數 token | `POST models/{model}:countTokens` | *(無)* |
| 取得結果 | *(不適用 — 同步)* | `GET interactions/{id}` |
| 取得串流（resume） | *(不適用)* | `GET interactions/{id}?stream=true&last_event_id=...` |
| 取消 | *(不適用)* | `POST interactions/{id}/cancel` |
| 刪除 | *(不適用)* | `DELETE interactions/{id}` |
| 嵌入 | `POST models/{model}:embedContent` | *(不適用 — 獨立 API)* |
| 批次 | `POST batches` | *(不支援)* |
| 快取 | `POST cachedContents` | *(不支援 — 使用 previous_interaction_id)* |

---

## Logging 規範

遵循 `AGENTS.md` 中的結構化 logging 規範，Interactions API 相關的 instrumentation：

```rust
// 建構 Interaction 請求
#[instrument(skip_all, fields(
    model,
    agent,
    tools.count,
    system.instruction.present,
    background,
    previous.interaction.present,
    status.code,
    usage.total_tokens,
    usage.input.tokens,
    usage.output.tokens,
    usage.thought.tokens,
    usage.cached.tokens,
))]

// Interaction get
#[instrument(skip_all, fields(
    interaction.id,
    status.code,
    usage.total_tokens,
))]

// Interaction cancel
#[instrument(skip_all, fields(
    interaction.id,
    status.code,
))]

// 串流事件
#[instrument(skip_all, fields(
    model,
    agent,
    tools.count,
))]
```
