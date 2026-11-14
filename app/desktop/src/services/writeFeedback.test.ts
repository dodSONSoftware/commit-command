import { expect, it } from "vitest";
import { writeFeedback } from "./writeFeedback";
import type { Payload } from "./bridge";
import type { WriteRequest } from "./request.generated";
const stage: WriteRequest = {
  kind: "stage",
  paths: [{ encoding: "utf8", value: "a" }],
  hooks: "execute",
};
function executed(phase: string, published = false): Payload {
  return {
    kind: "executed",
    value: { result: { phase, published, reason: "Fixture failure" } },
  };
}
it("reports staging success and blocked outcomes separately", () => {
  expect(writeFeedback(executed("complete"), stage, 1)).toEqual({
    error: false,
    completed: true,
    message: "1 file staged.",
  });
  expect(writeFeedback(executed("blocked"), stage, 1)).toEqual({
    error: true,
    message:
      "Could not stage files. The staging area was not changed. Fixture failure",
  });
});
it("does not advise retrying a commit that already exists or has an uncertain outcome", () => {
  expect(
    writeFeedback(executed("published_needs_reconciliation", true), stage, 1)
      .message,
  ).toContain("Do not commit again");
  expect(
    writeFeedback(executed("uncertain_outcome"), stage, 1).message,
  ).toContain("before retrying");
  expect(writeFeedback({ kind: "done" }, stage, 1).message).toContain(
    "before retrying",
  );
});

it("reports deliberate cancellation without calling it a failure", () => {
  expect(
    writeFeedback(executed("cancelled_before_publication"), stage, 1),
  ).toEqual({
    error: false,
    completed: false,
    message: "Staging cancelled. The staging area was not changed.",
  });
});

it.each(["stage", "unstage"] as const)(
  "reports the applied %s before the hook failure",
  (kind) => {
    const response: Payload = {
      kind: "executed",
      value: {
        result: {
          phase: "published_with_post_processing_failure",
          published: false,
          index_reconciled: true,
          reason: "post-index-change failed after index/ref publication",
        },
      },
    };
    const feedback = writeFeedback(response, { ...stage, kind }, 1);
    expect(feedback.error).toBe(true);
    expect(feedback.message).toContain(
      kind === "stage" ? "1 file staged." : "1 file unstaged.",
    );
    expect(feedback.message).toContain("post-index-change");
  },
);
