---
agent: 'agent'
model: GPT-4.1
description: svn logによるこのブランチの全履歴
---

以下のコマンドをターミナルで実行し、このブランチの全履歴を取得してください。

```
svn log src/ -r 1:HEAD --stop-on-copy 
```