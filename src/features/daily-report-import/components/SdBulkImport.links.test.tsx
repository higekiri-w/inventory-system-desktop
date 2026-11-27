// UI-07-D17 の (A-2) の分（Test Plan L1・L2）: 一括の結果から控えの画面へ。実の Router と控えの画面の search の schema で、
// link を押した先が受ける値を見る（href の完全一致は見ない。PR #305 の closure P3）
import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import type { EjImportResult, RegisterSdBulkResponse } from "@/lib/bindings";
import { ejReceiptsSearchSchema } from "@/features/ej-receipts/types";
import { SdBulkImport } from "./SdBulkImport";

function ejResult(overrides: Partial<EjImportResult> = {}): EjImportResult {
  return {
    inserted_count: 3,
    already_count: 0,
    conflict_count: 0,
    conflicts: [],
    held_count: 0,
    unreadable_files: [],
    gaps: [],
    ...overrides,
  };
}

function renderInRouter(result: EjImportResult) {
  const response: RegisterSdBulkResponse = {
    items: [{ source: { type: "Ej" }, outcome: { type: "ImportedEj", ...result } }],
  };
  const rootRoute = createRootRoute({ component: () => <Outlet /> });
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/",
    component: () => (
      <SdBulkImport
        scan={{
          volume_label: "E:",
          scanned_from: "2026-11-01",
          candidates: [],
          skipped_unknown_count: 0,
          tidy_failure: null,
          ej: null,
        }}
        busy={false}
        importing={false}
        response={response}
        ejManual={{ result: null, error: null, pending: false }}
        onBulkImport={() => undefined}
        onChooseEjFiles={() => undefined}
        onConfirmCandidate={() => undefined}
      />
    ),
  });
  const receiptsRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/reports/receipts",
    validateSearch: ejReceiptsSearchSchema,
    component: function Receipts() {
      return <output>{JSON.stringify(receiptsRoute.useSearch())}</output>;
    },
  });
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute, receiptsRoute]),
    history: createMemoryHistory({ initialEntries: ["/"] }),
  });
  render(<RouterProvider router={router} />);
  return router;
}

describe("SdBulkImport links to UI-09c REQ-908", () => {
  it("L1: 結果の下の「レシートの控えを見る」で控えの画面が開く", async () => {
    const user = userEvent.setup();
    const router = renderInRouter(ejResult());
    await user.click(await screen.findByRole("link", { name: "レシートの控えを見る" }));
    expect(await screen.findByRole("status")).toHaveTextContent("{}");
    expect(router.state.location.pathname).toBe("/reports/receipts");
  });

  it("L2: saved が true の食い違いの番号だけに link を出し、押すと控えの画面が文字列の番号を受ける", async () => {
    const user = userEvent.setup();
    renderInRouter(
      ejResult({
        conflict_count: 2,
        conflicts: [
          { machine_prefix: "0001", record_no: "100000", saved: true },
          { machine_prefix: "0001", record_no: "000200", saved: false },
        ],
      }),
    );
    expect(await screen.findByText(/前の控えは「レシートの控え」で見られます/)).toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "000200" })).not.toBeInTheDocument();
    expect(screen.getByText(/記録 000200: 同じ番号の控えが食い違っていたため/)).toBeInTheDocument();
    await user.click(screen.getByRole("link", { name: "100000" }));
    expect(await screen.findByRole("status")).toHaveTextContent('{"no":"100000"}');
  });

  it("L2: saved が true の食い違いが無ければ控えの画面への番号の文を出さない", async () => {
    renderInRouter(
      ejResult({
        conflict_count: 1,
        conflicts: [{ machine_prefix: "0001", record_no: "000200", saved: false }],
      }),
    );
    expect(await screen.findByText(/記録 000200/)).toBeInTheDocument();
    expect(screen.queryByText(/前の控えは「レシートの控え」で見られます/)).not.toBeInTheDocument();
  });
});
