# MediaVault — 智慧媒體去重工具開發計畫

> **Tech Stack**: Tauri 2 + Svelte 5 + Rust + SurrealDB (Embedded)
> **目標**: 掃描多個資料夾中的照片與影片，透過多層雜湊與感知指紋辨識重複檔案，提供視覺化介面進行去重管理。

---

## 一、系統架構總覽

```
┌─────────────────────────────────────────────────────┐
│                   Svelte 5 Frontend                  │
│  ┌───────────┐ ┌───────────┐ ┌────────────────────┐ │
│  │ 路徑匯入   │ │ 掃描進度   │ │ 重複群組瀏覽/管理  │ │
│  └─────┬─────┘ └─────┬─────┘ └─────────┬──────────┘ │
│        │             │                 │             │
│        └─────────────┼─────────────────┘             │
│                      │ Tauri IPC (invoke)            │
├──────────────────────┼──────────────────────────────-┤
│                      │                               │
│              Rust Backend (Tauri)                     │
│  ┌──────────┐ ┌──────────────┐ ┌──────────────────┐ │
│  │ 檔案掃描  │ │ 指紋計算引擎  │ │ 重複比對引擎     │ │
│  │ Walker   │ │ Hash+pHash   │ │ Hamming/BK-Tree  │ │
│  └────┬─────┘ └──────┬───────┘ └────────┬─────────┘ │
│       │              │                  │            │
│       └──────────────┼──────────────────┘            │
│                      │                               │
│              ┌───────┴────────┐                      │
│              │ SurrealDB      │                      │
│              │ (Embedded)     │                      │
│              └────────────────┘                      │
└──────────────────────────────────────────────────────┘
```

---

## 二、核心資料模型 (SurrealDB Schema)

### 2.1 media_file — 媒體檔案記錄

```sql
DEFINE TABLE media_file SCHEMAFULL;

DEFINE FIELD file_path       ON media_file TYPE string;    -- 完整絕對路徑
DEFINE FIELD file_name       ON media_file TYPE string;    -- 檔名
DEFINE FIELD file_size       ON media_file TYPE int;       -- bytes
DEFINE FIELD file_type       ON media_file TYPE string;    -- "image" | "video"
DEFINE FIELD mime_type       ON media_file TYPE string;    -- e.g. "image/jpeg"
DEFINE FIELD content_hash    ON media_file TYPE string;    -- BLAKE3 全檔 hash
DEFINE FIELD phash           ON media_file TYPE string;    -- perceptual hash (hex string, 64-bit)
DEFINE FIELD thumbnail       ON media_file TYPE bytes;     -- 縮圖 (JPEG, ~200px, 存 blob)
DEFINE FIELD scan_source     ON media_file TYPE record<scan_source>;
DEFINE FIELD created_at      ON media_file TYPE datetime DEFAULT time::now();
DEFINE FIELD file_modified_at ON media_file TYPE datetime; -- 檔案系統的修改時間

DEFINE INDEX idx_content_hash ON media_file FIELDS content_hash;
DEFINE INDEX idx_phash        ON media_file FIELDS phash;
DEFINE INDEX idx_file_path    ON media_file FIELDS file_path UNIQUE;
```

### 2.2 scan_source — 掃描來源路徑

```sql
DEFINE TABLE scan_source SCHEMAFULL;

DEFINE FIELD path         ON scan_source TYPE string;
DEFINE FIELD label        ON scan_source TYPE option<string>;  -- 使用者自定標籤
DEFINE FIELD scanned_at   ON scan_source TYPE datetime DEFAULT time::now();
DEFINE FIELD file_count   ON scan_source TYPE int DEFAULT 0;
DEFINE FIELD status       ON scan_source TYPE string DEFAULT "pending";
-- status: "pending" | "scanning" | "completed" | "error"

DEFINE INDEX idx_source_path ON scan_source FIELDS path UNIQUE;
```

### 2.3 duplicate_group — 重複群組

```sql
DEFINE TABLE duplicate_group SCHEMAFULL;

DEFINE FIELD match_type   ON duplicate_group TYPE string;  -- "exact" | "similar"
DEFINE FIELD anchor_hash  ON duplicate_group TYPE string;  -- 用於歸組的 hash 值
DEFINE FIELD members      ON duplicate_group TYPE array<record<media_file>>;
DEFINE FIELD created_at   ON duplicate_group TYPE datetime DEFAULT time::now();
```

---

## 三、Rust 後端模組設計

### 3.1 模組拆分

```
src-tauri/src/
├── main.rs              -- Tauri 啟動入口
├── lib.rs               -- 模組註冊
├── db/
│   └── mod.rs           -- SurrealDB 連線管理 (embedded)
├── scanner/
│   ├── mod.rs           -- 檔案系統掃描 (walkdir + ignore)
│   └── filter.rs        -- 副檔名過濾 (圖片/影片白名單)
├── hasher/
│   ├── mod.rs           -- Hash 引擎統一介面
│   ├── content_hash.rs  -- BLAKE3 全檔 hash
│   ├── perceptual.rs    -- pHash / dHash 計算
│   └── video.rs         -- 影片指紋 (關鍵幀提取 + pHash)
├── matcher/
│   ├── mod.rs           -- 重複比對邏輯
│   ├── exact.rs         -- content_hash 完全比對
│   └── similar.rs       -- Hamming distance 近似比對 (BK-tree)
├── thumbnail/
│   └── mod.rs           -- 縮圖生成
└── commands/
    └── mod.rs           -- Tauri IPC command 定義
```

### 3.2 關鍵 Crate 選型

| 用途 | Crate | 說明 |
|------|-------|------|
| 檔案遍歷 | `walkdir` | 遞迴掃描資料夾 |
| 內容雜湊 | `blake3` | 極快的 cryptographic hash |
| 圖片解碼 | `image` | 支援 JPEG/PNG/WebP/TIFF 等 |
| 感知雜湊 | `img_hash` | pHash/dHash/aHash 實作 |
| 影片處理 | `ffmpeg-next` (或 CLI 呼叫 ffmpeg) | 關鍵幀提取 |
| 縮圖生成 | `image` (resize) | 產生預覽用小圖 |
| 近似比對 | `bk-tree` | Hamming space 的高效近鄰搜尋 |
| 資料庫 | `surrealdb` (embedded feature) | 嵌入式模式，無需外部服務 |
| 並行處理 | `rayon` | data parallelism，充分利用多核 |
| 非同步 | `tokio` | Tauri 的 async runtime |
| 序列化 | `serde` + `serde_json` | 前後端資料傳遞 |
| EXIF | `kamadak-exif` 或 `rexiv2` | 讀取拍攝日期等 metadata |

### 3.3 處理 Pipeline（核心流程）

```
使用者選擇路徑
    │
    ▼
[Stage 1] 檔案掃描
    │  walkdir 遞迴掃描
    │  過濾支援的副檔名
    │  記錄 file_path, file_size, file_modified_at
    │
    ▼
[Stage 2] 快速去重 — BLAKE3 Content Hash
    │  rayon 並行計算每個檔案的 BLAKE3 hash
    │  寫入 DB 前先查詢：content_hash 是否已存在？
    │  ├─ 已存在 → 標記為 exact duplicate，加入 duplicate_group
    │  └─ 不存在 → 繼續下一階段
    │
    ▼
[Stage 3] 感知指紋 — Perceptual Hash
    │  對圖片：image decode → img_hash 計算 pHash (64-bit)
    │  對影片：ffmpeg 提取 N 個關鍵幀 → 每幀算 pHash → 取代表值
    │  同時生成縮圖 (200px JPEG)
    │  寫入 DB
    │
    ▼
[Stage 4] 近似比對
    │  新檔案的 pHash 與 DB 中所有既有 pHash 做 Hamming distance 比較
    │  閾值建議：distance ≤ 8 為「相似」（64-bit hash 中容許 8 bit 差異）
    │  使用 BK-tree 加速（從 DB 載入建樹，或常駐記憶體）
    │  命中 → 建立 duplicate_group (match_type = "similar")
    │
    ▼
[Stage 5] 結果寫回 DB + 通知前端
    │  透過 Tauri event 即時推送進度與結果
    └─ 前端更新 UI
```

### 3.4 影片指紋策略

影片不能像圖片一樣直接做 pHash，建議的策略：

1. **關鍵幀提取**: 用 ffmpeg 取影片的 I-frames（或等間距取 5-10 幀）
2. **逐幀 pHash**: 每個關鍵幀各算一個 pHash
3. **代表指紋**: 取所有幀的 pHash 組合成一個指紋向量
4. **比對方式**: 兩段影片的指紋向量做交叉比對，如果多數幀的 Hamming distance 都很小，判定為重複
5. **額外輔助**: 影片長度（duration）、檔案大小可以作為快速預篩條件

---

## 四、Tauri IPC Commands 定義

```rust
// === 掃描來源管理 ===
#[tauri::command]
async fn add_scan_source(path: String, label: Option<String>) -> Result<ScanSource, String>;

#[tauri::command]
async fn list_scan_sources() -> Result<Vec<ScanSource>, String>;

#[tauri::command]
async fn remove_scan_source(id: String) -> Result<(), String>;

// === 掃描控制 ===
#[tauri::command]
async fn start_scan(source_id: String, app: AppHandle) -> Result<(), String>;
// 進度透過 Tauri Event 推送: "scan://progress" { scanned, total, current_file, stage }

#[tauri::command]
async fn cancel_scan() -> Result<(), String>;

// === 查詢結果 ===
#[tauri::command]
async fn get_duplicate_groups(
    match_type: Option<String>,  // "exact" | "similar" | None (all)
    page: u32,
    page_size: u32,
) -> Result<PaginatedResult<DuplicateGroup>, String>;

#[tauri::command]
async fn get_group_detail(group_id: String) -> Result<DuplicateGroupDetail, String>;

// === 檔案操作 ===
#[tauri::command]
async fn get_thumbnail(file_id: String) -> Result<Vec<u8>, String>;

#[tauri::command]
async fn open_file_location(file_path: String) -> Result<(), String>;
// 呼叫系統檔案管理器開啟所在資料夾

#[tauri::command]
async fn delete_file(file_id: String, move_to_trash: bool) -> Result<(), String>;
// move_to_trash=true 時移到資源回收桶而非永久刪除

// === 統計 ===
#[tauri::command]
async fn get_stats() -> Result<Stats, String>;
// Stats { total_files, total_duplicates, space_saved_potential, sources_count }
```

---

## 五、Svelte 5 前端頁面設計

### 5.1 頁面結構

```
App
├── Layout (側邊欄 + 主內容區)
│   ├── Sidebar
│   │   ├── Logo + 應用名稱
│   │   ├── Nav: 總覽 / 掃描來源 / 重複項目 / 設定
│   │   └── 儲存空間統計摘要
│   │
│   └── Main Content Area
│       ├── [/] Dashboard — 總覽頁
│       │   ├── 統計卡片 (總檔案數、重複數、可節省空間)
│       │   ├── 最近掃描活動
│       │   └── 快速操作按鈕
│       │
│       ├── [/sources] 掃描來源管理
│       │   ├── 「新增路徑」按鈕 → 開啟系統資料夾選擇器
│       │   ├── 已匯入路徑列表 (顯示狀態、檔案數、上次掃描時間)
│       │   └── 每個來源可觸發「重新掃描」或「移除」
│       │
│       ├── [/duplicates] 重複項目瀏覽
│       │   ├── 篩選列: 全部 / 完全相同 / 相似
│       │   ├── 重複群組卡片列表 (每組顯示縮圖網格預覽)
│       │   └── 點擊群組 → 展開詳細比較視圖
│       │       ├── 並排/堆疊顯示所有成員
│       │       ├── 每張顯示: 縮圖、檔名、路徑、大小、修改日期
│       │       ├── 標記「保留」或「刪除」
│       │       └── 批次操作按鈕
│       │
│       └── [/settings] 設定
│           ├── 相似度閾值調整 (Hamming distance slider)
│           ├── 影片關鍵幀數量設定
│           ├── 支援的檔案格式管理
│           └── 資料庫路徑 / 重置資料庫
```

### 5.2 關鍵元件

```
components/
├── ScanProgress.svelte        -- 掃描進度條 + 即時狀態
│                                 (監聽 Tauri event "scan://progress")
├── DuplicateGroupCard.svelte  -- 重複群組預覽卡片
│                                 (縮圖網格 + 成員數 + 匹配類型標籤)
├── ComparisonView.svelte      -- 群組內檔案並排比較
│                                 (支援左右滑動對比)
├── FileCard.svelte            -- 單一檔案資訊卡
│                                 (縮圖 + metadata + 操作按鈕)
├── ThumbnailGrid.svelte       -- 縮圖網格顯示
├── PathSelector.svelte        -- 呼叫 Tauri dialog 選擇資料夾
├── StatsCard.svelte           -- 統計數字卡片
└── ConfirmDialog.svelte       -- 刪除確認對話框
```

### 5.3 狀態管理

使用 Svelte 5 的 `$state` rune 搭配一個簡單的 store 模式：

```
stores/
├── sources.svelte.ts     -- 掃描來源列表 + CRUD
├── scan.svelte.ts        -- 掃描進度狀態 (listening Tauri events)
├── duplicates.svelte.ts  -- 重複群組 + 分頁 + 篩選
└── settings.svelte.ts    -- 使用者偏好設定
```

---

## 六、開發階段與里程碑

### Phase 1：基礎建設（Week 1-2）

**目標**: 專案骨架跑起來，DB 連得上，能掃描檔案

- [ ] `cargo create-tauri-app` 初始化 Tauri 2 + Svelte 5 專案
- [ ] 整合 SurrealDB embedded（`surrealdb` crate，`kv-rocksdb` feature）
- [ ] 實作 DB 初始化：啟動時自動建表建索引
- [ ] 實作 `scanner` 模組：walkdir 掃描 + 副檔名白名單過濾
- [ ] 實作 `add_scan_source` / `list_scan_sources` commands
- [ ] 前端：基本 Layout + 路徑選擇器（Tauri dialog API）
- [ ] 前端：掃描來源列表頁面

**驗收標準**: 能選一個資料夾，掃描出所有圖片/影片的 metadata 並存入 DB。

### Phase 2：指紋引擎（Week 3-4）

**目標**: 能計算所有檔案的 hash 和 pHash

- [ ] 實作 `content_hash.rs`：BLAKE3 並行計算
- [ ] 實作 `perceptual.rs`：img_hash 計算圖片 pHash
- [ ] 實作 `video.rs`：ffmpeg 關鍵幀提取 + 逐幀 pHash
- [ ] 實作 `thumbnail/mod.rs`：縮圖生成 (200px JPEG)
- [ ] 整合進掃描 pipeline：Stage 1 → 2 → 3 依序執行
- [ ] Tauri Event 推送掃描進度
- [ ] 前端：掃描進度元件 (ScanProgress.svelte)

**驗收標準**: 掃描完成後，DB 中每個 media_file 都有 content_hash、phash、thumbnail。

### Phase 3：比對引擎（Week 5-6）

**目標**: 能自動找出完全重複和相似的檔案

- [ ] 實作 `exact.rs`：content_hash grouping（SQL GROUP BY 即可）
- [ ] 實作 `similar.rs`：BK-tree 建構 + Hamming distance 查詢
- [ ] 實作 duplicate_group 寫入邏輯
- [ ] 處理增量掃描：新檔案只需與既有指紋比對，不必全部重算
- [ ] 實作 `get_duplicate_groups` / `get_group_detail` commands

**驗收標準**: 掃描兩個含重複照片的資料夾後，能正確列出重複群組。

### Phase 4：前端完整 UI（Week 7-8）

**目標**: 完整的使用者體驗

- [ ] Dashboard 總覽頁（統計卡片 + 最近活動）
- [ ] 重複群組列表頁（篩選 + 分頁 + 縮圖預覽）
- [ ] 群組詳細比較視圖（並排對比 + metadata 顯示）
- [ ] 檔案操作：開啟檔案位置、移至資源回收桶
- [ ] 批次操作：一鍵保留最佳品質 / 刪除其餘
- [ ] 設定頁面（閾值、格式、DB 管理）

**驗收標準**: 完整走完「匯入 → 掃描 → 檢視重複 → 刪除」的完整流程。

### Phase 5：優化與打磨（Week 9-10）

**目標**: 效能與體驗的最後一哩路

- [ ] 大量檔案效能測試（10K+ 檔案）
- [ ] 記憶體使用優化（streaming 處理，避免一次載入所有縮圖）
- [ ] 增量掃描優化（只處理新增/修改的檔案，用 file_modified_at 判斷）
- [ ] 錯誤處理強化（損壞檔案、權限問題、磁碟滿）
- [ ] UI 動畫與回饋打磨
- [ ] 跨平台測試（若需要 macOS/Windows）

---

## 七、關鍵技術決策備註

### 7.1 SurrealDB Embedded 注意事項

```toml
# Cargo.toml
[dependencies]
surrealdb = { version = "2", features = ["kv-rocksdb"] }
# kv-rocksdb 用於持久化嵌入式模式
# 資料庫檔案位置建議放在 Tauri 的 app_data_dir
```

啟動方式：
```rust
use surrealdb::Surreal;
use surrealdb::engine::local::RocksDb;

let db = Surreal::new::<RocksDb>("path/to/db").await?;
db.use_ns("mediavault").use_db("main").await?;
```

### 7.2 影片 ffmpeg 策略

建議先用 CLI 呼叫 ffmpeg（`std::process::Command`），而非 `ffmpeg-next` binding，原因：
- 部署更簡單（使用者安裝 ffmpeg 或你 bundle 進去）
- 不需要在編譯時 link FFmpeg 的 C libraries
- 足夠應付「提取關鍵幀」這個簡單需求

```bash
# 提取 10 個等間距關鍵幀
ffmpeg -i input.mp4 -vf "select='not(mod(n,INTERVAL))'" -frames:v 10 -vsync vfn frame_%03d.jpg
```

### 7.3 pHash 閾值建議

| Hamming Distance | 意義 |
|-----------------|------|
| 0 | 感知完全相同 |
| 1-5 | 極度相似（壓縮差異、微調） |
| 6-10 | 相似（輕微裁剪、色調變化） |
| 11-15 | 有點像（不同角度、大幅編輯） |
| > 15 | 不同的圖片 |

建議預設閾值 **≤ 8**，並在設定頁面開放使用者調整。

### 7.4 支援的檔案格式白名單

```rust
const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif",
    "heic", "heif", "avif", "raw", "cr2", "nef", "arw",
];

const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "avi", "mkv", "wmv", "flv", "webm",
    "m4v", "3gp", "ts", "mts",
];
```

### 7.5 縮圖儲存策略

將縮圖存在 SurrealDB 的 `bytes` 欄位中（JPEG 壓縮後約 5-15KB/張）。
好處是不用額外管理縮圖檔案系統，壞處是 DB 會膨脹。
如果量級到十萬張以上，可考慮改為存在 filesystem 只在 DB 存路徑。

---

## 八、風險與備案

| 風險 | 影響 | 備案 |
|------|------|------|
| SurrealDB embedded 效能瓶頸 | 大量 pHash 查詢慢 | BK-tree 常駐記憶體，DB 只做持久化 |
| ffmpeg 安裝門檻 | 使用者不會裝 | Bundle ffmpeg binary 或改為選配功能 |
| HEIC/RAW 格式支持差 | 部分照片無法處理 | 用 `libheif` 或先轉 JPEG 再處理 |
| 十萬級檔案掃描耗時 | UX 不佳 | 增量掃描 + 背景處理 + 即時顯示已發現的重複 |
| 縮圖讓 DB 過大 | 磁碟空間 | 壓縮品質降低或改存 filesystem |
