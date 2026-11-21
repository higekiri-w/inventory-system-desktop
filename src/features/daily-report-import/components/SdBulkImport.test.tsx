// UI-07-D17（(A) の分）: 「まとめて取り込む」の押せる条件・結果の文・EJ の名指し・手で選ぶ EJ
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import type {
  DailyReportSdCandidateStatus,
  DailyReportSdScan,
  EjImportResult,
  RegisterSdBulkResponse,
} from "@/lib/bindings";
import { SdBulkImport, SdEjSummaryLine } from "./SdBulkImport";

vi.mock("@tanstack/react-router", () => ({
  Link: ({ children }: { children: ReactNode }) => <a href="/reports/daily">{children}</a>,
}));

function makeScan(
  statuses: DailyReportSdCandidateStatus[],
  ej: DailyReportSdScan["ej"],
): DailyReportSdScan {
  return {
    volume_label: "E:",
    scanned_from: "2026-02-20",
    candidates: statuses.map((status, index) => ({
      candidate_key: `k${String(index)}`,
      path_date: "2026-03-21",
      report_date: "2026-03-21",
      source_filenames: ["Z001_21 .CSV"],
      settlement_no: index + 1,
      status,
      unreadable_reason: null,
    })),
    skipped_unknown_count: 0,
    tidy_failure: null,
    ej,
  };
}

const ZERO = { new_count: 0, held_count: 0, conflict_count: 0, unreadable_file_count: 0 };

function makeEj(overrides: Partial<EjImportResult> = {}): EjImportResult {
  return {
    inserted_count: 0,
    already_count: 0,
    conflict_count: 0,
    conflicts: [],
    held_count: 0,
    unreadable_files: [],
    gaps: [],
    ...overrides,
  };
}

const NO_MANUAL = { result: null, error: null, pending: false };

function renderBulk(
  scan: DailyReportSdScan,
  response: RegisterSdBulkResponse | null = null,
  options: {
    manual?: { result: EjImportResult | null; error: string | null; pending: boolean };
  } = {},
) {
  const onBulkImport = vi.fn();
  const onChooseEjFiles = vi.fn();
  const onConfirmCandidate = vi.fn();
  const view = render(
    <SdBulkImport
      scan={scan}
      busy={false}
      importing={false}
      response={response}
      ejManual={options.manual ?? NO_MANUAL}
      onBulkImport={onBulkImport}
      onChooseEjFiles={onChooseEjFiles}
      onConfirmCandidate={onConfirmCandidate}
    />,
  );
  return { ...view, onBulkImport, onChooseEjFiles, onConfirmCandidate };
}

function ejResponse(result: EjImportResult): RegisterSdBulkResponse {
  return { items: [{ source: { type: "Ej" }, outcome: { type: "ImportedEj", ...result } }] };
}

describe("SdBulkImport (UI-07-D17)", () => {
  it.each([
    ["NotImported が 1 件", ["NotImported", "Imported"], ZERO, true],
    ["新しい控え 1", ["Imported"], { ...ZERO, new_count: 1 }, true],
    ["保留 1", ["Imported"], { ...ZERO, held_count: 1 }, true],
    ["食い違い 1", ["Imported"], { ...ZERO, conflict_count: 1 }, true],
    ["読めない file 1", ["Imported"], { ...ZERO, unreadable_file_count: 1 }, true],
    ["全部 0", ["Imported", "Incomplete"], ZERO, false],
    ["EJ が無い", ["Imported"], null, false],
  ] as const)(
    "U1 REQ-401: %s のときの「まとめて取り込む」の押せる・押せない",
    async (_label, statuses, ej, enabled) => {
      const { onBulkImport } = renderBulk(makeScan([...statuses], ej));
      const button = screen.getByRole("button", { name: "まとめて取り込む" });
      if (enabled) {
        expect(button).toBeEnabled();
        await userEvent.click(button);
        expect(onBulkImport).toHaveBeenCalledTimes(1);
      } else {
        expect(button).toBeDisabled();
      }
    },
  );

  it("U1 REQ-401（closure round 2 #1）: 止まった結果の後は、畳んだ scan に残りが無くても「まとめて取り込む」を押し直せる", () => {
    // 日報は全部入り、EJ だけが止まった（scan の EJ の数は 0 のまま）
    const failedEj: RegisterSdBulkResponse = {
      items: [
        {
          source: { type: "Ej" },
          outcome: { type: "Failed", kind: "internal", message: "保存できませんでした" },
        },
      ],
    };
    const view = renderBulk(makeScan(["Imported"], ZERO), failedEj);
    expect(screen.getByText(/途中で止まりました。/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "まとめて取り込む" })).toBeEnabled();
    view.unmount();
    // 止まっていなければ、残りの無い scan は押せない
    renderBulk(makeScan(["Imported"], ZERO), ejResponse(makeEj()));
    expect(screen.getByRole("button", { name: "まとめて取り込む" })).toBeDisabled();
  });

  it("U3 REQ-401: 5 種の outcome を label と icon で出し、止まったときは上に文を出す", () => {
    const daily = (key: string) => ({
      type: "DailyReport" as const,
      candidate_key: key,
      report_date: "2026-03-21",
    });
    renderBulk(makeScan(["NotImported"], null), {
      items: [
        {
          source: daily("a"),
          outcome: {
            type: "ImportedDailyReport",
            daily_report_import_id: 1,
            status: "completed",
            report_date: "2026-03-21",
            gross_amount: 12000,
            net_amount: 11000,
            warning_count: 0,
          },
        },
        { source: daily("b"), outcome: { type: "Skipped", message: "取込み済みです" } },
        { source: daily("c"), outcome: { type: "NeedsIndividualConfirmation" } },
        {
          source: daily("d"),
          outcome: { type: "Failed", kind: "internal", message: "写しを保存できませんでした" },
        },
        { source: daily("e"), outcome: { type: "NotAttempted" } },
      ],
    });
    const list = screen.getByRole("list", { name: "まとめて取り込んだ結果" });
    const rows = within(list).getAllByRole("listitem");
    const labels = [
      "取り込みました",
      "取り込みませんでした",
      "1 件ずつ確かめてください",
      "ここで止まりました",
      "取り込んでいません",
    ];
    labels.forEach((label, index) => {
      const row = rows[index];
      expect(within(row).getByText(label)).toBeInTheDocument();
      // 色だけで示さない: label の横に icon
      expect(within(row).getByText(label).querySelector("svg")).not.toBeNull();
    });
    expect(within(rows[0]).getByText(/総売上/)).toBeInTheDocument();
    expect(within(rows[1]).getByText("取込み済みです")).toBeInTheDocument();
    expect(within(rows[3]).getByText("写しを保存できませんでした")).toBeInTheDocument();
    expect(
      screen.getByText(
        "途中で止まりました。取り込めた分は残っています。原因を直してから、もう一度「まとめて取り込む」を押してください。",
      ),
    ).toBeInTheDocument();
  });

  it("U4・U5 REQ-401: EJ の欠け・読めない file・食い違いを名指し、0 件の行は出さず、保留は注意にしない", () => {
    renderBulk(
      makeScan(["Imported"], { ...ZERO, held_count: 1 }),
      ejResponse(
        makeEj({
          inserted_count: 3,
          held_count: 1,
          conflict_count: 2,
          conflicts: [
            { machine_prefix: "0001", record_no: "000110", saved: true },
            { machine_prefix: "0001", record_no: "000111", saved: false },
          ],
          unreadable_files: ["EJ260320_0002.TXT"],
          gaps: [
            { machine_prefix: "0001", from_no: "000102", to_no: "000103" },
            { machine_prefix: "0001", from_no: "000105", to_no: "000105" },
          ],
        }),
      ),
    );
    expect(screen.getByText("控え 3 件を取り込みました")).toBeInTheDocument();
    expect(screen.getByText("記録 000102〜000103 の控えがありません")).toBeInTheDocument();
    expect(screen.getByText("記録 000105 の控えがありません")).toBeInTheDocument();
    expect(screen.getByText(/読めないファイル EJ260320_0002\.TXT/)).toBeInTheDocument();
    expect(screen.getByText(/記録 000110: .*前の控えを残しました/)).toBeInTheDocument();
    expect(screen.getByText(/記録 000111: .*保存していません/)).toBeInTheDocument();
    const held = screen.getByText(/最後の控え 1 件は/);
    expect(held.querySelector("svg")).toBeNull();
    expect(held).not.toHaveAttribute("role", "alert");
    expect(held.closest('[role="alert"]')).toBeNull();
    expect(held.className).not.toMatch(/warning|destructive/);
  });

  it("U4 REQ-401: 件数が 0 の項目は行を出さない", () => {
    renderBulk(makeScan(["Imported"], { ...ZERO, new_count: 1 }), ejResponse(makeEj()));
    expect(screen.getByText("新しい控えはありませんでした")).toBeInTheDocument();
    expect(screen.queryByText(/控えがありません/)).not.toBeInTheDocument();
    expect(screen.queryByText(/読めないファイル/)).not.toBeInTheDocument();
    expect(screen.queryByText(/食い違/)).not.toBeInTheDocument();
    expect(screen.queryByText(/最後の控え/)).not.toBeInTheDocument();
    expect(screen.queryByText(/ほか/)).not.toBeInTheDocument();
  });

  it.each([
    [
      "欠けがある",
      makeEj({ gaps: [{ machine_prefix: "0001", from_no: "000104", to_no: "000105" }] }),
      true,
    ],
    ["読めない file 1 本・欠け 0", makeEj({ unreadable_files: ["EJ260320_0002.TXT"] }), true],
    ["どちらも 0", makeEj({ inserted_count: 1 }), false],
  ])(
    "U6 REQ-401: %s のときの「EJ のファイルを選んで取り込む」の有無",
    async (_label, ej, shown) => {
      const { onChooseEjFiles } = renderBulk(makeScan(["Imported"], ZERO), ejResponse(ej));
      const name = "EJ のファイルを選んで取り込む";
      if (shown) {
        await userEvent.click(screen.getByRole("button", { name }));
        expect(onChooseEjFiles).toHaveBeenCalledTimes(1);
      } else {
        expect(screen.queryByRole("button", { name })).toBeNull();
      }
    },
  );

  it("U6 REQ-401（Final round 1 #3）: 手で選んだ結果に読めない file があれば、欠けが 0 でも選び直すボタンを残す", () => {
    renderBulk(
      makeScan(["Imported"], ZERO),
      ejResponse(
        makeEj({ gaps: [{ machine_prefix: "0001", from_no: "000104", to_no: "000105" }] }),
      ),
      {
        manual: {
          result: makeEj({ inserted_count: 2, unreadable_files: ["EJ260320_0003.TXT"] }),
          error: null,
          pending: false,
        },
      },
    );
    expect(screen.queryByText(/控えがありません/)).not.toBeInTheDocument();
    expect(screen.getByText(/読めないファイル EJ260320_0003\.TXT/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "EJ のファイルを選んで取り込む" })).toBeEnabled();
  });

  it("U6 REQ-401（Final round 1 #4）: 再試行の error は、前に成功した結果と別に出す", () => {
    renderBulk(
      makeScan(["Imported"], ZERO),
      ejResponse(
        makeEj({ gaps: [{ machine_prefix: "0001", from_no: "000104", to_no: "000106" }] }),
      ),
      {
        manual: {
          result: makeEj({
            inserted_count: 2,
            gaps: [{ machine_prefix: "0001", from_no: "000106", to_no: "000106" }],
          }),
          error: "EJ のファイルの選択または読み取りに失敗しました",
          pending: false,
        },
      },
    );
    expect(screen.getByText("選んだファイルから控え 2 件を取り込みました")).toBeInTheDocument();
    expect(screen.getByText("記録 000106 の控えがありません")).toBeInTheDocument();
    expect(screen.queryByText("記録 000104〜000106 の控えがありません")).not.toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent(
      "EJ のファイルの選択または読み取りに失敗しました",
    );
  });

  it("REQ-401 / UI-07-D17（Final round 1 #2）: 「1 件ずつ確かめてください」の行の「確認する」で、その束の個別の確認へ進む", async () => {
    const { onConfirmCandidate } = renderBulk(makeScan(["NotImported"], null), {
      items: [
        {
          source: { type: "DailyReport", candidate_key: "k-confirm", report_date: "2026-03-21" },
          outcome: { type: "NeedsIndividualConfirmation" },
        },
        {
          source: { type: "DailyReport", candidate_key: "k-skip", report_date: "2026-03-22" },
          outcome: { type: "Skipped", message: "取込み済みです" },
        },
      ],
    });
    const rows = within(screen.getByRole("list", { name: "まとめて取り込んだ結果" })).getAllByRole(
      "listitem",
    );
    expect(within(rows[1]).queryByRole("button", { name: "確認する" })).toBeNull();
    expect(within(rows[0]).queryByText(/読み直し/)).toBeNull();
    await userEvent.click(within(rows[0]).getByRole("button", { name: "確認する" }));
    expect(onConfirmCandidate).toHaveBeenCalledWith("k-confirm");
  });

  it("U6 REQ-401: 手で選んだ EJ の結果をその場に出し、欠けはその結果の値にする", () => {
    renderBulk(
      makeScan(["Imported"], ZERO),
      ejResponse(
        makeEj({ gaps: [{ machine_prefix: "0001", from_no: "000104", to_no: "000105" }] }),
      ),
      { manual: { result: makeEj({ inserted_count: 2 }), error: null, pending: false } },
    );
    expect(screen.getByText("選んだファイルから控え 2 件を取り込みました")).toBeInTheDocument();
    expect(screen.queryByText(/控えがありません/)).not.toBeInTheDocument();
  });
});

describe("SdEjSummaryLine (UI-07-D17)", () => {
  it("REQ-401: 新しい控えの数と、読めない file・食い違い・保留を添え、EJ が無ければ出さない", () => {
    const { rerender, container } = render(
      <SdEjSummaryLine
        summary={{ new_count: 3, held_count: 1, conflict_count: 2, unreadable_file_count: 1 }}
      />,
    );
    expect(screen.getByText("レシートの控え（EJ）: 新しい控え 3 件")).toBeInTheDocument();
    expect(screen.getByText("読めない EJ のファイルが 1 本あります")).toBeInTheDocument();
    expect(
      screen.getByText("同じ番号で中身の違う控えが 2 件あり、取り込みません"),
    ).toBeInTheDocument();
    expect(screen.getByText(/最後の控え 1 件は/)).toBeInTheDocument();
    rerender(<SdEjSummaryLine summary={ZERO} />);
    expect(screen.getByText("レシートの控え（EJ）: 新しい控えはありません")).toBeInTheDocument();
    rerender(<SdEjSummaryLine summary={null} />);
    expect(container).toBeEmptyDOMElement();
  });
});
