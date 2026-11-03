import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import type { PushPreview } from "../services/response.generated";
import { PushPreviewResult } from "./PushPreviewResult";
afterEach(cleanup);
const preview: PushPreview = {
  source_ref: "refs/heads/main",
  source_oid: { format: "sha1", hex: "a".repeat(40) },
  remote: "origin",
  destination: "https://example.invalid/<img>",
  destination_ref: "refs/heads/main",
  observed_oid: { format: "sha1", hex: "b".repeat(40) },
  observation_id: "past",
  observed_at_ms: 0,
  generated_at_ms: 1000,
  generation: "snapshot",
  config_digest: "config",
  relation: "fast_forward",
  outgoing_count: 101,
  incoming_count: 0,
  outgoing: [
    {
      oid: { format: "sha1", hex: "a".repeat(40) },
      subject: Array.from(new TextEncoder().encode("Outgoing <script>")),
      subject_truncated: true,
    },
  ],
  outgoing_truncated: true,
  source_gitlinks: [],
  upstream_matches: false,
};
it("renders a bounded historical comparison without execution controls or HTML interpretation", () => {
  const { container } = render(<PushPreviewResult preview={preview} />);
  expect(screen.getByRole("status")).toHaveTextContent("not push approval");
  expect(screen.getByText(/101 outgoing/)).toHaveTextContent("0 incoming");
  expect(screen.getByText(/1 of 101/)).toHaveTextContent("display limited");
  expect(screen.getByText(/Outgoing <script>/)).toHaveTextContent(
    "subject truncated",
  );
  expect(container.querySelector("script,img")).toBeNull();
  expect(screen.queryByRole("button")).toBeNull();
  expect(screen.queryByRole("alert")).toBeNull();
});
it("reports divergence relative to the observation with no force action", () => {
  render(
    <PushPreviewResult
      preview={{ ...preview, relation: "diverged", incoming_count: 2 }}
    />,
  );
  expect(screen.getByRole("alert")).toHaveTextContent(
    "not a fast forward relative to the observed commit",
  );
  expect(screen.queryByRole("button")).toBeNull();
});
it("shows committed child requirements without implying remote availability", () => {
  const { container } = render(
    <PushPreviewResult
      preview={{
        ...preview,
        source_gitlinks: [
          {
            path: Array.from(new TextEncoder().encode("nested/<img>")),
            oid: { format: "sha1", hex: "c".repeat(40) },
          },
        ],
      }}
    />,
  );
  expect(
    screen.getByRole("heading", { name: "Child commits in source · 1" }),
  ).toBeVisible();
  expect(screen.getByText(/nested\/<img>/)).toHaveTextContent("c".repeat(40));
  expect(
    screen.getByText(/Child checkouts and remote availability/),
  ).toHaveTextContent("separate proof");
  expect(container.querySelector("img")).toBeNull();
  expect(screen.queryByRole("button")).toBeNull();
});
