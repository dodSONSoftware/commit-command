import { expect, it } from "vitest";
import { publicationStatus } from "./publication";

it("does not interpret an unconfirmed publication as proof that the ref stayed unchanged", () => {
  expect(
    publicationStatus({ published: false, phase: "uncertain_outcome" }),
  ).toBe("Publication unconfirmed — inspect guarded recovery");
  expect(
    publicationStatus({ published: false, phase: "reference_updated" }),
  ).toMatch(/Publication unconfirmed/);
  expect(
    publicationStatus({
      published: true,
      phase: "published_needs_reconciliation",
    }),
  ).toBe("Published");
  expect(
    publicationStatus({
      published: false,
      phase: "cancelled_before_publication",
    }),
  ).toBe("Not published");
  expect(publicationStatus(undefined)).toBe("No publication result recorded");
});
