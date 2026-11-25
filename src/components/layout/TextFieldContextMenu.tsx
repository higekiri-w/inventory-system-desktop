import { useEffect, useRef, useState, type ReactNode } from "react";
import { ContextMenu } from "radix-ui";
import { toast } from "sonner";

/// UI-12 / UI-CTX-D1〜D4: 文字の欄の右クリックの自前の小さなメニュー（切り取り・コピー・貼り付け・すべて選択）。
/// 欄の外では何も出さず、WebView2 の既定のメニューは document の contextmenu で常に止める。
/// 設計: docs/UI_TECH_STACK.md §6.13、docs/decision-log.md D-171

type TextField = HTMLInputElement | HTMLTextAreaElement;
type Action = "cut" | "copy" | "paste" | "selectAll";

// UI-CTX-D2: input の型が未指定（"text"）・text・search・number だけが文字の欄。date・month・time ほかは欄の外
const TEXT_INPUT_TYPES = new Set(["text", "search", "number"]);

// ponytail: 文字の欄は HTMLTextAreaElement と D2 の型の HTMLInputElement だけ。contentEditable を使う画面が出たらその判定を足す
function textFieldOf(target: EventTarget): TextField | null {
  const isField =
    target instanceof HTMLTextAreaElement ||
    (target instanceof HTMLInputElement && TEXT_INPUT_TYPES.has(target.type));
  return isField && !target.disabled ? target : null;
}

function hasSelection(field: TextField): boolean {
  // 数字の欄は selectionStart が null なので getSelection() で読む（P2 の実測、repo 外の回答台帳 TD-913）
  if (field.selectionStart === null) return (window.getSelection()?.toString() ?? "") !== "";
  return field.selectionStart !== field.selectionEnd;
}

async function paste(field: TextField) {
  let text: string;
  try {
    text = await navigator.clipboard.readText();
  } catch {
    toast.error("貼り付けられませんでした。Ctrl+V で貼り付けてください。", {
      id: "text-field-paste-error",
      duration: 8000,
    });
    return;
  }
  // 読みの間に別の欄へ移っていたら、どちらの欄も変えない
  if (text !== "" && document.activeElement === field) {
    // insertText は input event を出すので React の onChange・maxLength・元に戻すの履歴に届く
    // eslint-disable-next-line @typescript-eslint/no-deprecated -- 欄の state・元に戻すの履歴に届く代わりの標準 API が無い（UI-CTX-D3）
    document.execCommand("insertText", false, text);
  }
}

const ITEMS: { action: Action; label: string }[] = [
  { action: "cut", label: "切り取り" },
  { action: "copy", label: "コピー" },
  { action: "paste", label: "貼り付け" },
  { action: "selectAll", label: "すべて選択" },
];

export function TextFieldContextMenu({ children }: { children: ReactNode }) {
  const fieldRef = useRef<TextField | null>(null);
  const actionRef = useRef<Action | null>(null);
  const composingRef = useRef(false);
  const [field, setField] = useState({ readOnly: false, selected: false });

  useEffect(() => {
    // bubble で付ける（capture だと Radix の Trigger が先に defaultPrevented を見て開かない）
    const block = (event: MouseEvent) => {
      event.preventDefault();
    };
    document.addEventListener("contextmenu", block);
    return () => {
      document.removeEventListener("contextmenu", block);
    };
  }, []);

  const disabled: Record<Action, boolean> = {
    cut: field.readOnly || !field.selected,
    copy: !field.selected,
    paste: field.readOnly,
    selectAll: false,
  };

  return (
    <ContextMenu.Root>
      <ContextMenu.Trigger
        asChild
        onContextMenu={(event) => {
          const target = textFieldOf(event.target);
          if (!target || composingRef.current) {
            // Radix の open を飛ばす（defaultPrevented なら開かない）
            event.preventDefault();
            return;
          }
          fieldRef.current = target;
          setField({ readOnly: target.readOnly, selected: hasSelection(target) });
        }}
      >
        <div
          className="contents"
          onCompositionStart={() => (composingRef.current = true)}
          onCompositionEnd={() => (composingRef.current = false)}
        >
          {children}
        </div>
      </ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Content
          className="z-50 max-h-(--radix-context-menu-content-available-height) min-w-[8rem] origin-(--radix-context-menu-content-transform-origin) overflow-x-hidden overflow-y-auto rounded-md border bg-popover p-1 text-popover-foreground shadow-md data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95"
          onCloseAutoFocus={(event) => {
            // Radix は閉じるとき欄に select() をかけるので止め、focus だけを戻して控えた操作を打つ
            event.preventDefault();
            const target = fieldRef.current;
            const action = actionRef.current;
            actionRef.current = null;
            if (!target) return;
            target.focus();
            if (action === "selectAll") target.select();
            else if (action === "paste") void paste(target);
            // eslint-disable-next-line @typescript-eslint/no-deprecated -- 欄の選択の切り取り・コピーの代わりの標準 API が無い（UI-CTX-D3）
            else if (action) document.execCommand(action);
          }}
        >
          {ITEMS.map(({ action, label }) => (
            <ContextMenu.Item
              key={action}
              disabled={disabled[action]}
              // 開いている間は focus が menuitem に引き戻されるので、ここでは控えるだけ
              onSelect={() => (actionRef.current = action)}
              className="relative flex w-full cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50"
            >
              {label}
            </ContextMenu.Item>
          ))}
        </ContextMenu.Content>
      </ContextMenu.Portal>
    </ContextMenu.Root>
  );
}
