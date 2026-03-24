# リポジトリ インデックス

## プロジェクト概要

**XXXsoftwareXXXX  開発リポジトリ。**
XXXsoftwareXXXX分析アプリ（AcqEngine.exe）のGUM層に  オートサンプラをつなぐドライバ実装が主目的。  
アーキテクチャは `Client(LSSAnalysis.exe) ←IPC→ Server(AcqEngine.exe) → GUMM → GUM_XXXYYYYAOC6k2 → GUMtoDriver → Mimic Driver → 実機`。

---

## アーキテクチャ全体図

[docs/content/jp/lss/architecture.md](../docs/content/jp/lss/architecture.md) — LSS全体のクライアント/サーバー構成・GUM層の位置を把握するための最重要資料。

---

## 仕様書・設計書 (`docs/content/jp/`)

| 種別 | パス | 用途 |
|------|------|------|
| 機能仕様書 | [spec/](../docs/content/jp/spec/) | 機能仕様。新規機能・変更時にまず参照 |
| 詳細設計書 | [design/](../docs/content/jp/design/) | 実装設計の骨格。[アーキテクチャ](../docs/content/jp/design/20_GUM_Architecture.md)・[GUMtoDriver設計](../docs/content/jp/design/21_GUMtoDriver_Design.md)・[実装状況](../docs/content/jp/design/23_GUM_Implementation_Status.md) |
| LSSアーキテクチャ | [lss/](../docs/content/jp/lss/) | LSS全体設計。GUMインタフェース仕様群: [GUM_IF](../docs/content/jp/lss/GUM/GUM_IF.md) / [IGUM_Config](../docs/content/jp/lss/GUM/IGUM_Config.md) / [IGUM_Method](../docs/content/jp/lss/GUM/IGUM_Method.md) / [IGUM_CommunicationEngine](../docs/content/jp/lss/GUM/IGUM_CommunicationEngine.md) |
| Mimicインタフェース | [mimic/](../docs/content/jp/mimic/) | ドライバが準拠すべきMimic IF仕様（ReadOnly参照） |

---

## ソースコード (`src/SOURCE_INST/SOURCE_DLL/GUM/`)

### 本プロジェクトの実装対象

| モジュール | パス | 説明 |
|-----------|------|------|
|XXX|XXX|XXX|

### 参考実装

| モジュール | パス | 説明 |
|-----------|------|------|
| GUM_XXXYYYYAOC5k6k | [GUM_XXXYYYYAOC5k6k/](../src/SOURCE_INST/SOURCE_DLL/GUM/GUM_XXXYYYYAOC5k6k/) | 旧XXXX（参考） |
| GUM_XXXYYYYGC2 | [GUM_XXXYYYYGC2/](../src/SOURCE_INST/SOURCE_DLL/GUM/GUM_XXXYYYYGC2/) | 他装置GUMの設計参考 |

---

## 実装の原則

- GUM_XXXYYYYAOC6k2 は **C++/CLI呼び出しに徹する**。ビジネスロジックは GUMtoDriver 側に置く。
- 仕様変更時は `docs/content/jp/spec/` → `docs/content/jp/design/` の順に更新。
- 新規作業は必ず `/tasks/YYMMDD_<作業名>.md` の作業チケットを作成して管理する。
