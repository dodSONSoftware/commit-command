import { afterEach, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { WorkspaceScanStatus } from "./WorkspaceScanStatus";
afterEach(cleanup);
it("shows partial discovery even independently of repository attention counts", async () => {
  render(
    <WorkspaceScanStatus
      scan={{
        complete: false,
        issues: [
          {
            path: { encoding: "utf8", value: "/workspace/broken" },
            error: {
              kind: "inspection_failed",
              message: "Cannot inspect metadata",
            },
          },
        ],
        skipped_caches: [],
      }}
    />,
  );
  expect(screen.getByRole("status")).toHaveTextContent("Discovery incomplete");
  expect(screen.getByRole("status")).toHaveTextContent("1 scan issue");
  await userEvent.click(screen.getByText("Discovery details"));
  expect(screen.getByText("/workspace/broken")).toBeVisible();
  expect(
    screen.getByText(/inspection_failed: Cannot inspect metadata/),
  ).toBeVisible();
  expect(
    screen.getByText(/attention counts do not include scan-level issues/),
  ).toBeVisible();
});
it("reports complete within exclusions and explicitly lists skipped tagged caches", async () => {
  render(
    <WorkspaceScanStatus
      scan={{
        complete: true,
        issues: [],
        skipped_caches: [{ encoding: "utf8", value: "/workspace/cache" }],
      }}
    />,
  );
  expect(screen.getByRole("status")).toHaveTextContent(
    "Discovery complete · 0 scan issues · 1 tagged cache skipped",
  );
  await userEvent.click(screen.getByText("Discovery details"));
  expect(screen.getByText("/workspace/cache")).toBeVisible();
  expect(screen.getByText(/valid CACHEDIR.TAG signature/)).toBeVisible();
});
it("does not imply complete discovery without a saved scan", () => {
  render(<WorkspaceScanStatus />);
  expect(screen.getByRole("status")).toHaveTextContent(
    "Discovery not yet scanned",
  );
});
