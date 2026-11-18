# モデル差分メモ

公式確認日: 2026-11-18（GPT-6.2 の節。Claude Code の節は 2026-09-25）。実モデルが明示された場合だけ該当する補助を使う。設定によるモデル選択や、repositoryでの性能改善の実測結果ではない。

## Effort の選定

Claude 側・Codex 側とも、effort の値と決定日は [AGENT_OPERATING_MANUAL の座組](../AGENT_OPERATING_MANUAL.md#座組) の effort 列が持ち、ここには書かない。effort の選び方の owner 方針（2026-09-14、総 token で効率を判断する）は同じ座組の表の下の注記「effort の選び方」を読む。

## GPT-6.2（Astra・Sol）

`gpt-6.2-astra`・`gpt-6.2-sol`。reasoning effort は `low`・`medium`・`high`・`xhigh`・`max`（[Models](https://developers.openai.com/api/docs/models)）。

両方に当たる助言（[Prompting GPT-6.2 in Codex](https://learn.chatgpt.com/guides/prompting-gpt-6-2-astra)。この頁は、助言が GPT-6.2 Astra と GPT-6.2 Sol の両方に当たると書く）: 完了の条件と、念入りにする所を書く。高い effort は、長い作業の最後の確かめまで粘るのを助ける。effort の目安は、ざっと見るなら medium、1 つの疑わしい原因を複数の system にまたいで追うなら high、複数の原因が絡みうるなら xhigh。発注の effort の値は [座組](../AGENT_OPERATING_MANUAL.md#座組) が正本で、この目安で上書きしない。

[Using GPT-6.2](https://developers.openai.com/api/docs/guides/latest-model) の Prompting best practices の節は、例が GPT-6.2 Astra と Sol で見た振舞いに当たると書く。同じ節が GPT-6.2 Astra の名で書く振舞いには次がある: 既定で前の世代より簡潔に書く、意図が曖昧だと次の手を提案して確認を待つことがある。同じ節の助言: 移行のときは「簡潔に」の類の指示を見直す。長い作業では可逆な前提を置いて明記し、進める余地を与える。長い作業では推論の途中や他の agent の略号を最終の答えに持ち込むことがある（model を名指さない文）ので、それだけで読める答えを求める。

## 未知のモデル

共通契約を使う。共通の短い依頼で不足が観測された箇所にだけ補助を追加する。ある model 向けの傾向を別の model へ自動継承しない。effort の選び方を model 間で機械的に揃えない。過去の effort の比べはその model・その設定での観測に限り、通常運用への推奨や品質保証にしない。

## Claude Code との共有境界

共有workflowの契約は同じ。Claudeのモデル補助は `CLAUDE.md` とAnthropicの当該モデル向けガイドを参照する。CLIの設定とAPI専用パラメータを混同しない。

根拠: [Claude Code best practices](https://code.claude.com/docs/en/best-practices)、[Claude model-specific prompting](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices)。
