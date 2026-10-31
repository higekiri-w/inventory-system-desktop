import { Outlet, createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/inventory/transfer")({
  component: TransferLayout,
});

function TransferLayout() {
  return <Outlet />;
}
