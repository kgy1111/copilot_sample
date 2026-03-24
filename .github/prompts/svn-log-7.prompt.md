---
agent: 'agent'
model: GPT-4.1
description: svn logによるこのブランチの1週間分の履歴
---

以下のコマンドをターミナルで実行し、このブランチの7日間分（今日から7日前から今日まで）の履歴を取得してください。

```powershell
svn log src/ -r "{$((Get-Date).AddDays(-7).ToString('yyyy-MM-dd'))}:HEAD" --stop-on-copy
```

