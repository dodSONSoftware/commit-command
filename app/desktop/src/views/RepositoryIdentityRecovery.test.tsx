import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { RepositoryIdentityRecovery } from "./RepositoryIdentityRecovery";
import { fixtureEntry } from "../testdata";
afterEach(cleanup);
it("separates historical identities, preserves drafts and opens only the explicit replacement", async () => {
  const old = {
    ...fixtureEntry("old", "repo"),
    availability: "missing" as const,
    superseded_by: "new",
  };
  const current = fixtureEntry("new", "repo");
  const onOpen = vi.fn();
  const onRescan = vi.fn().mockResolvedValue(undefined);
  render(
    <RepositoryIdentityRecovery
      entries={[old, current]}
      drafts={{ old: "My original draft" }}
      busy={false}
      onOpen={onOpen}
      onRescan={onRescan}
    />,
  );
  expect(onOpen).not.toHaveBeenCalled();
  await userEvent.click(
    screen.getByText("Historical repository identities · 1"),
  );
  expect(screen.getByRole("textbox")).toHaveValue("My original draft");
  expect(screen.getByRole("textbox")).toHaveAttribute("readonly");
  expect(
    screen.getByText(/trust and approvals are never transferred/),
  ).toBeVisible();
  await userEvent.click(
    screen.getByRole("button", { name: "Open inspected replacement" }),
  );
  expect(onOpen).toHaveBeenCalledExactlyOnceWith(current);
  await userEvent.click(
    screen.getByRole("button", { name: "Rescan and review" }),
  );
  expect(onRescan).toHaveBeenCalledOnce();
});
it("keeps missing or inaccessible locations unresolved and never offers an unavailable replacement", async () => {
  const old = { ...fixtureEntry("old", "repo"), superseded_by: "new" };
  const missing = {
    ...fixtureEntry("new", "repo"),
    availability: "unavailable" as const,
  };
  render(
    <RepositoryIdentityRecovery
      entries={[old, missing]}
      drafts={{}}
      busy={true}
      onOpen={vi.fn()}
      onRescan={vi.fn()}
    />,
  );
  expect(screen.getByText("Needs inspection · 1")).toBeVisible();
  expect(
    screen.getByRole("button", { name: "Rescan and review" }),
  ).toBeDisabled();
  await userEvent.click(
    screen.getByText("Historical repository identities · 1"),
  );
  expect(
    screen.queryByRole("button", { name: "Open inspected replacement" }),
  ).toBeNull();
  expect(
    screen.getByText(
      /previously inspected replacement is currently unavailable/,
    ),
  ).toBeVisible();
});
it("does not show recovery when all identities are current", () => {
  render(
    <RepositoryIdentityRecovery
      entries={[fixtureEntry("new", "repo")]}
      drafts={{}}
      busy={false}
      onOpen={vi.fn()}
      onRescan={vi.fn()}
    />,
  );
  expect(screen.queryByRole("region")).toBeNull();
});
