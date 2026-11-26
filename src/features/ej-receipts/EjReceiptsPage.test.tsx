// UI-09c レシートの控え画面（SCREEN_DESIGN UI-09c-D2〜D5、80）。Test Plan U3〜U11
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useBlocker } from "@tanstack/react-router";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { toast } from "sonner";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import { commands } from "@/lib/bindings";
import type {
  EjImportResult,
  EjRecordDay,
  EjRecordDetail,
  EjRecordRef,
  EjRecordSummary,
} from "@/lib/bindings";
import { createAppQueryClient } from "@/lib/query-client";
import { queryKeys } from "@/lib/query-keys";
import { getWindowCloseGuardKinds } from "@/lib/window-close-guard";
import { EjReceiptsPage } from "./EjReceiptsPage";
import type { EjReceiptsSearch } from "./types";

vi.mock("@tanstack/react-router", () => ({
  Link: ({ children, to }: { children: ReactNode; to: string }) => <a href={to}>{children}</a>,
  useBlocker: vi.fn(),
}));

vi.mock("sonner", () => ({ toast: { error: vi.fn(), success: vi.fn() } }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/plugin-fs", () => ({ readFile: vi.fn() }));

vi.mock("@/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/bindings")>();
  return {
    ...actual,
    commands: {
      listEjRecords: vi.fn(),
      findEjRecordsByNo: vi.fn(),
      getEjRecord: vi.fn(),
      importEjFiles: vi.fn(),
    },
  };
});

const mockList = vi.mocked(commands.listEjRecords);
const mockFind = vi.mocked(commands.findEjRecordsByNo);
const mockGet = vi.mocked(commands.getEjRecord);
const mockImport = vi.mocked(commands.importEjFiles);
// eslint-disable-next-line @typescript-eslint/no-deprecated -- 共有の hook の useBlocker の配線を spy する
const mockUseBlocker = vi.mocked(useBlocker);

function summary(
  id: number,
  no: string,
  overrides: Partial<EjRecordSummary> = {},
): EjRecordSummary {
  return {
    id,
    machine_prefix: "0001",
    record_no: no,
    printed_at: "2026-11-20 10:05",
    kind: "sale",
    mode: "normal",
    total_amount: 1234,
    ...overrides,
  };
}

function day(overrides: Partial<EjRecordDay> = {}): EjRecordDay {
  return {
    date: "2026-11-20",
    prev_date: "2026-11-19",
    next_date: null,
    records: [summary(1, "000106"), summary(2, "000107", { kind: "sale", mode: "return" })],
    truncated: false,
    gaps: [],
    ...overrides,
  };
}

function detail(id: number, no: string, bodyText = `body ${no}`): EjRecordDetail {
  return { summary: summary(id, no), body_text: bodyText };
}

function ejResult(overrides: Partial<EjImportResult> = {}): EjImportResult {
  return {
    inserted_count: 2,
    already_count: 0,
    conflict_count: 0,
    conflicts: [],
    held_count: 0,
    unreadable_files: [],
    gaps: [],
    ...overrides,
  };
}

const ok = <T,>(data: T) => ({ status: "ok" as const, data });
const err = (kind: "validation" | "internal" | "not_found", message: string) => ({
  status: "error" as const,
  error: { kind, message, field: null, error_id: null, stocktake_recovery: null },
});

/** 日付 → その日の応答（null は最新の日の既定） */
function serveDays(days: Record<string, EjRecordDay>) {
  mockList.mockImplementation((date) => Promise.resolve(ok(days[date ?? "latest"])));
}

function renderPage(
  initial: EjReceiptsSearch = {},
  queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  }),
) {
  const onSearchChange = vi.fn();
  // サイドバー・戻るのように、moveTo を通らずに search を外から替える
  let setSearchFromOutside!: (next: EjReceiptsSearch) => void;
  function Harness() {
    const [search, setSearch] = useState<EjReceiptsSearch>(initial);
    setSearchFromOutside = setSearch;
    return (
      <EjReceiptsPage
        search={search}
        onSearchChange={(next, options) => {
          onSearchChange(next, options);
          setSearch(next);
        }}
      />
    );
  }
  render(
    <QueryClientProvider client={queryClient}>
      <Harness />
    </QueryClientProvider>,
  );
  return {
    onSearchChange,
    queryClient,
    setSearch: (next: EjReceiptsSearch) => {
      act(() => {
        setSearchFromOutside(next);
      });
    },
  };
}

type DayResponse = Awaited<ReturnType<typeof commands.listEjRecords>>;

function rowOf(no: string): HTMLElement {
  return screen.getByRole("row", { name: new RegExp(no) });
}

beforeEach(() => {
  vi.clearAllMocks();
  mockGet.mockImplementation((id) => Promise.resolve(ok(detail(id, `id${String(id)}`))));
  vi.mocked(readFile).mockResolvedValue(new Uint8Array([1, 2, 3]));
});

describe("EjReceiptsPage UI-09c REQ-908", () => {
  it("U3: 読込み中は空の文を出さず、成功で最新の日の表（時刻・番号・種類・合計）を出す", async () => {
    let resolveDay!: (value: Awaited<ReturnType<typeof commands.listEjRecords>>) => void;
    mockList.mockReturnValueOnce(
      new Promise((resolve) => {
        resolveDay = resolve;
      }),
    );
    renderPage();
    expect(screen.getByRole("status")).toHaveTextContent("読み込んでいます");
    expect(screen.queryByText(/レシートの控えはまだありません/)).not.toBeInTheDocument();
    expect(screen.queryByText("この日の控えはありません")).not.toBeInTheDocument();
    expect(mockList).toHaveBeenCalledWith(null);

    act(() => {
      resolveDay(ok(day()));
    });
    const table = await screen.findByRole("grid", { name: "控えの一覧" });
    const cells = within(rowOf("000106"))
      .getAllByRole("cell")
      .map((cell) => cell.textContent);
    expect(cells).toEqual(["10:05", "000106", "取引", "1,234円"]);
    expect(within(table).getByText("返品")).toBeInTheDocument();
  });

  it("U4: 前の日・次の日は記録のある隣の日へ移り、無ければ押せない。日付の入力で移り、記録の無い日は 80 の文", async () => {
    serveDays({
      latest: day(),
      "2026-11-19": day({
        date: "2026-11-19",
        prev_date: null,
        next_date: "2026-11-20",
        records: [summary(9, "000101")],
      }),
      "2026-11-10": day({
        date: "2026-11-10",
        records: [],
        prev_date: null,
        next_date: "2026-11-19",
      }),
    });
    const user = userEvent.setup();
    const { onSearchChange } = renderPage();
    await screen.findByRole("grid");
    expect(screen.getByRole("button", { name: "次の日" })).toBeDisabled();

    await user.click(screen.getByRole("button", { name: "前の日" }));
    expect(onSearchChange).toHaveBeenLastCalledWith(
      { date: "2026-11-19", no: undefined },
      undefined,
    );
    expect(await screen.findByRole("row", { name: /000101/ })).toBeInTheDocument();
    expect(mockList).toHaveBeenLastCalledWith("2026-11-19");
    expect(screen.getByRole("button", { name: "前の日" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "次の日" })).toBeEnabled();

    fireEvent.change(screen.getByLabelText("日付を選択"), { target: { value: "2026-11-10" } });
    fireEvent.keyDown(screen.getByLabelText("日付を選択"), { key: "Enter" });
    expect(await screen.findByText("この日の控えはありません")).toBeInTheDocument();
    expect(mockList).toHaveBeenLastCalledWith("2026-11-10");
  });

  it("U4: 日付の欄は打っている間は日を移さず、Enter か欄から出たときに 1 回だけ移る。無効・空なら今の日へ戻す", async () => {
    serveDays({
      latest: day(),
      "2026-11-10": day({ date: "2026-11-10", records: [] }),
      "2026-11-19": day({ date: "2026-11-19", records: [summary(9, "000101")] }),
    });
    const { onSearchChange } = renderPage();
    await screen.findByRole("grid");
    const input = screen.getByLabelText("日付を選択");

    // WebView2 は日の 2 桁目を待つ間も change を出す（`2` → `02`）。ここで移ると打ちかけが捨てられる
    fireEvent.change(input, { target: { value: "2026-11-02" } });
    fireEvent.change(input, { target: { value: "2026-11-10" } });
    expect(onSearchChange).not.toHaveBeenCalled();
    expect(input).toHaveValue("2026-11-10");

    fireEvent.keyDown(input, { key: "Enter" });
    fireEvent.focusOut(input);
    expect(onSearchChange).toHaveBeenCalledTimes(1);
    expect(onSearchChange).toHaveBeenLastCalledWith(
      { date: "2026-11-10", no: undefined },
      undefined,
    );
    expect(await screen.findByText("この日の控えはありません")).toBeInTheDocument();

    fireEvent.change(input, { target: { value: "2026-11-19" } });
    fireEvent.focusOut(input);
    expect(onSearchChange).toHaveBeenCalledTimes(2);
    expect(await screen.findByRole("row", { name: /000101/ })).toBeInTheDocument();

    // 打ちかけで欄から出た（値が空）ときは移らず、今の日へ戻す
    fireEvent.change(input, { target: { value: "" } });
    fireEvent.focusOut(input);
    expect(onSearchChange).toHaveBeenCalledTimes(2);
    expect(input).toHaveValue("2026-11-19");
  });

  it("U4: 日付を打ったまま mouse で前の日を押すと、打った日へは移らず表示中の日から 1 回だけ移る", async () => {
    serveDays({
      latest: day(),
      "2026-11-19": day({ date: "2026-11-19", records: [summary(9, "000101")] }),
    });
    const user = userEvent.setup();
    const { onSearchChange } = renderPage();
    await screen.findByRole("grid");
    const input = screen.getByLabelText("日付を選択");
    await user.click(input);
    fireEvent.change(input, { target: { value: "2026-11-10" } });

    await user.click(screen.getByRole("button", { name: "前の日" }));
    expect(await screen.findByRole("row", { name: /000101/ })).toBeInTheDocument();
    expect(onSearchChange).toHaveBeenCalledTimes(1);
    expect(onSearchChange).toHaveBeenLastCalledWith(
      { date: "2026-11-19", no: undefined },
      undefined,
    );
    expect(input).toHaveValue("2026-11-19");
  });

  it("U4: 欄で移った後に前後のボタンで移ると、前に打った日をもう一度打って Enter で移れる", async () => {
    serveDays({
      latest: day(),
      "2026-11-10": day({
        date: "2026-11-10",
        prev_date: null,
        next_date: "2026-11-19",
        records: [],
      }),
      "2026-11-19": day({ date: "2026-11-19", records: [summary(9, "000101")] }),
    });
    const user = userEvent.setup();
    const { onSearchChange } = renderPage();
    await screen.findByRole("grid");
    const input = screen.getByLabelText("日付を選択");

    fireEvent.change(input, { target: { value: "2026-11-10" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(await screen.findByText("この日の控えはありません")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "次の日" }));
    expect(await screen.findByRole("row", { name: /000101/ })).toBeInTheDocument();

    fireEvent.change(input, { target: { value: "2026-11-10" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onSearchChange).toHaveBeenCalledTimes(3);
    expect(onSearchChange).toHaveBeenLastCalledWith(
      { date: "2026-11-10", no: undefined },
      undefined,
    );
  });

  it("U6: 番号で 1 件ならその日に移って行を選び、本文は一覧の外の ID でも get_ej_record で読む", async () => {
    serveDays({ latest: day(), "2026-11-19": day({ date: "2026-11-19", records: [] }) });
    mockFind.mockResolvedValueOnce(ok<EjRecordRef[]>([{ id: 5000, date: "2026-11-19" }]));
    const user = userEvent.setup();
    const { onSearchChange } = renderPage();
    await screen.findByRole("grid");
    await user.type(screen.getByLabelText("番号で探す"), "102{Enter}");
    expect(mockFind).toHaveBeenCalledWith("102");
    await waitFor(() => {
      expect(onSearchChange).toHaveBeenCalledWith(
        { date: "2026-11-19", no: undefined },
        { replace: false },
      );
    });
    expect(await screen.findByText("body id5000")).toBeInTheDocument();
    expect(mockGet).toHaveBeenCalledWith(5000);
  });

  it("U6: 2 件なら移らず件数と日付の link を出し、押すとその id の行。0 件と形の誤りは入力の下に出す", async () => {
    serveDays({
      latest: day(),
      "2026-11-18": day({ date: "2026-11-18", records: [summary(7, "000102")] }),
    });
    mockFind
      .mockResolvedValueOnce(
        ok<EjRecordRef[]>([
          { id: 7, date: "2026-11-18" },
          { id: 8, date: "2026-11-20" },
        ]),
      )
      .mockResolvedValueOnce(ok<EjRecordRef[]>([]))
      .mockResolvedValueOnce(err("validation", "記録の番号は 6 桁までの数字で入れてください"));
    const user = userEvent.setup();
    const { onSearchChange } = renderPage();
    await screen.findByRole("grid");
    const input = screen.getByLabelText("番号で探す");

    await user.type(input, "102{Enter}");
    expect(await screen.findByText("同じ番号の控えが 2 件あります")).toBeInTheDocument();
    expect(onSearchChange).not.toHaveBeenCalled();
    await user.click(screen.getByRole("button", { name: "2026-11-18" }));
    expect(onSearchChange).toHaveBeenLastCalledWith(
      { date: "2026-11-18", no: undefined },
      undefined,
    );
    expect(await screen.findByRole("row", { name: /000102/ })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(mockGet).toHaveBeenLastCalledWith(7);

    await user.clear(input);
    await user.type(input, "999{Enter}");
    expect(await screen.findByText("その番号の控えはありません")).toBeInTheDocument();

    await user.clear(input);
    await user.type(input, "12a{Enter}");
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "記録の番号は 6 桁までの数字で入れてください",
    );
  });

  it("U7: 変換中の Enter は送らず、全角の数字は半角で送る。?no= で開くと探して行を選び、search を replace で日に替え、その後に日を移れる", async () => {
    serveDays({
      latest: day(),
      "2026-11-19": day({
        date: "2026-11-19",
        next_date: "2026-11-20",
        records: [summary(3, "000123")],
      }),
      "2026-11-20": day(),
    });
    mockFind.mockResolvedValue(ok<EjRecordRef[]>([{ id: 3, date: "2026-11-19" }]));
    const user = userEvent.setup();
    const { onSearchChange } = renderPage({ no: "000123" });
    expect(mockFind).toHaveBeenCalledWith("000123");
    await waitFor(() => {
      expect(onSearchChange).toHaveBeenCalledWith(
        { date: "2026-11-19", no: undefined },
        { replace: true },
      );
    });
    expect(await screen.findByRole("row", { name: /000123/ })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await user.click(screen.getByRole("button", { name: "次の日" }));
    expect(onSearchChange).toHaveBeenLastCalledWith(
      { date: "2026-11-20", no: undefined },
      undefined,
    );
    expect(await screen.findByRole("row", { name: /000106/ })).toBeInTheDocument();

    mockFind.mockClear();
    const input = screen.getByLabelText("番号で探す");
    fireEvent.change(input, { target: { value: "１０２" } });
    fireEvent.keyDown(input, { key: "Enter", isComposing: true });
    expect(mockFind).not.toHaveBeenCalled();
    fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() => {
      expect(mockFind).toHaveBeenCalledWith("102");
    });
  });

  it("U8: 控えの文は行末の空白を含めて記録のまま <pre> に出し、折り返さず、80 の字形を当てる。行は上下と Enter で選べる", async () => {
    const body = "000106 2026/11/20 10:05  \nﾘﾝｺﾞ          ¥100 \n合  計        ¥100   ";
    serveDays({ latest: day() });
    mockGet.mockImplementation((id) => Promise.resolve(ok(detail(id, "000106", body))));
    const user = userEvent.setup();
    renderPage();
    await screen.findByRole("grid");
    // roving tabindex: 選ぶ前は先頭の行、選んだ後は選んだ行だけが Tab で止まる（80 §80.3）
    expect(rowOf("000106")).toHaveAttribute("tabindex", "0");
    expect(rowOf("000107")).toHaveAttribute("tabindex", "-1");
    await user.click(rowOf("000107"));
    expect(rowOf("000107")).toHaveAttribute("tabindex", "0");
    expect(rowOf("000106")).toHaveAttribute("tabindex", "-1");
    await user.click(rowOf("000106"));
    const section = await screen.findByRole("region", { name: "控えの文" });
    const pre = section.querySelector("pre");
    expect(pre?.textContent).toBe(body);
    // 80 §80.4 の字形の順（本番の定数を写さない独立の期待値）
    expect(pre).toHaveStyle({
      whiteSpace: "pre",
      fontFamily: '"BIZ UDGothic", "MS Gothic", monospace',
    });

    fireEvent.keyDown(rowOf("000106"), { key: "ArrowDown" });
    expect(rowOf("000107")).toHaveAttribute("aria-selected", "true");
    expect(rowOf("000107")).toHaveFocus();
    fireEvent.keyDown(rowOf("000107"), { key: "ArrowUp" });
    expect(rowOf("000106")).toHaveAttribute("aria-selected", "true");
    fireEvent.keyDown(rowOf("000107"), { key: "Enter" });
    expect(rowOf("000107")).toHaveAttribute("aria-selected", "true");
  });

  it("U8: 選んだ控えは表示中の日に結ぶ。行を選んだ後に search の日を外から替えると控えの文が消える", async () => {
    serveDays({
      latest: day(),
      "2026-11-19": day({ date: "2026-11-19", records: [summary(9, "000101")] }),
      "2026-11-20": day(),
    });
    const user = userEvent.setup();
    const { setSearch } = renderPage();
    await screen.findByRole("grid");
    await user.click(rowOf("000106"));
    expect(await screen.findByText("body id1")).toBeInTheDocument();

    // サイドバーの「レシートの控え」・戻る（moveTo を通らない日の移動）
    setSearch({ date: "2026-11-19" });
    expect(await screen.findByRole("row", { name: /000101/ })).toBeInTheDocument();
    expect(screen.queryByRole("region", { name: "控えの文" })).not.toBeInTheDocument();
    expect(rowOf("000101")).toHaveAttribute("aria-selected", "false");
  });

  it("U8: 日を移る間に前の表の行を押しても、移った後にその控えを残さない", async () => {
    let resolvePrev!: (value: DayResponse) => void;
    mockList.mockImplementation((date) =>
      date === "2026-11-19"
        ? new Promise((resolve) => {
            resolvePrev = resolve;
          })
        : Promise.resolve(ok(day())),
    );
    const user = userEvent.setup();
    renderPage();
    await screen.findByRole("grid");
    await user.click(screen.getByRole("button", { name: "前の日" }));
    // 前の日の応答の前は前の表が残る（keepPreviousData）。その行を押す
    await user.click(rowOf("000106"));
    act(() => {
      resolvePrev(ok(day({ date: "2026-11-19", records: [summary(1, "000106")] })));
    });
    await waitFor(() => {
      expect(screen.getByLabelText("日付を選択")).toHaveValue("2026-11-19");
    });
    // 同じ id の行が移った先にあっても、前の日で選んだものは選ばない
    expect(rowOf("000106")).toHaveAttribute("aria-selected", "false");
    expect(screen.queryByRole("region", { name: "控えの文" })).not.toBeInTheDocument();
  });

  it("U6: 番号の件数は今回の応答で判定する（C27 で無効にした cache が 1 件でも、応答が 2 件なら移らず候補を出す）", async () => {
    serveDays({ latest: day() });
    let resolveFind!: (value: Awaited<ReturnType<typeof commands.findEjRecordsByNo>>) => void;
    mockFind.mockReturnValueOnce(
      new Promise((resolve) => {
        resolveFind = resolve;
      }),
    );
    const user = userEvent.setup();
    const { onSearchChange, queryClient } = renderPage();
    await screen.findByRole("grid");
    // 前の検索の 1 件が cache に残り、取込みの C27 で無効になった
    queryClient.setQueryData(queryKeys.ejRecords.byNo("102"), [{ id: 5000, date: "2026-11-19" }]);
    await queryClient.invalidateQueries({ queryKey: queryKeys.ejRecords.root() });

    await user.type(screen.getByLabelText("番号で探す"), "102{Enter}");
    await waitFor(() => {
      expect(mockFind).toHaveBeenCalledWith("102");
    });
    expect(onSearchChange).not.toHaveBeenCalled();
    expect(screen.queryByRole("region", { name: "控えの文" })).not.toBeInTheDocument();

    act(() => {
      resolveFind(
        ok<EjRecordRef[]>([
          { id: 5000, date: "2026-11-19" },
          { id: 6000, date: "2026-11-20" },
        ]),
      );
    });
    expect(await screen.findByText("同じ番号の控えが 2 件あります")).toBeInTheDocument();
    expect(onSearchChange).not.toHaveBeenCalled();
    expect(mockGet).not.toHaveBeenCalledWith(5000);
  });

  it("U6: 番号の取り直しが失敗したら、cache の 1 件で移らず失敗の文を出す", async () => {
    serveDays({ latest: day() });
    mockFind.mockResolvedValueOnce(
      err("validation", "記録の番号は 6 桁までの数字で入れてください"),
    );
    const user = userEvent.setup();
    const { onSearchChange, queryClient } = renderPage();
    await screen.findByRole("grid");
    queryClient.setQueryData(queryKeys.ejRecords.byNo("102"), [{ id: 5000, date: "2026-11-19" }]);
    await queryClient.invalidateQueries({ queryKey: queryKeys.ejRecords.root() });

    await user.type(screen.getByLabelText("番号で探す"), "102{Enter}");
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "記録の番号は 6 桁までの数字で入れてください",
    );
    expect(onSearchChange).not.toHaveBeenCalled();
    expect(screen.queryByText(/同じ番号の控え/)).not.toBeInTheDocument();
  });

  it("U6: 番号の検索が失敗した後、同じ番号の Enter で取り直し、その応答で件数を出す（本番の QueryClient）", async () => {
    serveDays({ latest: day() });
    mockFind
      .mockResolvedValueOnce(
        err("internal", "データベースエラーが発生しました。もう一度お試しください"),
      )
      .mockResolvedValueOnce(ok<EjRecordRef[]>([]));
    const user = userEvent.setup();
    renderPage({}, createAppQueryClient());
    await screen.findByRole("grid");
    const input = screen.getByLabelText("番号で探す");

    await user.type(input, "102{Enter}");
    expect(await screen.findByRole("alert")).toHaveTextContent("データベースエラーが発生しました");
    expect(mockFind).toHaveBeenCalledTimes(1);

    await user.type(input, "{Enter}");
    expect(await screen.findByText("その番号の控えはありません")).toBeInTheDocument();
    expect(mockFind).toHaveBeenCalledTimes(2);
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it.each([
    ["初回の検索", false],
    ["失敗の後の同じ番号の取り直し", true],
  ])(
    "U6: %s が取込みの前から走り取込みの後に取込み前の 0 件を返しても、C27 の後の応答で件数を出す（本番の QueryClient）",
    async (_label, retryAfterFailure) => {
      serveDays({ latest: day(), "2026-11-19": day({ date: "2026-11-19", prev_date: null }) });
      let resolvePreImport!: (
        value: Awaited<ReturnType<typeof commands.findEjRecordsByNo>>,
      ) => void;
      if (retryAfterFailure) {
        mockFind.mockResolvedValueOnce(
          err("internal", "データベースエラーが発生しました。もう一度お試しください"),
        );
      }
      mockFind
        .mockReturnValueOnce(
          new Promise((resolve) => {
            resolvePreImport = resolve;
          }),
        )
        .mockResolvedValue(ok<EjRecordRef[]>([{ id: 1, date: "2026-11-19" }]));
      let resolveImport!: (value: Awaited<ReturnType<typeof commands.importEjFiles>>) => void;
      mockImport.mockReturnValueOnce(
        new Promise((resolve) => {
          resolveImport = resolve;
        }),
      );
      vi.mocked(open).mockResolvedValue(["C:\\tmp\\EJ261119.TXT"]);
      const user = userEvent.setup();
      const { onSearchChange } = renderPage({}, createAppQueryClient());
      await screen.findByRole("grid");
      const input = screen.getByLabelText("番号で探す");
      await user.type(input, "102{Enter}");
      if (retryAfterFailure) {
        expect(await screen.findByRole("alert")).toHaveTextContent(
          "データベースエラーが発生しました",
        );
        await user.type(input, "{Enter}");
      }
      const callsBeforeImport = retryAfterFailure ? 2 : 1;
      await waitFor(() => {
        expect(mockFind).toHaveBeenCalledTimes(callsBeforeImport);
      });

      const pick = screen.getByRole("button", { name: "EJ のファイルを選んで取り込む" });
      await user.click(pick);
      await waitFor(() => {
        expect(mockImport).toHaveBeenCalled();
      });
      act(() => {
        resolveImport(ok(ejResult()));
      });
      // C27 の取り直しが始まった後に、取込みの前に読んだ 0 件が返る
      await waitFor(() => {
        expect(mockList.mock.calls.filter(([date]) => date === null)).toHaveLength(2);
      });
      act(() => {
        resolvePreImport(ok<EjRecordRef[]>([]));
      });
      await waitFor(() => {
        expect(pick).toBeEnabled();
      });

      expect(mockFind).toHaveBeenCalledTimes(callsBeforeImport + 1);
      await waitFor(() => {
        expect(onSearchChange).toHaveBeenCalledWith(
          { date: "2026-11-19", no: undefined },
          { replace: false },
        );
      });
      expect(await screen.findByText("body id1")).toBeInTheDocument();
      expect(screen.queryByText("その番号の控えはありません")).not.toBeInTheDocument();
    },
  );

  it("U9: 欠けは warning の Alert が icon・見出し・区間ごとの文で示し、無ければ Alert を出さない。下の補助のリンクは常にある", async () => {
    serveDays({
      latest: day({
        gaps: [
          { machine_prefix: "0001", from_no: "000104", to_no: "000105" },
          { machine_prefix: "0001", from_no: "000109", to_no: "000109" },
        ],
      }),
      "2026-11-19": day({ date: "2026-11-19", gaps: [] }),
    });
    const user = userEvent.setup();
    renderPage();
    const alert = await screen.findByRole("alert");
    expect(within(alert).getByText("控えが足りない所があります")).toBeInTheDocument();
    expect(within(alert).getByText("記録 000104〜000105 の控えがありません。")).toBeInTheDocument();
    expect(within(alert).getByText("記録 000109 の控えがありません。")).toBeInTheDocument();
    expect(alert.querySelector("svg")).not.toBeNull();
    expect(
      screen.getByRole("button", { name: "EJ のファイルを選んで取り込む" }),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "前の日" }));
    await waitFor(() => {
      expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    });
    expect(
      screen.getByRole("button", { name: "EJ のファイルを選んで取り込む" }),
    ).toBeInTheDocument();
  });

  it("U10: 手で選ぶ EJ は待つ間に離脱と close を止め、成功で C27 で取り直して Alert を消し、toast と EjNotes で名指す", async () => {
    const withGap = day({ gaps: [{ machine_prefix: "0001", from_no: "000104", to_no: "000105" }] });
    mockList.mockResolvedValueOnce(ok(withGap)).mockResolvedValue(ok(day()));
    let resolveImport!: (value: Awaited<ReturnType<typeof commands.importEjFiles>>) => void;
    mockImport.mockReturnValueOnce(
      new Promise((resolve) => {
        resolveImport = resolve;
      }),
    );
    vi.mocked(open).mockResolvedValue(["C:\\tmp\\EJ261119.TXT"]);
    const user = userEvent.setup();
    renderPage();
    expect(await screen.findByRole("alert")).toHaveTextContent("控えが足りない所があります");

    await user.click(screen.getByRole("button", { name: "EJ のファイルを選んで取り込む" }));
    await waitFor(() => {
      expect(mockImport).toHaveBeenCalledWith([
        { filename: "EJ261119.TXT", file_bytes: [1, 2, 3] },
      ]);
    });
    const blocker = mockUseBlocker.mock.lastCall?.[0] as unknown as {
      shouldBlockFn: () => boolean;
      enableBeforeUnload: () => boolean;
    };
    expect(blocker.shouldBlockFn()).toBe(true);
    expect(blocker.enableBeforeUnload()).toBe(true);
    expect(getWindowCloseGuardKinds()).toEqual(["ej-import"]);
    expect(screen.getByRole("button", { name: "EJ のファイルを選んで取り込む" })).toBeDisabled();

    act(() => {
      resolveImport(
        ok(
          ejResult({
            unreadable_files: ["EJ261118.TXT"],
            conflict_count: 1,
            conflicts: [{ machine_prefix: "0001", record_no: "000099", saved: true }],
          }),
        ),
      );
    });
    await waitFor(() => {
      expect(screen.queryByText("控えが足りない所があります")).not.toBeInTheDocument();
    });
    expect(mockList).toHaveBeenCalledTimes(2);
    expect(vi.mocked(toast.success)).toHaveBeenCalledWith(
      "選んだファイルから控え 2 件を取り込みました",
    );
    const notes = screen.getByRole("status", { name: "選んだファイルの取込みの結果" });
    expect(notes).toHaveTextContent("読めないファイル EJ261118.TXT");
    expect(notes).toHaveTextContent("記録 000099: 前に取り込んだ控えと食い違う控えがあったため");
    expect(getWindowCloseGuardKinds()).toEqual([]);
    const after = mockUseBlocker.mock.lastCall?.[0] as unknown as { shouldBlockFn: () => boolean };
    expect(after.shouldBlockFn()).toBe(false);

    // 追加 0 件でも toast で知らせる
    mockImport.mockResolvedValueOnce(ok(ejResult({ inserted_count: 0 })));
    await user.click(screen.getByRole("button", { name: "EJ のファイルを選んで取り込む" }));
    await waitFor(() => {
      expect(vi.mocked(toast.success)).toHaveBeenLastCalledWith(
        "選んだファイルから新しい控えはありませんでした",
      );
    });

    // 失敗は文を出し、もう一度押せる
    mockImport.mockResolvedValueOnce(err("validation", "ファイルサイズが上限(20MB)を超えています"));
    await user.click(screen.getByRole("button", { name: "EJ のファイルを選んで取り込む" }));
    expect(await screen.findByText("ファイルサイズが上限(20MB)を超えています")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "EJ のファイルを選んで取り込む" })).toBeEnabled();
  });

  it("U10: 手で選ぶ EJ の待ちの間に番号で 1 件見つかっても移らず検索を残し、待ちが終わるとその日に移る", async () => {
    serveDays({ latest: day(), "2026-11-19": day({ date: "2026-11-19", prev_date: null }) });
    mockFind.mockResolvedValue(ok<EjRecordRef[]>([{ id: 1, date: "2026-11-19" }]));
    let resolveImport!: (value: Awaited<ReturnType<typeof commands.importEjFiles>>) => void;
    mockImport.mockReturnValueOnce(
      new Promise((resolve) => {
        resolveImport = resolve;
      }),
    );
    vi.mocked(open).mockResolvedValue(["C:\\tmp\\EJ261119.TXT"]);
    const user = userEvent.setup();
    const { onSearchChange } = renderPage();
    await screen.findByRole("grid");

    await user.click(screen.getByRole("button", { name: "EJ のファイルを選んで取り込む" }));
    await waitFor(() => {
      expect(mockImport).toHaveBeenCalled();
    });
    await user.type(screen.getByLabelText("番号で探す"), "102{Enter}");
    await waitFor(() => {
      expect(mockFind).toHaveBeenCalledWith("102");
    });
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 50));
    });
    // 待ちの間は route の移動が止まるので、移らない
    expect(onSearchChange).not.toHaveBeenCalled();

    act(() => {
      resolveImport(ok(ejResult()));
    });
    await waitFor(() => {
      expect(onSearchChange).toHaveBeenCalledWith(
        { date: "2026-11-19", no: undefined },
        { replace: false },
      );
    });
    expect(await screen.findByRole("region", { name: "控えの文" })).toBeInTheDocument();
  });

  it("U10: 手で選ぶ EJ の待ちの間は日付の欄と前の日・次の日を押せず、待ちの後に打って Enter で移る", async () => {
    serveDays({
      latest: day({ next_date: "2026-11-21" }),
      "2026-11-10": day({ date: "2026-11-10", records: [] }),
    });
    let resolveImport!: (value: Awaited<ReturnType<typeof commands.importEjFiles>>) => void;
    mockImport.mockReturnValueOnce(
      new Promise((resolve) => {
        resolveImport = resolve;
      }),
    );
    vi.mocked(open).mockResolvedValue(["C:\\tmp\\EJ261119.TXT"]);
    const user = userEvent.setup();
    const { onSearchChange } = renderPage();
    await screen.findByRole("grid");
    const input = screen.getByLabelText("日付を選択");

    await user.click(screen.getByRole("button", { name: "EJ のファイルを選んで取り込む" }));
    await waitFor(() => {
      expect(mockImport).toHaveBeenCalled();
    });
    expect(input).toBeDisabled();
    expect(screen.getByRole("button", { name: "前の日" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "次の日" })).toBeDisabled();

    act(() => {
      resolveImport(ok(ejResult()));
    });
    await waitFor(() => {
      expect(input).toBeEnabled();
    });
    expect(screen.getByRole("button", { name: "前の日" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "次の日" })).toBeEnabled();
    fireEvent.change(input, { target: { value: "2026-11-10" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onSearchChange).toHaveBeenCalledWith({ date: "2026-11-10", no: undefined }, undefined);
    expect(await screen.findByText("この日の控えはありません")).toBeInTheDocument();
  });

  it("U11: 記録が 1 件も無ければ SCREEN_DESIGN の文と取込みへの link と、下の補助のリンクを出す", async () => {
    mockList.mockResolvedValueOnce(ok(day({ date: null, prev_date: null, records: [] })));
    renderPage();
    expect(
      await screen.findByText(
        "レシートの控えはまだありません。売上データ取込みで SD を読むと、ここに出ます。",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "売上データ取込みへ" })).toHaveAttribute(
      "href",
      "/csv-import",
    );
    expect(
      screen.getByRole("button", { name: "EJ のファイルを選んで取り込む" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "前の日" })).not.toBeInTheDocument();
  });

  it("U11: 読込みの失敗は原因と再試行を出し、再試行で取り直す。1000 件を超えた日は文で示す", async () => {
    mockList
      .mockResolvedValueOnce(
        err("internal", "データベースエラーが発生しました。もう一度お試しください"),
      )
      .mockResolvedValueOnce(ok(day({ truncated: true })));
    const user = userEvent.setup();
    renderPage();
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("レシートの控えを読めませんでした");
    expect(alert).toHaveTextContent("データベースエラーが発生しました");
    expect(screen.queryByText(/レシートの控えはまだありません/)).not.toBeInTheDocument();
    await user.click(within(alert).getByRole("button", { name: "再試行" }));
    expect(
      await screen.findByText("この日の控えが多いため、先頭 1000 件だけを出しています"),
    ).toBeInTheDocument();
    expect(mockList).toHaveBeenCalledTimes(2);
  });

  it("U10: 日の一覧を読めない間も、取込みの失敗の理由をリンクの近くに出す", async () => {
    mockList.mockResolvedValue(
      err("internal", "データベースエラーが発生しました。もう一度お試しください"),
    );
    mockImport.mockResolvedValueOnce(err("validation", "ファイルサイズが上限(20MB)を超えています"));
    vi.mocked(open).mockResolvedValue(["C:\\tmp\\EJ261119.TXT"]);
    const user = userEvent.setup();
    renderPage();
    expect(await screen.findByText("レシートの控えを読めませんでした")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "EJ のファイルを選んで取り込む" }));
    expect(await screen.findByText("ファイルサイズが上限(20MB)を超えています")).toBeInTheDocument();
  });
});
