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

it("shows resumable progress and explicitly continues without a full rescan", async () => {
  const { vi } = await import("vitest");
  const onContinue = vi.fn();
  const { rerender } = render(
    <WorkspaceScanStatus
      onContinue={onContinue}
      scan={{
        complete: false,
        issues: [],
        skipped_caches: [],
        progress: {
          inspected_repositories: 93,
          pending_directories: 7,
          classified_repositories: 0,
          phase: "traversal",
        },
      }}
    />,
  );
  expect(screen.getByText(/93 repositories inspected/)).toBeVisible();
  await userEvent.click(
    screen.getByRole("button", { name: "Continue discovery" }),
  );
  expect(onContinue).toHaveBeenCalledOnce();
  rerender(
    <WorkspaceScanStatus
      scan={{ complete: false, issues: [], skipped_caches: [] }}
    />,
  );
  expect(
    screen.queryByRole("button", { name: "Continue discovery" }),
  ).not.toBeInTheDocument();
});

it("distinguishes a time-sliced background scan from an inspection failure", async () => {
  const scan = {
    complete: false,
    issues: [
      {
        path: { encoding: "utf8" as const, value: "/workspace" },
        error: { kind: "timeout" as const, message: "Read deadline exceeded" },
      },
    ],
    skipped_caches: [],
    progress: {
      inspected_repositories: 5,
      pending_directories: 12,
      classified_repositories: 0,
      phase: "traversal",
    },
  };
  const { rerender } = render(<WorkspaceScanStatus scan={scan} />);
  expect(screen.getByText("Scanning repositories…")).toBeVisible();
  await userEvent.click(screen.getByText("Discovery details"));
  expect(
    screen.getByText(
      /Previously inspected repositories keep their last known status/,
    ),
  ).toBeVisible();
  expect(screen.queryByText(/timeout:/)).not.toBeInTheDocument();
  rerender(
    <WorkspaceScanStatus
      scan={{
        ...scan,
        issues: [
          {
            path: scan.issues[0].path,
            error: {
              kind: "inspection_failed",
              message: "Cannot inspect metadata",
            },
          },
        ],
      }}
    />,
  );
  expect(screen.getByText("△ Discovery incomplete")).toBeVisible();
  expect(
    screen.getByText(/inspection_failed: Cannot inspect metadata/),
  ).toBeVisible();
});
