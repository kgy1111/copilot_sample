# Agent Architecture

agents/ 配下のエージェント定義を、保守しやすく揃えて管理するためのガイドです。

このディレクトリでは、長い手順書よりも次の 4 点を重視します。

- いつ使うか
- 何を担当するか
- どこまで委譲するか
- 出力で何を返すべきか

---

## エージェントの種類

### Orchestrators（オーケストレーター）

- [s.pm.agent.md](s.pm.agent.md): 開発タスク全体の進行管理
- [s.user-docs.agent.md](s.user-docs.agent.md): 日本語ユーザー文書の作成進行

オーケストレーターは、調査、設計、実装、レビューを自分で抱え込まず、適切なエージェントへ委譲して全体を完遂します。

### Subagents（専門エージェント）

- [s.researcher.agent.md](s.researcher.agent.md): 調査とコンテキスト収集
- [s.architect.agent.md](s.architect.agent.md): 設計方針、タスク分解、設計書更新
- [s.coder.agent.md](s.coder.agent.md): 実装、デバッグ、必要なテスト追加
- [s.dev-reviewer.agent.md](s.dev-reviewer.agent.md): 実装レビュー、承認可否、設計書同期
- [s.legend.agent.md](s.legend.agent.md): 事故りそうなポイントの直感チェック

Subagent は専門作業に集中します。内部専用にしたいものは frontmatter で `user-invocable: false` を付けます。

### Utility（ユーティリティ）

- [s.lunch.agent.md](s.lunch.agent.md): 設計雑談

### Personas（ペルソナ）

- [p.manager.agent.md](p.manager.agent.md): ROI、リスク、スケジュール
- [p.engineer.agent.md](p.engineer.agent.md): 実現可能性、複雑度、保守性
- [p.tech-reviewer.agent.md](p.tech-reviewer.agent.md): 抜け漏れ、失敗時挙動、一貫性
- [p.user-doc-writer-ja.agent.md](p.user-doc-writer-ja.agent.md): 日本語ユーザー文書
- [p.user-doc-writer-en.agent.md](p.user-doc-writer-en.agent.md): 英語ユーザー文書
- [p.user-doc-qa.agent.md](p.user-doc-qa.agent.md): 品質保証・サポート責任観点
- [p.user-doc-enduser-expert.agent.md](p.user-doc-enduser-expert.agent.md): ベテラン利用者観点
- [p.user-doc-enduser-mid.agent.md](p.user-doc-enduser-mid.agent.md): 中堅利用者観点

Persona は特定視点に絞って評価や執筆を行います。

---

## 推奨フロー

### 開発タスク

1. [s.pm.agent.md](s.pm.agent.md) が要件を整理する
2. 必要なら [s.researcher.agent.md](s.researcher.agent.md) と [s.architect.agent.md](s.architect.agent.md) に委譲する
3. [s.coder.agent.md](s.coder.agent.md) が実装する
4. [s.dev-reviewer.agent.md](s.dev-reviewer.agent.md) がレビューする
5. リスクが大きい場合だけ [s.legend.agent.md](s.legend.agent.md) を挟む

### ユーザー文書タスク

1. [s.user-docs.agent.md](s.user-docs.agent.md) が要件を整理する
2. 必要な調査を [s.researcher.agent.md](s.researcher.agent.md) に委譲する
3. [p.user-doc-writer-ja.agent.md](p.user-doc-writer-ja.agent.md) で日本語版を作る
4. [p.user-doc-qa.agent.md](p.user-doc-qa.agent.md)、[p.user-doc-enduser-expert.agent.md](p.user-doc-enduser-expert.agent.md)、[p.user-doc-enduser-mid.agent.md](p.user-doc-enduser-mid.agent.md) で並列レビューする
5. マスターが指摘を統合し、必要なら差し戻す
6. 最低 3 ラウンドのレビュー後に合格判定する

---

## Frontmatter の統一ルール

各 agent ファイルの frontmatter は、必要最小限で揃えます。

### 基本形

```yaml
---
description: 'Use when: このエージェントを使う場面を 1 文で書く。'
argument-hint: 'ユーザーや親エージェントが渡す依頼内容の例'
tools: ['read']
user-invocable: false
---
```

### ルール

- `description` は必ず `Use when:` で始める
- `description` は何をするかではなく、いつ使うかを先に書く
- 本文冒頭に役割説明がなければ、見出し直下に `XXXエージェント。YYYを行う。` の形で 1 文補う
- `argument-hint` は必要なエージェントだけ付ける
- `tools` は制限したい場合だけ付ける
- 内部専用エージェントには `user-invocable: false` を付ける
- YAML の文字列は原則クォートする

---

## 設計方針

- 長い例示より、判断基準と責務を明確にする
- エージェントごとの役割が重なりすぎないようにする
- 同じ説明を複数ファイルに重複させない
- 細かい出力テンプレートは、本当に品質に効くものだけ残す
- 迷ったら「この説明がないと誤動作しやすいか」で残すか決める

---

## このディレクトリを更新するときの目安

- README に大きな運用変更が必要か
- frontmatter が統一ルールから外れていないか
- 役割の重複が増えていないか
- 例示や補足が増えすぎて保守コストを上げていないか

必要なら各 agent ファイルを先に簡潔化し、そのあと README で全体像だけを説明します。