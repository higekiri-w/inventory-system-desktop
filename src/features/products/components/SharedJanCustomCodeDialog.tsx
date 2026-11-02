// src/features/products/components/SharedJanCustomCodeDialog.tsx
//
// UI-01b-D26: 新規登録で入れた JAN を他の商品が使っているときの確認ダイアログ（D-141）。
// shadcn AlertDialog を使用（先例: DiscontinueConfirmDialog の UI-01b-D13）。
// 1 段目は BIZ の message（BIZ-01-D11 が画面の語で作る。解析しない）、2・3 段目と button は 51 の文字どおり。
// 設計: docs/function-design/51-ui-product-form.md UI-01b-D26 / §7.5 Create 5

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

/** 51 UI-01b-D26 の 2 段目・3 段目と、接頭辞の無い部門の文（文字どおり） */
const SECOND_TEXT =
  "色やサイズの違う別の品なら、独自コードで登録できます。JAN はこの商品にも残ります。JAN が同じ商品はレジの売上でどれが売れたか見分けられないため、この商品と、JAN が同じ商品は全て「POS販売で在庫を減らす」が「減らさない」になります（在庫は棚卸しと手入力で合わせます）。「商品コードだけ同じ」の商品は変わりません。";
const THIRD_TEXT =
  "同じ品なら、新しく登録せずに、商品一覧で JAN を検索してその商品を直してください（廃番なら「表示に戻す」）。";
const noCustomCodeText = (departmentName: string) =>
  `部門「${departmentName}」は独自コードを振れません。独自コードを振れる部門を選び直してから、もう一度「登録する」を押してください。`;

export interface SharedJanCustomCodeDialogProps {
  open: boolean;
  /** BIZ の `JanInUse` の message（使っている商品の名指し） */
  message: string;
  departmentName: string;
  /** 選んでいる部門に `code_prefix` があるか（無ければ主の button を出さず部門を名指す） */
  canAssignCustomCode: boolean;
  isPending: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

/// open は parent state。閉じる経路は onConfirm の後の parent と onCancel（戻る / Esc）。
/// 送り直しの間は閉じない（送った結果を parent が受けて閉じる）。
export function SharedJanCustomCodeDialog({
  open,
  message,
  departmentName,
  canAssignCustomCode,
  isPending,
  onConfirm,
  onCancel,
}: SharedJanCustomCodeDialogProps) {
  return (
    <AlertDialog
      open={open}
      onOpenChange={(next) => {
        if (!next && !isPending) onCancel();
      }}
    >
      {/* 1 段目は使っている商品を省略せず並べるため長くなる。高さを viewport の中に収め、
          文だけを scroll し、題と button は scroll の外に固定する（先例: SupplierPickerDialog）。
          文の箱は名前の付いた scroll の region にし、tabIndex 0 で Tab・Shift+Tab で入れて
          矢印の key で読めるようにする（Radix の FocusScope は tabIndex >= 0 だけを Tab の循環に入れる）。
          focus は scroll-area と同じ ring で示す */}
      <AlertDialogContent className="flex max-h-[90vh] flex-col">
        <AlertDialogHeader className="flex min-h-0 flex-col">
          <AlertDialogTitle>この JAN はほかの商品が使っています</AlertDialogTitle>
          <AlertDialogDescription asChild>
            <div
              role="region"
              aria-label="使っている商品と変わる所"
              // eslint-disable-next-line jsx-a11y/no-noninteractive-tabindex -- scroll できる region を keyboard で読むため（WAI の scrollable region）
              tabIndex={0}
              className="min-h-0 space-y-2 overflow-y-auto rounded-sm outline-none focus-visible:ring-[3px] focus-visible:ring-ring"
            >
              <p>{message}</p>
              <p>{SECOND_TEXT}</p>
              <p>{THIRD_TEXT}</p>
              {canAssignCustomCode ? null : <p>{noCustomCodeText(departmentName)}</p>}
            </div>
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={isPending} onClick={onCancel}>
            戻る
          </AlertDialogCancel>
          {canAssignCustomCode ? (
            <AlertDialogAction
              disabled={isPending}
              onClick={(event) => {
                // 送り直しの結果が出るまでダイアログを開いたままにする（Radix の自動の close を止める）
                event.preventDefault();
                onConfirm();
              }}
            >
              独自コードで登録する
            </AlertDialogAction>
          ) : null}
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
