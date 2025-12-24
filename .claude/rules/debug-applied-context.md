# Debug Applied Context

応答の最後に**この応答で実際に参照・適用した**指示群を出力する。

## 判断基準

- Commands: 今回のリクエストで実行されたスラッシュコマンド
- Rules: 今回の作業に関連して**意識的に適用した**ルール
- Skills: 今回の作業で呼び出されたスキル

## 出力形式

```
---  
Applied Context:
- Commands: [実行したコマンド]
- Rules: [適用したルール名]
- Skills: [呼び出したスキル]
---
```

## 出力例

```
---  
Applied Context:
- Commands: (none)
- Rules: documentation
- Skills: (none)
---
```

## 注意

- 読み込まれているが適用していないルールは含めない
