# 跨平台測試策略

本文件整理 media-app（Tauri 2 + SvelteKit）在只有 macOS 開發環境下，如何驗證 Windows / Linux / macOS 三平台的可用性，以及前端 E2E 自動化測試的各種方案與取捨。

## 背景與限制

- **開發機器**：僅 macOS（Apple Silicon）
- **目標平台**：macOS、Linux、Windows 三平台都要能正常建置與執行
- **原則**：盡量自動化，避免每次發布都手動點擊操作
- **成本考量**：Repo 為 public，GitHub Actions 對 public repo 無額度上限，可自由使用

---

## 第一層：跨平台 Build 驗證

確保程式碼在三個平台都能成功編譯、通過型別檢查與單元測試。這是最基本的保護網。

### 1. GitHub Actions CI（推薦）

對於 public repository，GitHub Actions 完全免費、無分鐘數限制。

| Repository 類型 | Actions 額度 |
|----------------|-------------|
| Public | 無限制 |
| Private (Free) | 每月 2,000 分鐘 |
| Private (Pro) | 每月 3,000 分鐘 |

**CI 可驗證的項目**：

- 三平台 Rust 編譯（OpenCV、ffmpeg、SurrealDB 等原生依賴的連結）
- `svelte-check` 型別檢查
- `media-core/tests/` Rust 單元測試
- `yarn tauri build` 打包是否成功

**CI 無法驗證的項目**：

- GUI 實際渲染結果
- Runtime 互動行為
- 平台特定的檔案對話框、權限等

預計每次 push 約 10-15 分鐘跑完三平台。

### 2. Docker 用於本機 Linux 驗證

在 macOS 上用 Docker 可以**完整測試 Linux 的 build 與 runtime 邏輯**，包括產出 `.deb` / `.AppImage` 安裝包。

**Docker 能做到的**：

- Rust 編譯 + 單元測試
- 前端 build + 型別檢查
- `yarn tauri build` 產出真正可安裝的 Linux 桌面應用
- 跨平台路徑處理驗證

**Docker 做不到的**：

- 開啟 GUI 視窗（需要 X11 或 Wayland display server）
- 跑 Windows container（需要 Windows kernel，macOS 上不可能）
- 跑 macOS container（Apple 授權限制）

**架構注意事項**：

Apple Silicon Mac 預設會 build ARM image，若目標使用者是 x86_64 Linux，需要在 build 時指定平台：

```bash
docker build --platform linux/amd64 -t media-app-linux .
```

透過 QEMU 模擬 x86_64，速度較慢但可產出正確架構的安裝包。

### 3. Windows 測試方案

macOS 本機**完全無法**在 Docker 中測試 Windows，因為 Docker container 共享宿主機 kernel，Windows container 需要 Windows kernel。

可選方案：

| 方案 | 成本 | 適用情境 |
|------|------|---------|
| **GitHub Actions** | 免費 | 自動化 build 與單元測試 |
| **Parallels + Windows VM** | 付費 | 需要手動測 GUI 時 |
| **UTM + Windows VM** | 免費 | 同上，但 Windows on ARM 相容性偶爾有問題 |

**推薦**：日常開發完全依賴 GitHub Actions，發布前若有疑慮再用 VM 手動驗證。

---

## 第二層：前端測試分層

Tauri 的前端本質上是一個網頁（WebKit2GTK / WebView2 / WKWebView），因此可以沿用 Web 生態的測試工具。合理的分層如下：

```
  多    Vitest + Testing Library（元件單元測試）
   ↑      純邏輯、資料轉換、derived 狀態
   |
   |    Playwright（前端 E2E）
   |      按鈕點擊、路由切換、表單驗證、UI 狀態
   |
  少    WebdriverIO + Tauri Driver（完整整合 E2E）
          關鍵前後端串通流程，大版本發布前才跑
```

### Vitest：元件單元測試

- 測試純邏輯函數、資料轉換、Svelte 5 runes 的 derived/effect 行為
- 不開瀏覽器，在 Node 環境下執行，極快
- 搭配 `@testing-library/svelte` 可以測元件的行為（而非實作細節）

### Playwright：前端 E2E

- 測試對象是前端 UI 邏輯
- 跑起來是真實瀏覽器，但連接的是 `yarn dev` 起的 localhost:1520
- 設定簡單、生態成熟、支援截圖比對與 trace 錄影

### WebdriverIO + Tauri Driver：完整整合測試

- 啟動真正打包後的 Tauri 桌面應用
- 透過 WebDriver 協議操作視窗中的 webview
- 設定成本高、跑得慢，只用於關鍵流程的驗證

---

## 第三層：各平台 E2E 自動化方案

### Linux（首選 Docker + WebdriverIO）

Linux 上 WebDriver 對 WebKit2GTK 支援最完整，是 Tauri 官方推薦的 E2E 測試方式。

```
Docker container (Ubuntu)
  ├─ Xvfb（虛擬 framebuffer，無需實體螢幕）
  ├─ tauri-driver（WebDriver bridge）
  ├─ 打包後的 Tauri 應用
  └─ WebdriverIO 測試腳本
```

**流程**：

1. `cargo install tauri-driver`
2. `yarn tauri build` 產出 Linux binary
3. 在容器中透過 Xvfb 啟動虛擬 display
4. 啟動 `tauri-driver`（預設 port 4444）
5. 跑 WebdriverIO 測試腳本

**設定範例**：

```js
// wdio.conf.js
exports.config = {
  specs: ['./tests/e2e/**/*.test.js'],
  maxInstances: 1,

  capabilities: [{
    'tauri:options': {
      application: './src-tauri/target/release/media-app',
    },
  }],

  hostname: '127.0.0.1',
  port: 4444,
  framework: 'mocha',

  onPrepare: () => {
    // 自動啟動 tauri-driver
  },
};
```

### Windows（GitHub Actions）

WebView2 有完整的 WebDriver 支援。由於 macOS 本機無法測試 Windows，直接把 WebdriverIO 整合測試跑在 GitHub Actions 的 `windows-latest` runner 即可。

### macOS（最棘手）

Apple 對 WKWebView 的 WebDriver 支援有限，`tauri-driver` 在 macOS 上的穩定度不如 Linux / Windows。有以下幾種可行方案：

#### 方案 A：Playwright + Mock Tauri API（最務實）

跑 `yarn dev` 只起前端，用 Playwright 操作，`invoke()` 全部 mock 掉。

```js
await page.addInitScript(() => {
  window.__TAURI_INTERNALS__ = {
    invoke: async (cmd, args) => {
      const mocks = {
        'get_scan_sources': [
          { id: '1', path: '/test/photos', role: 'source' }
        ],
        'get_dedup_stats': { total_files: 150, duplicates: 23 },
      };
      return mocks[cmd] ?? null;
    },
  };
});
```

**優點**：設定簡單、速度快、CSS selector 定位、生態成熟
**缺點**：Rust 後端是假的，不是真正的整合測試
**適用**：日常開發與 CI，涵蓋 90% 的 UI 邏輯測試

#### 方案 B：Playwright + 真實 Tauri Dev Server

跑 `yarn tauri dev` 啟動完整 Tauri 應用，同時用 Playwright 對 `localhost:1520` 跑測試。此時 `invoke()` 會真的打到 Rust 後端。

```
Terminal 1:  yarn tauri dev        ← 完整的 Tauri 應用跑起來
Terminal 2:  npx playwright test   ← 對 localhost:1520 跑測試
```

**優點**：前後端都是真的、設定比 Appium 簡單
**缺點**：Playwright 的操作不會反映在 Tauri 視窗上（兩個獨立 client），Tauri 專屬原生 API（檔案對話框、系統通知等）無法測試
**適用**：macOS 發布前的整合驗證

#### 方案 C：Appium + Mac2 Driver（原生自動化）

透過 Apple 的 XCUITest 框架操作整個 macOS 應用，包括原生對話框、選單、系統功能。

```
Appium (Node.js)
    │  WebDriver 協議
    ▼
Mac2 Driver
    │  XCUITest (Apple Accessibility API)
    ▼
你的 Tauri 應用（完整桌面視窗）
```

```js
const capabilities = {
  platformName: 'mac',
  'appium:automationName': 'mac2',
  'appium:bundleId': 'com.media-app.dev',
};
```

**優點**：唯一能測試原生 macOS UI 的方案
**缺點**：元素定位靠 accessibility label 而非 CSS selector，寫起來沒那麼直覺；設定複雜
**適用**：需要測原生功能（檔案對話框、系統選單）時

---

## 建議的測試矩陣

| 測試層級 | Linux | Windows | macOS |
|---------|-------|---------|-------|
| **Build 驗證** | GitHub Actions + 本機 Docker | GitHub Actions | 本機 `yarn tauri build` |
| **Rust 單元測試** | GitHub Actions + Docker | GitHub Actions | 本機 |
| **前端單元測試** | Vitest（CI 跑一次即可） | — | — |
| **前端 E2E** | Playwright（CI 或本機） | Playwright（CI） | Playwright + Mock |
| **整合 E2E** | WebdriverIO + tauri-driver（Docker） | WebdriverIO + tauri-driver（GitHub Actions） | Playwright + 真實 dev server |
| **原生 UI 測試** | — | — | Appium + Mac2 driver |

---

## 實作優先順序

依照投資報酬率排序，建議按順序導入：

1. **GitHub Actions CI**（三平台 build + 單元測試）
   - 成本最低、效益最高
   - 每次 push 都自動擋住編譯問題
   - 免費無限制

2. **Playwright + Mock Tauri API**
   - 涵蓋大部分前端 UI 邏輯
   - 跑得快、可在 CI 執行
   - 日常開發的主力 E2E

3. **Docker Linux build 腳本**
   - 本機快速驗證 Linux 可用性
   - 產出真正可安裝的 `.deb` / `.AppImage`

4. **Vitest 元件單元測試**
   - 針對純邏輯函數與元件行為補強

5. **WebdriverIO + tauri-driver（Docker + Xvfb）**
   - 大版本發布前才跑
   - 涵蓋 Linux 上的完整前後端整合

6. **Appium + Mac2 driver（macOS 原生測試）**
   - 僅在需要測原生功能時才導入
   - 成本最高，優先級最低

---

## 關鍵結論

- **日常開發**：GitHub Actions CI + Playwright（mock 版）就夠用
- **發布前完整驗證**：Docker Linux + WebdriverIO 是最完整的自動化路徑
- **macOS 原生功能**：Appium + Mac2 driver 是唯一選項，但通常不需要
- **Windows**：完全仰賴 GitHub Actions，本機無法用 Docker 測試
- **macOS E2E 的務實選擇**：Playwright 搭配真實的 `yarn tauri dev`，是設定成本與測試覆蓋率的最佳平衡點
