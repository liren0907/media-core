# PLAN — `/composer` Phase 0：後端骨架

## 這份文檔是什麼（2026-08-07）

**Phase 0 的實作規格**：manifest ＋ registry ＋ validate ＋ runner ＋ command。
**這階段前端一行都不改**，做完就能用手寫的 JSON 圖跑出真實結果。

前置閱讀：`REFERENCE_composer_method_manifest.md`（埠宣告）、
`PLAN_composer_graph_editor.md`（主計畫 ＋ 五個決策）。

**Phase 0 存在的真正理由**：在花 1500 行畫布之前，先確認「圖跑得動、而且圖有內容」。
如果做完發現圖太扁（見 manifest §4），這裡就是喊停的地方，沉沒成本只有 ~900 行——
而且那份 manifest ＋ registry 對重構 `commands/pipeline.rs`（839 行寫死的組裝）本身
就有價值。

---

## 1. 模組地圖

```
src-tauri/src/graph/
├── mod.rs           出口 ＋ 對外 re-export
├── types.rs         wire 格式：GraphRecipe / GraphNode / GraphEdge / NodeParams
├── manifest.rs      方法宣告表（11 顆 step ＋ 1 顆自訂 sink）
├── registry.rs      (node_ref, params, &MediaContext) → Box<dyn PipelineStep<MediaContext>>
├── validate.rs      topo_order ＋ G1–G7
├── runner.rs        逐顆建構 ＋ 執行 ＋ 進度事件
└── steps/
    └── save_report.rs   app 側自訂 step（見 manifest §4）

src-tauri/src/commands/graph.rs   Tauri command 層
```

修改的既有檔案（純加法）：`src-tauri/src/commands/mod.rs`（`pub mod graph;`）、
`src-tauri/src/main.rs`（`generate_handler!` 加 3 支）。

**`src-tauri/crates/media-core/` 零 diff。**

---

## 2. wire 格式（`types.rs`）

前後端唯一契約。snake_case 存檔、camelCase 上線（serde `rename_all`）。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphRecipe {
    pub label: String,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    pub node_id: String,        // 'n0', 'n1', … 前端 mint
    pub kind: String,           // 'source' | 'method' | 'sink'  ← 開放字串，不用 enum
    pub node_ref: String,       // 'file' / 'extract_frames_to_disk' / 'save_report'
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<NodeParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge { pub from: String, pub to: String }
```

### `NodeParams` — 弱型別 map，不是具名欄位

參考文檔用了七個具名 `Option<T>` 欄位（`window_lines`、`top_k`…），因為他們的參數
種類固定。**我們不行**——11 顆 step 的參數種類差異太大（bool / f64 / path / 五種
不同的 enum），具名欄位會爆炸成 25 個以上。

```rust
pub type NodeParams = std::collections::BTreeMap<String, serde_json::Value>;
```

- `BTreeMap` 不是 `HashMap`：**序列化順序穩定**，前端的 `dirty` 比對才可靠
  （參考文檔 §7a 的 `baseline` vs `current` 逐字串比對）
- **缺 key ＝ 沒設 ＝ 用 manifest 預設**。空 map 整個不上線（`skip_serializing_if`）
- 型別檢查交給 `validate.rs` 對照 `manifest.rs` 的 `ParamSpec`——存進來時驗一次，
  registry 取值時再 `expect_*` 一次

### 來源節點的 `node_ref`

`kind: 'source'` 的三個變體：

| `node_ref` | params | → `MediaContext` |
|---|---|---|
| `file` | `path: string`（必填） | `MediaContext::from_file(PathBuf::from(path))` |
| `stream` | `url: string`（必填） | `MediaContext::from_stream(url)` |
| `camera` | `cameraId: i32`（預設 0） | `MediaContext::from_camera(id)` |

**一張圖恰好一顆 source 節點**（G4）。多來源＝多張圖分開跑，v1 不做。

---

## 3. `manifest.rs`

靜態表，`OnceLock<Vec<MethodMeta>>`。形狀見 `REFERENCE_composer_method_manifest.md` §6。

```rust
pub struct MethodMeta {
    pub id: &'static str,
    pub label: &'static str,
    pub group: Option<&'static str>,
    pub category: Category,
    pub reads: &'static [SlotRef],
    pub writes: &'static [SlotWrite],
    pub params: &'static [ParamSpec],
    pub requires_ffmpeg: bool,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Slot {
    Source, Metadata, Analysis, ExtractedFrames,
    HlsResult, AnnotationResult, VideoProcessResult, ProcessResult,
}

pub struct SlotRef  { pub slot: Slot, pub accepts: &'static [SourceKind] }  // accepts 只對 Source 有意義
pub struct SlotWrite { pub slot: Slot, pub mode: WriteMode }                 // Replace | Merge | Append

pub struct ParamSpec {
    pub key: &'static str,
    pub kind: ParamKind,       // Bool | Int | Float | Str | Path | Dir | Enum
    pub label: &'static str,
    pub default: Option<DefaultValue>,
    pub required: bool,
    pub options: &'static [(&'static str, &'static str)],   // Enum 專用 (value, label)
    pub min: Option<f64>,
    pub max: Option<f64>,
    /// 留空時可以從哪個槽的哪個欄位取——有值代表這是一個「真的輸入埠」
    pub fallback_from: Option<FallbackSource>,
}

pub enum FallbackSource {
    VideoProcessOutputDir,      // video_process_result.output_dir
    ContextSourcePath,          // source（File 變體）的路徑
}
```

**`fallback_from` 是圖能不能串起來的關鍵**（見 manifest §4）。有 `fallback_from` 的
參數，manifest 的 `reads` 要同時把對應的槽列進去——否則 picker 不會把上游上架。

初版填 **11 顆 media-core step ＋ `save_report`**。逐顆的內容照
`REFERENCE_composer_method_manifest.md` §2 抄。

---

## 4. `registry.rs` — 字串 → step 工廠

```rust
pub fn build_step(
    node_ref: &str,
    params: &NodeParams,
    ctx: &MediaContext,          // ← 關鍵：建構時就能讀黑板，解析 fallback
) -> Result<Box<dyn PipelineStep<MediaContext>>, String>
```

一顆 step 一個 match arm，11＋1 個 arm。範例：

```rust
"extract_frames_to_disk" => {
    let out = p.dir("outputDir")?;                       // required
    let mut s = ExtractFramesToDisk::new(out);
    if let Some(v) = p.int("interval")?      { s = s.interval(v as usize); }
    if let Some(v) = p.enum_("extractionMode")? {
        use media_core::video_process::ExtractionMode as EM;   // ⚠️ 全路徑，見 manifest §5
        s = s.mode(match v {
            "opencv_sequential" => EM::OpenCVSequential,
            "ffmpeg"            => EM::FFmpeg,
            "ffmpeg_interval"   => EM::FFmpegInterval,
            "parallel"          => EM::Parallel,
            _                   => EM::OpenCVInterval,
        });
    }
    if let Some(v) = p.enum_("saveMode")? { … }
    Ok(Box::new(s))
}

"group_similar_images" => {
    // 留空 ⇒ 吃上游寫進黑板的 output_dir
    let input = p.dir_opt("inputDir")?
        .or_else(|| ctx.video_process_result.as_ref().map(|r| PathBuf::from(&r.output_dir)))
        .ok_or("inputDir 未填，且上游沒有可用的輸出目錄")?;
    let output = p.dir("outputDir")?;
    let mut s = GroupSimilarImages::new(input, output);   // ⚠️ 兩參數同一個泛型 P，統一 PathBuf
    …
}
```

配套一支小 helper（`ParamsExt`）把 `serde_json::Value` 取值收斂成
`bool_/int/float/str/path/dir/enum_` ＋ `*_opt` 兩組，錯誤訊息帶 key 名，**中文原句**
（跟 repo 既有 `Result<T, String>` 慣例一致，前端 `Error.message` 原樣顯示）。

---

## 5. `runner.rs` — 為什麼不能直接用 `Pipeline::execute`

`media-core` 的 `Pipeline<T>` 是**先收集完 `Vec<Box<dyn Step>>` 才 `execute`**：

```rust
pub fn execute(&self, mut context: T) -> Result<T, PipelineError> {
    for step in &self.steps { step.execute(&mut context)?; }   // steps 早就建好了
    Ok(context)
}
```

但 §4 的 `group_similar_images` 需要在**建構時**讀 `ctx.video_process_result`——而那個值
要等上游 `extract_frames_to_disk` **執行完**才有。先建全部再跑，拿到的永遠是 `None`。

⇒ Phase 0 在 app 側寫一個小 runner，把「建構」推遲到每顆自己要跑的前一刻：

```rust
pub fn run_graph(
    app: &tauri::AppHandle,
    recipe: &GraphRecipe,
    run_id: &str,
    cancel: &AtomicBool,
) -> Result<MediaContext, String> {
    let order = validate::topo_order(recipe)?;                  // 驗證已在呼叫端跑過
    let source = recipe.nodes.iter().find(|n| n.kind == "source").unwrap();
    let mut ctx = build_context(source)?;                       // from_file / from_stream / from_camera

    let total = order.len();
    for (i, node_id) in order.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) { return Err("已取消".into()); }
        let node = recipe.node(node_id).unwrap();
        if node.kind == "source" { continue; }

        emit_progress(app, run_id, i, total, &node.node_ref, Phase::Start);

        let params = node.params.clone().unwrap_or_default();
        let step = registry::build_step(&node.node_ref, &params, &ctx)?;   // ← 此刻才建，讀得到上游
        step.execute(&mut ctx).map_err(|e| format!("{} 失敗：{}", node.node_ref, e))?;

        emit_progress(app, run_id, i, total, &node.node_ref, Phase::Done);
    }
    Ok(ctx)
}
```

**約 80–120 行。** `media-core` 的 `PipelineStep` / `MediaContext` 原樣使用，
`Pipeline<T>` 這個 struct 我們單純不用（既有的 14 支 command 繼續用它，零 diff）。

### 執行語意的三點記帳

1. **拓樸線性化不是真並行**。畫面上的分岔在執行時是依序跑。資料本來就在黑板上不在
   線上，所以結果等價；只是不會有速度紅利。真並行要 `Send` 的 step ＋ 分裂 context，
   **v1 不做**。
2. **`topo_order` 用 Kahn 演算法，且對同層節點照 `edges` 出現序排**——同一張圖每次
   跑的順序必須一樣（決定性），否則 `analysis` 這種多寫入者的槽結果會飄。
3. **`is_critical()` 尊重原設計**：step 自己說不 critical 就記一筆警告繼續跑。
   runner 收集 `Vec<StepWarning>` 一起回傳。

### 進度事件

沿用 repo 既有慣例（`asr:progress` / `dedup:*-progress`）：

| 事件 | payload（camelCase） |
|---|---|
| `graph:progress` | `{ runId, nodeId, nodeRef, stepIndex, totalSteps, phase: 'start'\|'done', progressPercent }` |
| `graph:complete` | `{ runId, succeeded, warnings: string[] }` |
| `graph:error` | `{ runId, nodeId, message }` |

取消用 `AtomicBool`，存在 `OnceLock<Mutex<HashMap<String, Arc<AtomicBool>>>>`——
與 `commands/events.rs` 的 `ACTIVE_PIPELINES` 同一個 pattern。

---

## 6. `validate.rs` — G1–G7

分兩層（抄參考文檔 §6c 的兩層拆分，那個設計是對的）：

**`validate_integrity`＝寫入閘**，只問「這份資料讀不讀得回來」：

- **G1** `node_id` 唯一且非空；`node_ref` 非空；`kind` ∈ {source, method, sink}
- **G2** 邊的兩端都指向存在的節點；不自環；不重複

**刻意不查** `node_ref` 在不在 manifest——manifest 換版之後，圖也要存得下「把那顆
換掉」的修法。

**`validate_runnable`＝執行前的判官**（`validate_integrity` 之後再往下）：

- **G3** 引用層：`node_ref` 在 manifest 查得到；params 的 key 都是宣告過的；
  `required` 的都有值；enum 值在 `options` 裡；數值在 `min`/`max` 內
- **G4** 恰好一顆 `kind: 'source'`，且它入度 0
- **G5** 無環（`topo_order` 排不出來＝有環）
- **G6** 槽供給：每顆 method 的 `reads`，在拓樸序上它之前必須有人寫過那個槽
  （或者該 param 有值不需要 fallback）；`source` 的 `accepts` 要包含實際來源變體
  （⚠️ Stream / Camera 只有 `capture_frame` 接得住，見 manifest §1a）
- **G7** 同一個 `node_ref` 在一張圖裡**最多一顆**（黑板一個槽一份，見主計畫 §7）

**只警告不擋**（`GraphVerdict { warnings }`）：

- 同一個槽有多個寫入者，且其中至少一顆是 `WriteMode::Replace`
  （例：`detect_motion` 會清掉 `group_similar_images` 寫的東西）
- 節點宣告 `requires_ffmpeg` 但環境 PATH 找不到 ffmpeg
- 孤兒節點（沒有任何邊連到它）——不是壞圖，是還沒畫完

錯誤訊息**全部中文原句**，前端原樣顯示，後端是裁判。

---

## 7. Tauri commands（`commands/graph.rs`）

| command | 簽章 | 用途 |
|---|---|---|
| `list_graph_methods` | `() -> Result<Vec<MethodMetaDto>, String>` | 前端建 picker ＋ 右欄；**前端不硬編任何 step 名字** |
| `validate_graph` | `(recipe: GraphRecipe) -> Result<GraphVerdict, String>` | `{ valid, reason?, warnings, order? }` |
| `execute_graph` | `(app, recipe: GraphRecipe) -> Result<GraphRunResult, String>` | 先 validate，再 `run_graph`，回傳序列化後的黑板 |
| `cancel_graph_run` | `(runId: String) -> Result<(), String>` | 掀 AtomicBool |

`GraphRunResult` 是 `MediaContext` 的可序列化投影（`MediaContext` 本身有 `dyn Any`
不能 serde）：

```rust
#[derive(Serialize)] #[serde(rename_all = "camelCase")]
pub struct GraphRunResult {
    pub run_id: String,
    pub metadata: Option<MediaMetadata>,
    pub analysis: Option<AnalysisResultDto>,      // 重用 commands/pipeline.rs 既有的 DTO
    pub extracted_frame_count: usize,             // ⚠️ 不回傳 base64 陣列，太肥
    pub hls_result: Option<HlsResultDto>,
    pub annotation_result: Option<AnnotationResultDto>,
    pub video_process_result: Option<VideoProcessResultDto>,
    pub process_result: Option<ProcessFilesResultDto>,
    pub warnings: Vec<String>,
}
```

> `commands/pipeline.rs` 已經有 `AnalysisResult`（:58）／`HLSConversionResult`（:352）
> ／`ExtractToDiskResult`（:419）等 DTO，且**都已經是 `pub`**（`commands/mod.rs` 也是
> `pub mod pipeline;`）⇒ 直接 `use crate::commands::pipeline::AnalysisResult;` 就好，
> **不用搬家、不用改可見度、既有檔案零 diff**。
> 缺的只有 `AnnotationResult` / `ProcessFilesResult` 兩個投影，在 `graph/types.rs` 新寫。

`execute_graph` 要在 `std::thread::spawn` 裡跑（沿用既有 14 支 command 的做法——
opencv 的東西不能在 async runtime 上直接跑）。

---

## 8. 驗收（Phase 0 的 Definition of Done）— ✅ 2026-08-07 完成

> 全部通過。`cargo check` 0 errors（只剩既有的 `db/queries.rs:288` dead_code）、
> `cargo test --bin media-app graph::` **26 passed / 0 failed**
> （22 支圖形驗證 ＋ 4 支真實執行，測試檔 `src/graph/tests.rs`、`src/graph/tests_run.rs`）。
> 執行類測試用 ffmpeg 合成一支 2 秒 160×120 的 testsrc 當 fixture，ffmpeg 不在就明講
> 跳過而不是靜默通過。


1. `cargo check` 0 errors（baseline：只有 `db/queries.rs:288` 那個既有 dead_code 警告）
2. `list_graph_methods` 回得出 12 筆，欄位齊全
3. **扇形圖**跑得動：
   ```json
   { "label": "smoke-fan", "nodes": [
       {"nodeId":"n0","kind":"source","nodeRef":"file","params":{"path":"…/sample.mp4"}},
       {"nodeId":"n1","kind":"method","nodeRef":"extract_metadata"},
       {"nodeId":"n2","kind":"method","nodeRef":"detect_motion","params":{"threshold":30.0}}
     ], "edges": [{"from":"n0","to":"n1"},{"from":"n0","to":"n2"}] }
   ```
4. **串接圖**跑得動（這條才是重點——證明 §5 的 runner 設計成立）：
   ```json
   { "label": "smoke-chain", "nodes": [
       {"nodeId":"n0","kind":"source","nodeRef":"file","params":{"path":"…/sample.mp4"}},
       {"nodeId":"n1","kind":"method","nodeRef":"extract_frames_to_disk",
        "params":{"outputDir":"…/frames","interval":30}},
       {"nodeId":"n2","kind":"method","nodeRef":"group_similar_images",
        "params":{"outputDir":"…/groups"}}
     ], "edges": [{"from":"n0","to":"n1"},{"from":"n1","to":"n2"}] }
   ```
   **`n2` 沒填 `inputDir`**——它必須自動吃到 `n1` 的 `outputDir`。這條過了，
   整個設計就成立。
5. 壞圖被擋且訊息看得懂：有環、兩顆 source、`capture_frame` 以外的 step 接 Camera 來源、
   同一個 `node_ref` 放兩顆、required param 沒填
6. 取消：長跑中途 `cancel_graph_run` 能中斷

驗收先用**手寫 JSON ＋ 一個暫時的 debug 呼叫**（或 `#[cfg(test)]` 測試），
**不建前端頁面**。

---

## 9. Phase 0 刻意不做

| 項目 | 為什麼 |
|---|---|
| 存載（`store.rs`、JSON 檔 CRUD） | Phase 2。Phase 0 用手寫 JSON 驗收就夠 |
| 任何前端 | 這階段的價值就在於「不碰前端也能知道值不值得做」 |
| 真並行執行 | 拓樸線性化已經正確，並行是效能題不是正確性題 |
| `benchmark` 那兩顆 step | `BenchmarkContext` 泛型不同，見 manifest §3 |
| `RebindSource` 節點 | 威力大但語意 hacky（黑板中途換 source），記帳留 Phase 2 |
| 逐邊傳值（同一 method 放多顆） | 黑板模型的固有限制，G7 直接擋掉，記帳 |
| 進度百分比的精細化 | 現在是「第幾顆／共幾顆」。step 內部進度要改 media-core，跨 repo，不做 |

---

## 10. 拍板紀錄

1. ~~`save_report` 要不要放進 Phase 0？~~ → **做了**。`src/graph/steps/save_report.rs`，
   圖的深度因此變 3 層，也證明了 app 側自訂 step 可行（media-core 零 diff）。
2. ~~`GraphRunResult` 的 `extracted_frames`~~ → **只回數量**（`extractedFrameCount`）。
   base64 幀陣列太肥，Phase 1 真的要預覽再開新 command。
3. ~~`commands/pipeline.rs` 的 DTO 搬家~~ → 不需要，那些 DTO 已經是 `pub`，直接 `use`。
   既有後端檔案**完全零 diff**（只有 `commands/mod.rs` ＋ `main.rs` 各加幾行註冊）。

## 11. 實作時偏離本文檔的地方（以 code 為準）

| 文檔原本寫 | 實際做法 | 為什麼 |
|---|---|---|
| 錯誤訊息「全部中文原句」 | **英文** | 那是照抄參考專案（中文介面）的慣例。本 repo 的 nav label、既有後端錯誤訊息全是英文，跟著走才一致 |
| `run_graph(app, ...)` 直接 `app.emit` | 多一層 `run_graph_with(recipe, run_id, cancel, emit)`，事件輸出注入 | 否則沒有 `AppHandle` 就測不動。`run_graph` 只是包一層 |
| `NodeParams` 型別檢查只在 validate | validate（對 manifest）＋ registry（取值時）**兩道** | registry 那道是防禦性的：驗過的圖才會進來，但錯誤訊息要能指出是哪個 key |
| 三條路徑繼承鏈 | **兩條**（`process_files` 接不上） | 見 `REFERENCE_composer_method_manifest.md` §4 |
| `saveMode` 交給 step 自己的預設 | registry 明確補 `single_directory` | media-core 的 `SaveMode::default()` 是 `MultipleDirectory`，會讓 `output_dir` 變成幀目錄的**父目錄**，繼承鏈整條指錯地方。見 manifest §5 第一列 |
