# Rust 言語設計解説 — myprotocol プロジェクト

このドキュメントは `myprotocol` プロジェクトで使われている **Rust の言語設計・デザインパターン** を解説し、  
他の言語（Go / TypeScript / Python / Java / C# など）でも応用できる設計知識を伝えることを目的とします。  
通信プロトコルの仕様そのものは扱いません。

---

## 目次

1. [プロジェクト全体の構造設計](#1-プロジェクト全体の構造設計)
2. [Enum Dispatch（静的ディスパッチ）](#2-enum-dispatch静的ディスパッチ)
3. [エラー型設計 — thiserror と Result](#3-エラー型設計--thiserror-と-result)
4. [Option による状態管理](#4-option-による状態管理)
5. [非同期設計 — async/await と spawn_blocking](#5-非同期設計--asyncawait-と-spawn_blocking)
6. [共通設定の継承パターン — Default トレイト](#6-共通設定の継承パターン--default-トレイト)
7. [スレッド安全な共有状態 — Arc と Mutex](#7-スレッド安全な共有状態--arc-と-mutex)
8. [デザインパターン早見表](#8-デザインパターン早見表)

---

## 1. プロジェクト全体の構造設計

### Cargo ワークスペース（モノレポ）

```
myprotocol/
├── myprotocol-core/    ← 共通型・エラー・フレーム定義
├── myprotocol-tcp/     ← TCP実装（core に依存）
├── myprotocol-usb/     ← USB実装（core に依存）
└── myprotocol-client/  ← 統合 API（tcp・usb・core に依存）
```

**設計意図：依存方向の一方向化**

```
core ←── tcp ──┐
core ←── usb ──┴──→ client
```

`core` は誰にも依存しない。`client` だけがすべてを知る。  
これにより `tcp` と `usb` は互いに独立しており、片方を差し替えてもコンパイルエラーが起きない。

> **他言語への応用**  
> - Go / Node.js / Python のモノレポでも「共通パッケージ」→「トランスポート実装」→「統合クライアント」という依存の向きを揃えると、テスト・差し替えが容易になる。  
> - 依存は常に「安定したもの（抽象・型定義）」に向けるのが原則（Dependency Inversion Principle）。

---

## 2. Enum Dispatch（静的ディスパッチ）

### コード

```rust
// myprotocol-client/src/lib.rs
enum TransportKind {
    Tcp(TcpTransport),
    Usb(UsbTransport),
}

pub struct Client {
    transport: TransportKind,
}

impl Client {
    pub async fn connect(&mut self) -> Result<(), ProtocolError> {
        match &mut self.transport {
            TransportKind::Tcp(t) => t.connect().await,
            TransportKind::Usb(t) => t.connect().await,
        }
    }
    // send / close も同様
}
```

### なぜトレイトオブジェクト（`Box<dyn Transport>`）ではなく enum なのか

| 比較軸 | `Box<dyn Trait>`（動的ディスパッチ） | `enum`（静的ディスパッチ） |
|---|---|---|
| ランタイムオーバーヘッド | vtable 経由の間接呼び出し | ゼロコスト（コンパイル時解決） |
| ヒープアロケーション | 必要（`Box` で包む） | 不要（スタック上に収まる） |
| async との相性 | `async_trait` クレートが必要（内部で `Box<dyn Future>` を生成） | ネイティブ `async fn` がそのまま使える |
| 拡張性 | クレート外から実装を追加できる | enum のバリアントを変更する必要がある |
| 型消去 | される（実行時に型が失われる） | されない（コンパイル時に型が残る） |

このプロジェクトでは「トランスポートの種類は TCP と USB の 2 択で固定」という前提があるため、  
`enum` による静的ディスパッチが適切。

### デザインパターンとしての解釈

これは **Strategy パターン** の Rust 版実装です。

```
Strategy（インターフェース）  ← 今回は enum で代替
  ConcreteStrategyA = TcpTransport
  ConcreteStrategyB = UsbTransport
Context = Client
```

> **他言語への応用**  
> - Java / C# / TypeScript では `interface Transport` を定義して Strategy パターンを実装するのが一般的。  
> - Go では `interface` の duck typing が Rust の `dyn Trait` に相当する（ただし Go の interface はゼロコスト）。  
> - Rust 特有の点として「async + 動的ディスパッチ」の組み合わせにコストがかかるため、種類が有限・固定なら enum dispatch が慣用句となっている。

---

## 3. エラー型設計 — thiserror と Result

### コード

```rust
// myprotocol-core/src/error.rs
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("connect timeout")]
    ConnectTimeout,

    #[error("not connected")]
    NotConnected,

    #[error("transport error: {0}")]
    Transport(Box<dyn std::error::Error + Send + Sync + 'static>),
    // ...
}
```

### 設計のポイント

**1. エラーの列挙化（Enum Error）**

発生しうるエラーをすべて `enum` のバリアントとして列挙する。  
呼び出し元はパターンマッチングで網羅的に扱えるため、「どんなエラーが起きるか」がコンパイル時に保証される。

**2. `#[from]` による自動変換**

`std::io::Error` は `#[from]` マクロにより `ProtocolError::Io` に自動変換される。  
これにより `?` 演算子（後述）が機能する。

**3. `Box<dyn Error>` でライブラリ非依存の汎用エラーラッピング**

`Transport(Box<dyn Error + Send + Sync>)` バリアントは、`rusb` など外部クレートのエラーを包む「ワイルドカード」として機能する。  
クレート外のエラー型を自型に `From` 実装できない場合（孤児ルール）の回避策。

**4. `?` 演算子（Railway-Oriented Programming）**

```rust
// ? は Result が Err なら即 return する
let stream = TcpStream::connect(&addr).await?;
//                                         ↑ Err の場合は関数全体から Err を返す
```

これにより「成功パス」だけを直線的に書ける。エラー分岐がコードに埋もれない。

> **他言語への応用**  
> - Go のエラー処理 `if err != nil { return err }` は `?` 演算子の手動版。  
> - Kotlin の `Result<T>` / `runCatching`、Swift の `throws`、F# の `Result` も同じ思想。  
> - 「エラー型を enum で列挙する」パターンは TypeScript の Union Type（`type Error = TimeoutError | NotFoundError`）でも実現できる。

---

## 4. Option による状態管理

### コード

```rust
pub struct TcpTransport {
    config: TcpConfig,
    stream: Option<TcpStream>,  // ← 未接続は None、接続済みは Some
}
```

```rust
// 接続後に値をセット
self.stream = Some(stream);

// 使用時は ok_or で None をエラーに変換
let stream = self.stream.as_mut().ok_or(ProtocolError::NotConnected)?;

// close 時は take() で所有権を奪いつつ None に戻す
let mut stream = match self.stream.take() {
    Some(s) => s,
    None => return Ok(()),  // 二重 close は無視
};
```

### 設計のポイント

**`Option<T>` は Null Safety の型レベル保証**

Rust には `null` / `nil` がない。「値がないかもしれない」状態は `Option<T>` で明示する。  
コンパイラが「None の可能性を考慮せずに値を使おうとした」コードをエラーにする。

**`take()` によるムーブ＋リセット**

```rust
self.stream.take()  // stream の所有権を奪い、self.stream を None にする（アトミック操作）
```

この 1 行で「ストリームを取り出しつつフィールドを未接続状態に戻す」という 2 つの操作が完結する。  
二重 close・二重 free のバグを型システムで防げる。

> **他言語への応用**  
> - Java の `Optional<T>`、Kotlin の `T?`、Swift の `Optional<T>`、C# の `T?` が同様の概念。  
> - これらを使う場合も「接続済み / 未接続」を `null` で管理するのではなく、State パターン or `Option` 型で管理すると、扱い忘れをコンパイラ（型チェッカー）が検出できる。

---

## 5. 非同期設計 — async/await と spawn_blocking

### TCP の場合（純粋非同期）

```rust
pub async fn connect(&mut self) -> Result<(), ProtocolError> {
    let stream = time::timeout(
        self.config.timeout,
        TcpStream::connect(&addr)
    )
    .await
    .map_err(|_| ProtocolError::ConnectTimeout)?
    .map_err(ProtocolError::Io)?;

    self.stream = Some(stream);
    Ok(())
}
```

`tokio::time::timeout` でタイムアウトをラップ → `map_err` でエラー型を変換。  
この組み合わせで「タイムアウト付き非同期 I/O」が 5 行で書ける。

### USB の場合（同期ライブラリとの橋渡し）

```rust
pub async fn connect(&mut self) -> Result<(), ProtocolError> {
    let inner = tokio::task::spawn_blocking(move || {
        // rusb はブロッキング API → 専用スレッドで実行
        let handle = rusb::open_device_with_vid_pid(vid, pid)...;
        // ... 同期処理 ...
        Ok(UsbInner { ... })
    })
    .await
    .map_err(|e| ProtocolError::Transport(Box::new(e)))??;

    self.inner = Some(inner);
    Ok(())
}
```

**設計のポイント：`spawn_blocking` によるブロッキング I/O の隔離**

`tokio` の非同期ランタイムは少数のスレッドで多数のタスクを処理する。  
そこにブロッキング処理（`rusb` のような同期 API）を混ぜるとスレッドが占有されてしまい、他のタスクが止まる。

`spawn_blocking` はブロッキング処理を専用のスレッドプールに追い出す。  
これにより「非同期ランタイムをブロックしない」原則を守れる。

```
tokio スレッドプール        blocking スレッドプール
    [async task]      →  spawn_blocking → [USB同期処理]
    [async task]      ←     JoinHandle（Future として await）
```

> **他言語への応用**  
> - Go の `goroutine` でも「CGo 呼び出しや重い同期処理は goroutine 内で実行する」が同じ原則。  
> - Node.js では `worker_threads` や `libuv` のスレッドプールへのオフロードが対応する。  
> - Python `asyncio` では `loop.run_in_executor()` が同様の役割。  
> - **核心原則**: 非同期ランタイムのイベントループ（スレッド）をブロックしない。重い同期処理は別スレッドに隔離する。

---

## 6. 共通設定の継承パターン — Default トレイト

### コード

```rust
// myprotocol-core/src/config.rs
#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub timeout: Duration,
    pub retry_count: u32,
    pub close_timeout: Duration,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            retry_count: 0,
            close_timeout: Duration::from_secs(5),
        }
    }
}
```

```rust
// myprotocol-tcp/src/config.rs
pub struct TcpConfig {
    pub address: String,
    pub port: u16,
    pub timeout: Duration,      // ← TransportConfig と同じフィールド
    pub retry_count: u32,       // ← TransportConfig と同じフィールド
    pub close_timeout: Duration,// ← TransportConfig と同じフィールド
    pub _tls: Option<()>,
}
```

### 利用側での「部分オーバーライド」パターン

```rust
let config = TcpConfig {
    address: "192.168.1.100".to_string(),
    port: 8080,
    ..Default::default()  // ← 残りはデフォルト値
};
```

`..Default::default()` により、指定しなかったフィールドはデフォルト値が自動で埋まる。  
これは **Builder パターン** をボイラープレートなしで実現するイディオム。

> **他言語への応用**  
> - Python のデフォルト引数 `def connect(timeout=30, retry=0)` や `dataclass` の `field(default=...)` に対応。  
> - TypeScript / JavaScript の `{ ...defaults, ...overrides }` スプレッド構文が近い。  
> - Java / C# では Builder クラスを明示的に作るか、名前付き引数で代替する。  
> - 「デフォルト値を中央定義し、必要な部分だけオーバーライド」という思想は Open-Closed Principle に沿っている。

---

## 7. スレッド安全な共有状態 — Arc と Mutex

### コード

```rust
// myprotocol-usb/src/transport.rs
struct UsbInner {
    handle: Arc<Mutex<DeviceHandle<GlobalContext>>>,
    interface_num: u8,
    in_ep: u8,
    out_ep: u8,
}
```

```rust
// send 内
let handle = Arc::clone(&inner.handle);     // 参照カウンタをインクリメント
let jh = tokio::task::spawn_blocking(move || {
    let h = handle.lock()                   // 排他ロック取得
        .map_err(|_| ProtocolError::Transport("mutex poisoned".into()))?;
    h.write_bulk(...)?;
    // ...
});
```

### 設計のポイント

| 型 | 役割 |
|---|---|
| `Arc<T>` | **A**tomic **R**eference **C**ounted — 複数スレッドで所有権を共有 |
| `Mutex<T>` | 排他ロック — 同時アクセスを防ぐ |
| `Arc<Mutex<T>>` | スレッド間で共有しつつ、アクセスは排他制御 |

**Mutex Poisoning への対応**

```rust
handle.lock().map_err(|_| ProtocolError::Transport("mutex poisoned".into()))
```

Rust の `Mutex` は、ロック保持中にスレッドがパニックすると「汚染（poisoned）」状態になる。  
`lock()` は `PoisonError` を返すため、`.map_err` でドメインエラーに変換している。  
これにより「汚染された状態で処理続行」というバグを防げる。

> **他言語への応用**  
> - Java の `synchronized` / `ReentrantLock`、Go の `sync.Mutex`、Python の `threading.Lock` が対応。  
> - `Arc` に相当するのは、Go の参照型（すべてがポインタ経由）、Java/C# の参照型（GC 管理）。  
> - Rust の特徴は「Arc を使わないと別スレッドに渡せない」という **コンパイル時保証**。Java/Go は実行時エラーで気づく。

---

## 8. デザインパターン早見表

| デザインパターン | Rust での実現方法 | コード箇所 |
|---|---|---|
| **Facade（窓口）** | `Client` 構造体が TCP/USB の詳細を隠蔽 | `myprotocol-client/src/lib.rs` |
| **Strategy（戦略）** | `enum TransportKind` による静的ディスパッチ | `myprotocol-client/src/lib.rs` |
| **Builder（ビルダー）** | `Default` トレイト + `..Default::default()` | 各 `config.rs` |
| **Null Object（ヌルオブジェクト）** | `Option<TcpStream>` で未接続状態を表現 | `myprotocol-tcp/src/transport.rs` |
| **Template Method** | `connect / send / close` の共通シグネチャ | 各 `transport.rs` |
| **Adapter** | `spawn_blocking` で同期 API を非同期に変換 | `myprotocol-usb/src/transport.rs` |
| **Layered Architecture** | core → transport → client の一方向依存 | `Cargo.toml` ワークスペース構成 |

---

## まとめ：他言語でも活かせる設計原則

1. **依存は一方向に** — 共通型・エラーを中央クレートに置き、実装クレートから参照する  
2. **エラーを型で表現する** — 発生しうるエラーを enum で列挙し、呼び出し元が網羅的に扱える  
3. **null は型で制御する** — `Option<T>` や `Optional<T>` で「値がない」状態を明示的に扱う  
4. **非同期ランタイムをブロックしない** — 重い同期処理は別スレッドに隔離する  
5. **有限・固定の種類は enum で** — 拡張不要な戦略の切り替えは動的ディスパッチより enum の方がシンプル  
6. **デフォルト値を中央定義する** — 必要な部分だけオーバーライドできる設計で利用側の記述量を減らす
