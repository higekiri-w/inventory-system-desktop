# Plan Packet: <件名>（PR の件名と同じ）

R3/R4 の packet の最小形（D-145 (10)）。規則は `docs/DEV_WORKFLOW.md` の Plan Packet Rules と Workflow State。R0〜R2 は packet を作らず PR body の 5 行で進める。前文に wave と lane を書いてよい（任意）。

## Workflow State

- Risk: <R3|R4>
- Plan Commit: pending
- Amendments: none

`Plan Commit`・`Amendments` は `scripts/plan-approve.sh` が書く（手で写さない）。Final Review Minimum（2）と Human Gate は helper が Risk・classifier・`## L3` 節から導く。明示の承認を足すときだけ `- Human Gate: ready,merge,r4` のように 1 行足す（導いた値から落とす向きは受けない）。

## Risk

Risk: <R3|R4>

Reason:
<理由 1 文。出所つき>

## Goal

Goal Invariant:

### 最小完了条件

- <利用者に見える最小の完了条件>

### 失敗定義

- <この change が失敗したと言える結果>

### 非目的

- <広げない・最適化しないもの>

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- <範囲。触る file と変更の目的>

対象を使う呼出し側・隣接 test・helper / mock・生成物まで確かめて列挙する。「関連 file 全般」を許可範囲にしない。Non-scope は必要なら同じ節に書く。

## Acceptance Criteria

- <観測できる成功の条件。command・file・test 名・出力・exit code・`WARN` / `ERROR` の期待を含める>

- AC は要る振舞いを定め、対象 test があれば全 PASS を求める。test の本数・文字列の出現数を完成の代理にしない。
- 数値の baseline は対象版・測定 command・出力を併記するか `未実測`。
- 削除の検査は旧い表現で一致し、新しい許容の表現で一致しないことを確かめる。

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules。当たらない箇条は 1 行の理由で閉じる。

- 引用する設計正本（節まで）:
- 必要な設計成果物（existing sufficient / updated in this PR / deferred）:
- plan にしかない durable な判断の昇格先:
- 前提・制約と延期した design gap:
- 絶対保証（cannot happen / always happens）の例外と抜け道:
- 判定（ready / not ready）と理由:

## Contract Ledger

触る設計正本の節の契約・設計判断の ID を全部行にする（行の欠けは計画の review の blocker）。行を書く前に隣の契約の sweep で Scope が触りうる契約を足すか除外を書く。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| <ID> | <節> | <Scope の項> | <test 名> | <L3 の項か非対象> |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| <契約> | <防ぐ失敗> | <unit / CLI / integration> | <test 名> | <どの変更で red になるか> |

runtime の R3 は状態の遷移（initial / pending / success / invalidate / refetch / revisit / restart / failure / retry）・negative・mutation の注入（実装に注入して red を確かめる）を含める。

## L3

<!-- manual があるときだけ置く（節があること = manual）。無ければ節ごと消す。 -->

- <画面> / <到達手順> / <観測できる合格基準>（`docs/DEV_WORKFLOW.md` L3 の L3 Eligibility と fixture の実 encoding）

## Recovery

<!-- R4 だけ。R3 は節ごと消す。 -->

- <失敗したときの戻し方と、戻せる範囲>

## Data Safety

<!-- R4 だけ（R3 は計画の review の発注の固定の問いで確かめる）。R3 は節ごと消す。 -->

- <commit しないもの・local だけの path・合成だけの path>
