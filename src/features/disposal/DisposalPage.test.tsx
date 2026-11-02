import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import dateTimeSource from "./DisposalPage.tsx?raw";

import { makeMockProductWithRelations } from "@/features/products/lib/test-fixtures";
import { commands } from "@/lib/bindings";
import type { DisposalCreateRequest } from "@/lib/bindings";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { DisposalPage } from "./DisposalPage";

vi.mock("@/hooks/useUnsavedChangesWarning", () => ({
  useUnsavedChangesWarning: () => ({
    isBlocked: false,
    continueEditing: vi.fn(),
    discardAndProceed: vi.fn(),
  }),
}));

vi.mock("@tanstack/react-router", () => ({
  Link: ({
    to,
    params,
    search,
    className,
    children,
  }: {
    to: string;
    params?: Record<string, string>;
    search?: Record<string, string>;
    className?: string;
    children: ReactNode;
  }) => {
    const path = params?.recordId ? to.replace("$recordId", params.recordId) : to;
    const query = search ? `?${new URLSearchParams(search).toString()}` : "";
    return (
      <a className={className} href={`${path}${query}`}>
        {children}
      </a>
    );
  },
  useRouterState: ({ select }: { select: (state: { location: { href: string } }) => unknown }) =>
    select({ location: { href: "/inventory/disposal" } }),
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn(), dismiss: vi.fn() },
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    listDisposals: vi.fn(),
    searchProducts: vi.fn(),
    createDisposal: vi.fn(),
  },
}));

const mockListDisposals = vi.mocked(commands.listDisposals);
const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockCreateDisposal = vi.mocked(commands.createDisposal);
const mockScrollTo = vi.fn();

function renderWithClient(ui: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return {
    queryClient,
    ...render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>),
  };
}

function createDeferred<T>() {
  let resolve: (value: T) => void = () => {
    throw new Error("deferred promise is not initialized");
  };
  const promise = new Promise<T>((promiseResolve) => {
    resolve = promiseResolve;
  });
  return { promise, resolve };
}

function mockDefaultQueries() {
  mockListDisposals.mockResolvedValue({
    status: "ok",
    data: { items: [], total_count: 0, page: 1, per_page: 10 },
  });
}

async function addSingleProduct(user: ReturnType<typeof userEvent.setup>) {
  mockSearchProducts.mockResolvedValue({
    status: "ok",
    data: {
      items: [
        makeMockProductWithRelations({
          product_code: "DP-001",
          name: "廃棄確認 商品",
          cost_price_centi: 12000,
          stock_quantity: 4,
        }),
      ],
      total_count: 1,
      page: 1,
      per_page: 10,
    },
  });

  await user.type(await screen.findByLabelText("廃棄・破損商品検索"), "DP-001{enter}");
  expect(await screen.findByText("DP-001")).toBeInTheDocument();
}

beforeEach(() => {
  mockScrollTo.mockReset();
  vi.stubGlobal("scrollTo", mockScrollTo);
  mockListDisposals.mockReset();
  mockSearchProducts.mockReset();
  mockCreateDisposal.mockReset();
  mockDefaultQueries();
});

describe("DisposalPage (UI-05 / REQ-204)", () => {
  it("Matrix C17 / UI-05-D8: 候補の一覧の原価は ¥83.33、行の原価の初期値は 83.33 で 8333 を送る", async () => {
    const user = userEvent.setup();
    mockCreateDisposal.mockResolvedValue({
      status: "ok",
      data: { record_id: 42, created: true, idempotent_replay: false, stock_warnings: [] },
    });
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({
            product_code: "C17-A",
            name: "原価の小数",
            cost_price_centi: 8333,
          }),
          makeMockProductWithRelations({ product_code: "C17-B", name: "原価の小数 B" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 10,
      },
    });
    renderWithClient(<DisposalPage />);
    await user.type(await screen.findByLabelText("廃棄・破損商品検索"), "原価{enter}");
    const candidate = (await screen.findByText("C17-A")).closest("tr");
    if (candidate === null) throw new Error("expected table structure");
    expect(within(candidate).getByText("¥83.33")).toBeInTheDocument();
    await user.click(within(candidate).getByRole("button", { name: "廃棄・破損に追加" }));
    const cost = await screen.findByLabelText("C17-A の原価");
    expect(cost).toHaveValue(83.33);
    expect(cost).toHaveAttribute("inputmode", "decimal");
    expect(cost).toHaveAttribute("step", "0.01");
    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));
    await waitFor(() => {
      expect(mockCreateDisposal).toHaveBeenCalledTimes(1);
    });
    const [request] = mockCreateDisposal.mock.calls[0] as [DisposalCreateRequest];
    expect(request.items[0]?.cost_price_centi).toBe(8333);
  });

  it("REQ-204 adds a single matching product and restores focus with damage defaults", async () => {
    const user = userEvent.setup();
    renderWithClient(<DisposalPage />);

    await addSingleProduct(user);

    expect(screen.getByText("廃棄確認 商品")).toBeInTheDocument();
    const disposalTypeTrigger = screen.getByLabelText("DP-001 の種別");
    expect(disposalTypeTrigger).toHaveAttribute("data-slot", "select-trigger");
    expect(disposalTypeTrigger.tagName).toBe("BUTTON");
    expect(disposalTypeTrigger).toHaveTextContent("破損");
    expect(screen.getByLabelText("DP-001 の数量")).toHaveValue(1);
    expect(screen.getByLabelText("DP-001 の原価")).toHaveValue(120);
    expect(screen.getByLabelText("DP-001 の理由")).toHaveValue("破損");
    await waitFor(() => {
      expect(screen.getByLabelText("廃棄・破損商品検索")).toHaveFocus();
    });
  });

  it("REQ-204 requires explicit selection when multiple products match", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({ product_code: "DP-001", name: "商品A" }),
          makeMockProductWithRelations({ product_code: "DP-002", name: "商品B" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 10,
      },
    });

    renderWithClient(<DisposalPage />);
    await user.type(await screen.findByLabelText("廃棄・破損商品検索"), "商品{enter}");

    expect(
      await screen.findByText("候補から廃棄・破損する商品を選んでください"),
    ).toBeInTheDocument();
    expect(screen.queryByLabelText("DP-001 の数量")).not.toBeInTheDocument();

    await user.click(screen.getAllByRole("button", { name: "廃棄・破損に追加" })[1]);

    expect(await screen.findByLabelText("DP-002 の数量")).toBeInTheDocument();
  });

  it("REQ-204 shows product registration guidance for no match and unsaved rows", async () => {
    const user = userEvent.setup();
    renderWithClient(<DisposalPage />);
    await addSingleProduct(user);
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 10 },
    });

    await user.type(screen.getByLabelText("廃棄・破損商品検索"), "missing{enter}");

    expect(await screen.findByText("該当する商品がありません")).toBeInTheDocument();
    expect(
      screen.getByText(
        "未登録商品の場合は、商品マスタに登録してから廃棄・破損へ戻って追加します。",
      ),
    ).toBeInTheDocument();
    const unsaved = screen.getByText(
      "未保存の廃棄・破損内容があります。商品登録へ進むとこの画面の入力は残りません。",
    );
    // D-CE10: 失敗ではない知らせは注意・確認の文字（危険の赤にしない）。
    expect(unsaved).toHaveClass("text-warning-emphasis");
    expect(unsaved).not.toHaveClass("text-destructive");
    expect(screen.getByRole("link", { name: "商品登録へ進む" })).toHaveAttribute(
      "href",
      "/products/new",
    );
  });

  it("REQ-204 merges duplicate product type and reason rows", async () => {
    const user = userEvent.setup();
    renderWithClient(<DisposalPage />);

    await addSingleProduct(user);
    await user.type(screen.getByLabelText("廃棄・破損商品検索"), "DP-001{enter}");

    expect(screen.getAllByText("DP-001")).toHaveLength(1);
    expect(screen.getByLabelText("DP-001 の数量")).toHaveValue(2);
  });

  it("REQ-204 keeps different reasons separate and merges them when edited to match", async () => {
    const user = userEvent.setup();
    renderWithClient(<DisposalPage />);

    await addSingleProduct(user);
    fireEvent.change(screen.getByLabelText("DP-001 の理由"), { target: { value: "期限切れ" } });
    await user.type(screen.getByLabelText("廃棄・破損商品検索"), "DP-001{enter}");

    expect(screen.getAllByText("DP-001")).toHaveLength(2);

    fireEvent.change(screen.getAllByLabelText("DP-001 の理由")[1], {
      target: { value: "期限切れ" },
    });

    expect(screen.getAllByText("DP-001")).toHaveLength(1);
    expect(screen.getByLabelText("DP-001 の数量")).toHaveValue(2);
  });

  it("REQ-204 keeps focus while typing a disposal reason", async () => {
    const user = userEvent.setup();
    renderWithClient(<DisposalPage />);

    await addSingleProduct(user);
    const reasonInput = screen.getByLabelText("DP-001 の理由");
    await user.clear(reasonInput);
    await user.type(reasonInput, "期限切れ");

    expect(screen.getByLabelText("DP-001 の理由")).toHaveValue("期限切れ");
    expect(screen.getByLabelText("DP-001 の理由")).toHaveFocus();
  });

  it("REQ-204 blocks invalid quantity cost and reason before submit", async () => {
    const user = userEvent.setup();
    renderWithClient(<DisposalPage />);
    await addSingleProduct(user);

    fireEvent.change(screen.getByLabelText("DP-001 の数量"), { target: { value: "0" } });
    fireEvent.change(screen.getByLabelText("DP-001 の原価"), { target: { value: "-1" } });
    fireEvent.change(screen.getByLabelText("DP-001 の理由"), { target: { value: "" } });
    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));

    expect(
      await screen.findByText(
        "DP-001: 数量は1以上の整数で入力してください / 原価は0以上で、小数は2桁までで入力してください / 理由は必須です",
      ),
    ).toBeInTheDocument();
    expect(mockScrollTo).not.toHaveBeenCalled();
    expect(mockCreateDisposal).not.toHaveBeenCalled();
  });

  it("REQ-204 creates a disposal record and invalidates disposal and inventory queries", async () => {
    const user = userEvent.setup();
    mockCreateDisposal.mockResolvedValue({
      status: "ok",
      data: { record_id: 41, created: true, idempotent_replay: false, stock_warnings: [] },
    });

    const { queryClient } = renderWithClient(<DisposalPage />);
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    await addSingleProduct(user);
    fireEvent.change(screen.getByLabelText("DP-001 の数量"), { target: { value: "2" } });
    fireEvent.change(screen.getByLabelText("DP-001 の理由"), { target: { value: "棚卸差異" } });
    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));

    await waitFor(() => {
      expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
    });
    expect(await screen.findByText("廃棄・破損を保存しました")).toBeInTheDocument();
    expect(screen.getByText("¥240")).toBeInTheDocument();
    const createCalls = mockCreateDisposal.mock.calls as [DisposalCreateRequest][];
    expect(createCalls[0][0].disposal_date).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    expect(createCalls[0][0].items).toEqual([
      {
        product_code: "DP-001",
        disposal_type: "damage",
        quantity: 2,
        cost_price_centi: 12000,
        reason: "棚卸差異",
      },
    ]);
    await waitFor(() => {
      expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.disposal());
    });
  });

  it("Codex round4 P2-class: 種別selectで全option（廃棄・破損・その他）の内部値が独立リテラルで検査される（初期値damageもonValueChange経由で確認）", async () => {
    const user = userEvent.setup();
    mockCreateDisposal.mockResolvedValue({
      status: "ok",
      data: { record_id: 41, created: true, idempotent_replay: false, stock_warnings: [] },
    });

    const DISPOSAL_TYPE_OPTIONS: [label: string, value: string][] = [
      ["廃棄", "disposal"],
      ["破損", "damage"],
      ["その他", "other"],
    ];
    for (const [label, expectedValue] of DISPOSAL_TYPE_OPTIONS) {
      mockCreateDisposal.mockClear();
      const { unmount } = renderWithClient(<DisposalPage />);
      await addSingleProduct(user);
      fireEvent.change(screen.getByLabelText("DP-001 の数量"), { target: { value: "2" } });
      fireEvent.change(screen.getByLabelText("DP-001 の理由"), { target: { value: "棚卸差異" } });

      await user.click(screen.getByLabelText("DP-001 の種別"));
      if (expectedValue === "damage") {
        // 初期値(破損)のままでは Radix が onValueChange を発火しないため、
        // 一度別の値へ切り替えてから破損へ戻す実操作を経由させる。
        await user.click(await screen.findByRole("option", { name: "廃棄" }));
        await user.click(screen.getByLabelText("DP-001 の種別"));
      }
      await user.click(await screen.findByRole("option", { name: label }));
      await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));

      await waitFor(() => {
        expect(mockCreateDisposal).toHaveBeenCalled();
      });
      const createCalls = mockCreateDisposal.mock.calls as [DisposalCreateRequest][];
      expect(createCalls[0][0].items[0]?.disposal_type).toBe(expectedValue);

      unmount();
    }
  });

  it("T8 UI-05-D17: saved disposal result links to the detail with returnTo", async () => {
    const user = userEvent.setup();
    mockCreateDisposal.mockResolvedValue({
      status: "ok",
      data: { record_id: 41, created: true, idempotent_replay: false, stock_warnings: [] },
    });

    renderWithClient(<DisposalPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));

    const heading = await screen.findByText("廃棄・破損を保存しました");
    const section = heading.closest("section");
    if (section === null) throw new Error("保存結果 section が見つかりません");
    expect(within(section).getByRole("link", { name: "詳細を見る" })).toHaveAttribute(
      "href",
      "/inventory/disposal/records/41?returnTo=%2Finventory%2Fdisposal",
    );
  });

  it("REQ-204 keeps the idempotency key for same-content retry and rotates it after edits or reset", async () => {
    const user = userEvent.setup();
    mockCreateDisposal
      .mockResolvedValueOnce({
        status: "error",
        error: {
          kind: "internal",
          message: "一時的なエラー",
          field: null,
          error_id: null,
          stocktake_recovery: null,
        },
      })
      .mockResolvedValueOnce({
        status: "error",
        error: {
          kind: "internal",
          message: "一時的なエラー",
          field: null,
          error_id: null,
          stocktake_recovery: null,
        },
      })
      .mockResolvedValueOnce({
        status: "ok",
        data: { record_id: 51, created: true, idempotent_replay: false, stock_warnings: [] },
      })
      .mockResolvedValueOnce({
        status: "ok",
        data: { record_id: 52, created: true, idempotent_replay: false, stock_warnings: [] },
      });

    renderWithClient(<DisposalPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));
    expect(
      await screen.findByText("一時的なエラー。詳細は診断ログに記録されています。"),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
    });
    const firstKey = mockCreateDisposal.mock.calls[0][0].idempotency_key;

    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));
    const sameContentRetryKey = mockCreateDisposal.mock.calls[1][0].idempotency_key;
    expect(sameContentRetryKey).toBe(firstKey);

    fireEvent.change(screen.getByLabelText("DP-001 の理由"), { target: { value: "棚卸差異" } });
    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));
    expect(await screen.findByText("廃棄・破損を保存しました")).toBeInTheDocument();
    const editedRetryKey = mockCreateDisposal.mock.calls[2][0].idempotency_key;
    expect(editedRetryKey).not.toBe(firstKey);

    await user.click(screen.getByRole("button", { name: "続けて廃棄・破損" }));
    await user.type(screen.getByLabelText("廃棄・破損商品検索"), "DP-001{enter}");
    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));
    const nextRecordKey = mockCreateDisposal.mock.calls[3][0].idempotency_key;
    expect(nextRecordKey).not.toBe(editedRetryKey);
  });

  it("REQ-204 UI-05-D15 unlocks product search directly after a successful reset", async () => {
    const user = userEvent.setup();
    mockCreateDisposal.mockResolvedValue({
      status: "ok",
      data: { record_id: 53, created: true, idempotent_replay: false, stock_warnings: [] },
    });

    renderWithClient(<DisposalPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));
    expect(await screen.findByText("廃棄・破損を保存しました")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "続けて廃棄・破損" }));
    mockSearchProducts.mockResolvedValueOnce({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({
            product_code: "DP-002",
            name: "reset直後の商品",
          }),
        ],
        total_count: 1,
        page: 1,
        per_page: 10,
      },
    });
    await user.type(screen.getByLabelText("廃棄・破損商品検索"), "DP-002{enter}");

    expect(await screen.findByText("reset直後の商品")).toBeInTheDocument();
    expect(screen.getByLabelText("DP-002 の数量")).toBeInTheDocument();
  });

  it("REQ-204 displays recent disposal records", async () => {
    mockListDisposals.mockResolvedValue({
      status: "ok",
      data: {
        items: [{ id: 12, disposal_date: "2026-06-27", created_at: "2026-06-27T10:30:00" }],
        total_count: 1,
        page: 1,
        per_page: 10,
      },
    });

    renderWithClient(<DisposalPage />);

    expect(await screen.findByText("2026-06-27")).toBeInTheDocument();
    expect(screen.getByText("12")).toBeInTheDocument();
    // ⑮ SC8: 種別固定の記録ID列を維持し備考列を足さない。
    expect(screen.getAllByRole("columnheader").map((cell) => cell.textContent)).toEqual([
      "廃棄日",
      "記録ID",
      "記録日時",
      "操作",
    ]);
    expect(screen.getByText("2026-06-27 10:30:00")).toBeInTheDocument();
    // ⑰ SC6 / UIDISP-D6: 記録日時セルの書体契約。
    expect(screen.getByRole("cell", { name: "2026-06-27 10:30:00" })).toHaveClass(
      "font-mono",
      "tabular-nums",
    );
    expect(screen.getByRole("link", { name: "すべての履歴を見る" })).toHaveAttribute(
      "href",
      "/inventory/records?recordType=disposal_record",
    );
    expect(screen.getByRole("link", { name: "詳細を見る" })).toHaveAttribute(
      "href",
      "/inventory/disposal/records/12?returnTo=%2Finventory%2Fdisposal",
    );
    expect(screen.getByRole("link", { name: "詳細を見る" })).toHaveClass("border");
  });

  it("REQ-204 hides product registration recovery and locks editing while saving", async () => {
    const user = userEvent.setup();
    const deferred = createDeferred<Awaited<ReturnType<typeof commands.createDisposal>>>();
    mockCreateDisposal.mockReturnValue(deferred.promise);

    renderWithClient(<DisposalPage />);
    await addSingleProduct(user);
    mockSearchProducts.mockResolvedValueOnce({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 10 },
    });
    await user.type(screen.getByLabelText("廃棄・破損商品検索"), "NO-HIT{enter}");
    expect(await screen.findByRole("link", { name: "商品登録へ進む" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));

    await waitFor(() => {
      expect(screen.queryByRole("link", { name: "商品登録へ進む" })).not.toBeInTheDocument();
    });
    expect(screen.getByLabelText("DP-001 の数量")).toBeDisabled();
    expect(screen.getByLabelText("DP-001 の種別")).toBeDisabled();

    deferred.resolve({
      status: "ok",
      data: { record_id: 42, created: true, idempotent_replay: false, stock_warnings: [] },
    });
    expect(await screen.findByText("廃棄・破損を保存しました")).toBeInTheDocument();
  });

  it("REQ-204 ignores late product search results after the form is locked", async () => {
    const user = userEvent.setup();
    const deferredSearch = createDeferred<Awaited<ReturnType<typeof commands.searchProducts>>>();
    mockCreateDisposal.mockResolvedValue({
      status: "ok",
      data: { record_id: 43, created: true, idempotent_replay: false, stock_warnings: [] },
    });

    renderWithClient(<DisposalPage />);
    await addSingleProduct(user);
    mockSearchProducts.mockReturnValueOnce(deferredSearch.promise);

    await user.type(screen.getByLabelText("廃棄・破損商品検索"), "DP-002{enter}");
    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));
    expect(await screen.findByText("廃棄・破損を保存しました")).toBeInTheDocument();

    deferredSearch.resolve({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "DP-002", name: "遅延検索商品" })],
        total_count: 1,
        page: 1,
        per_page: 10,
      },
    });

    await waitFor(() => {
      expect(screen.queryByText("遅延検索商品")).not.toBeInTheDocument();
    });
    expect(screen.getByText("廃棄・破損を保存しました")).toBeInTheDocument();
    expect(screen.queryByText("DP-002")).not.toBeInTheDocument();
  });

  it("REQ-204 locks late product results at the save event before the pending render", async () => {
    const user = userEvent.setup();
    const deferredSearch = createDeferred<Awaited<ReturnType<typeof commands.searchProducts>>>();
    const deferredSave = createDeferred<Awaited<ReturnType<typeof commands.createDisposal>>>();
    mockCreateDisposal.mockImplementation(() => {
      deferredSearch.resolve({
        status: "ok",
        data: {
          items: [
            makeMockProductWithRelations({
              product_code: "DP-002",
              name: "保存event競合商品",
            }),
          ],
          total_count: 1,
          page: 1,
          per_page: 10,
        },
      });
      return deferredSave.promise;
    });

    renderWithClient(<DisposalPage />);
    await addSingleProduct(user);
    mockSearchProducts.mockReturnValueOnce(deferredSearch.promise);
    await user.type(screen.getByLabelText("廃棄・破損商品検索"), "DP-002{enter}");

    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));

    expect(await screen.findByRole("button", { name: "保存中..." })).toBeDisabled();
    expect(screen.queryByText("保存event競合商品")).not.toBeInTheDocument();
    expect(screen.queryByText("DP-002")).not.toBeInTheDocument();

    deferredSave.resolve({
      status: "ok",
      data: { record_id: 44, created: true, idempotent_replay: false, stock_warnings: [] },
    });
    expect(await screen.findByText("廃棄・破損を保存しました")).toBeInTheDocument();
  });

  it("REQ-204 unlocks product search after frontend validation failure", async () => {
    const user = userEvent.setup();
    renderWithClient(<DisposalPage />);
    await addSingleProduct(user);
    fireEvent.change(screen.getByLabelText("DP-001 の数量"), { target: { value: "0" } });

    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));
    expect(
      await screen.findByText("DP-001: 数量は1以上の整数で入力してください"),
    ).toBeInTheDocument();

    mockSearchProducts.mockResolvedValueOnce({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({
            product_code: "DP-002",
            name: "validation失敗後の商品",
          }),
        ],
        total_count: 1,
        page: 1,
        per_page: 10,
      },
    });
    await user.type(screen.getByLabelText("廃棄・破損商品検索"), "DP-002{enter}");
    expect(await screen.findByText("validation失敗後の商品")).toBeInTheDocument();
  });

  it("REQ-204 unlocks product search after command failure", async () => {
    const user = userEvent.setup();
    mockCreateDisposal.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "保存command失敗",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    renderWithClient(<DisposalPage />);
    await addSingleProduct(user);

    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));
    expect(
      await screen.findByText("保存command失敗。詳細は診断ログに記録されています。"),
    ).toBeInTheDocument();

    mockSearchProducts.mockResolvedValueOnce({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({
            product_code: "DP-002",
            name: "command失敗後の商品",
          }),
        ],
        total_count: 1,
        page: 1,
        per_page: 10,
      },
    });
    await user.type(screen.getByLabelText("廃棄・破損商品検索"), "DP-002{enter}");
    expect(await screen.findByText("command失敗後の商品")).toBeInTheDocument();
  });

  // describe-error-adoption packet（2026-08-04）AC4 / A群代表: internal kind fixture
  // （error_id 付き）で保存失敗 → describeError 出力に `エラーID:` が含まれることを assert する
  // （UI-ERR-D1、A1 是正）。
  it("REQ-204 shows the diagnostic error ID for an internal kind save failure", async () => {
    const user = userEvent.setup();
    mockCreateDisposal.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "廃棄・破損の保存でエラーが発生しました",
        field: null,
        error_id: "E-20260627-101500-syn5",
        stocktake_recovery: null,
      },
    });
    renderWithClient(<DisposalPage />);
    await addSingleProduct(user);

    await user.click(screen.getByRole("button", { name: "廃棄・破損を保存" }));

    expect(
      await screen.findByText(
        "廃棄・破損の保存でエラーが発生しました（エラーID: E-20260627-101500-syn5）。詳細は診断ログに記録されています。",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText(/エラーID:/)).toBeInTheDocument();
    expect(screen.queryByText(/\[commands:/)).not.toBeInTheDocument();
  });

  it("SC1: per-row の種別selectは他行の表示値へ波及しない（L8-D3）", async () => {
    const user = userEvent.setup();
    renderWithClient(<DisposalPage />);
    await addSingleProduct(user);
    mockSearchProducts.mockResolvedValueOnce({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "DP-002", name: "商品B" })],
        total_count: 1,
        page: 1,
        per_page: 10,
      },
    });
    await user.type(screen.getByLabelText("廃棄・破損商品検索"), "DP-002{enter}");
    expect(await screen.findByLabelText("DP-002 の種別")).toBeInTheDocument();

    await user.click(screen.getByRole("combobox", { name: "DP-001 の種別" }));
    await user.click(await screen.findByRole("option", { name: "廃棄" }));

    expect(screen.getByLabelText("DP-001 の種別")).toHaveTextContent("廃棄");
    expect(screen.getByLabelText("DP-002 の種別")).toHaveTextContent("破損");

    // row 2 自身の select を直接操作しても row 2 だけが変わること
    // （updateDisposalRow が常に rows[0] を書き換える mutant を kill する）。
    await user.click(screen.getByRole("combobox", { name: "DP-002 の種別" }));
    await user.click(await screen.findByRole("option", { name: "その他" }));

    expect(screen.getByLabelText("DP-002 の種別")).toHaveTextContent("その他");
    expect(screen.getByLabelText("DP-001 の種別")).toHaveTextContent("廃棄");
  });
});

describe("DisposalPage native input tokens（Lane 5 SC4c）", () => {
  it("SC4c: 種別selectがbg-control-surfaceでbg-backgroundを持たない", async () => {
    const user = userEvent.setup();
    renderWithClient(<DisposalPage />);
    await addSingleProduct(user);

    const disposalType = screen.getByLabelText("DP-001 の種別");
    expect(disposalType).toHaveClass("bg-control-surface");
    expect(disposalType).not.toHaveClass("bg-background");
  });
});

it("⑮ SC3/SC14: 副題と直近件数の説明を表示する", () => {
  renderWithClient(<DisposalPage />);
  expect(
    screen.getByText("販売ではない理由で在庫を減らし、ロス理由と原価を記録します"),
  ).toBeInTheDocument();
  expect(screen.getByText("直近 10 件の廃棄・破損を新しい順に表示します。")).toBeInTheDocument();
});

it.each([
  ["pcs", "1,234 個", "個"],
  ["m", "12.34 m", "m"],
] as const)(
  "⑮ SC19: %s は現在庫を単位付きで表示し、数量と単位を同じ cell に添える",
  async (unit, display, unitLabel) => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({
            product_code: "UNIT-001",
            name: "単位確認A",
            stock_quantity: 1234,
            stock_unit: unit,
          }),
          makeMockProductWithRelations({ product_code: "UNIT-002", name: "単位確認B" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 10,
      },
    });
    renderWithClient(<DisposalPage />);
    await user.type(await screen.findByLabelText("廃棄・破損商品検索"), "単位{enter}");
    const candidate = (await screen.findByText("UNIT-001")).closest("tr");
    if (candidate === null) throw new Error("expected table structure");
    expect(within(candidate).getByText(display)).toBeInTheDocument();
    await user.click(within(candidate).getByRole("button", { name: "廃棄・破損に追加" }));
    const inputRow = (await screen.findByLabelText("UNIT-001 の数量")).closest("tr");
    if (inputRow === null) throw new Error("expected table structure");
    // SC19 Amendment 3: 入力表の列順と、数量・単位が同じ cell にある契約を固定する。
    const inputTable = inputRow.closest("table");
    if (inputTable === null) throw new Error("expected table structure");
    expect(
      within(inputTable)
        .getAllByRole("columnheader")
        .map((cell) => cell.textContent),
    ).toEqual(["商品コード", "商品名", "部門", "現在庫", "種別", "数量", "原価", "理由", "操作"]);
    const cells = within(inputRow).getAllByRole("cell");
    expect(cells[3].textContent).toBe(display);
    expect(
      within(cells[5]).getByRole("spinbutton", { name: "UNIT-001 の数量" }),
    ).toBeInTheDocument();
    expect(within(cells[5]).getByText(unitLabel)).toBeInTheDocument();
    expect(
      within(cells[6]).getByRole("spinbutton", { name: "UNIT-001 の原価" }),
    ).toBeInTheDocument();
    expect(within(cells[6]).getByText("円")).toBeInTheDocument();
    expect(within(cells[7]).getByLabelText("UNIT-001 の理由")).toBeInTheDocument();
  },
);

it("⑰ SC6 / UIDISP-D6: 共有 formatDateTime を import しローカル定義を持たない", () => {
  expect(dateTimeSource).toMatch(
    /import\s*\{[^}]*\bformatDateTime\b[^}]*\}\s*from\s*"@\/lib\/date-time"/,
  );
  expect(dateTimeSource).not.toMatch(/function\s+(?:formatDateTime|formatCheckedAt)\s*\(/);
  expect(dateTimeSource).not.toContain("formatCheckedAt");
});

// REQ-204 / UI-05-D8 / SPEC-UNIT-D3: 数量欄の属性は m の商品の行だけ小数 2 桁の入力にする
it.each([
  ["m", { inputmode: "decimal", min: "0.01", step: "0.01" }],
  ["pcs", { inputmode: "numeric", min: "1", step: null }],
] as const)("UI-05-D8: %s の商品の数量欄の属性", async (unit, expected) => {
  const user = userEvent.setup();
  mockSearchProducts.mockResolvedValue({
    status: "ok",
    data: {
      items: [
        makeMockProductWithRelations({
          product_code: "ATTR-001",
          name: "属性確認A",
          stock_unit: unit,
        }),
        makeMockProductWithRelations({ product_code: "ATTR-002", name: "属性確認B" }),
      ],
      total_count: 2,
      page: 1,
      per_page: 10,
    },
  });
  renderWithClient(<DisposalPage />);
  await user.type(await screen.findByLabelText("廃棄・破損商品検索"), "属性{enter}");
  const candidate = (await screen.findByText("ATTR-001")).closest("tr");
  if (candidate === null) throw new Error("expected table structure");
  await user.click(within(candidate).getByRole("button", { name: "廃棄・破損に追加" }));
  const input = await screen.findByLabelText("ATTR-001 の数量");
  expect(input).toHaveValue(1);
  expect(input).toHaveAttribute("type", "number");
  expect(input.getAttribute("inputmode")).toBe(expected.inputmode);
  expect(input.getAttribute("min")).toBe(expected.min);
  expect(input.getAttribute("step")).toBe(expected.step);
});
