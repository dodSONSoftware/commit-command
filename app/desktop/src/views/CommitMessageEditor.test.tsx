import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";
import {
  CommitMessageEditor,
  replaceBody,
  replaceSubject,
  splitMessage,
} from "./CommitMessageEditor";

afterEach(cleanup);
it("splits only the first blank line and preserves all untouched draft characters", () => {
  for (const raw of [
    "",
    "Subject",
    "Subject\ncontinuation",
    "Subject\n\nBody\n\n",
    "Subject\r\n \t\r\nBody\r\n\r\n",
    "\n\nLeading body\n",
    "  spaced subject  \n\n  spaced body  \n",
  ]) {
    const parts = splitMessage(raw);
    expect(parts.subject + parts.separator + parts.body).toBe(raw);
    expect(replaceSubject(raw, parts.subject)).toBe(raw);
    expect(replaceBody(raw, parts.body)).toBe(raw);
  }
  expect(splitMessage("Subject\n\nBody\n\nFurther paragraph")).toEqual({
    subject: "Subject",
    separator: "\n\n",
    body: "Body\n\nFurther paragraph",
  });
});
it("preserves the exact separator and untouched body when editing a subject", () => {
  expect(
    replaceSubject("Original\r\n \t\r\nBody\r\n\r\n", "New subject  "),
  ).toBe("New subject  \r\n \t\r\nBody\r\n\r\n");
  expect(replaceBody("Original\r\n \t\r\nBody\r\n", "New body  \n")).toBe(
    "Original\r\n \t\r\nNew body  \n",
  );
  expect(replaceBody("Subject  ", "Body\n")).toBe("Subject  \n\nBody\n");
  expect(replaceBody("Subject\n\nBody", "")).toBe("Subject\n\n");
});
it("switches layouts without emitting a draft mutation and composes structured edits", async () => {
  const change = vi.fn();
  const user = userEvent.setup();
  const original = "Original\r\n \t\r\nBody\r\n\r\n";
  render(
    <CommitMessageEditor label="Message" value={original} onChange={change} />,
  );
  await user.click(
    screen.getByRole("button", { name: "Edit subject and body" }),
  );
  expect(change).not.toHaveBeenCalled();
  fireEvent.change(screen.getByRole("textbox", { name: "Message subject" }), {
    target: { value: "New subject  " },
  });
  expect(change).toHaveBeenLastCalledWith(
    "New subject  \r\n \t\r\nBody\r\n\r\n",
  );
  change.mockClear();
  await user.click(screen.getByRole("button", { name: "Edit raw message" }));
  expect(change).not.toHaveBeenCalled();
});
it("keeps both editing modes disabled during a bulk job", () => {
  render(
    <CommitMessageEditor
      label="Shared commit message"
      value="draft"
      onChange={vi.fn()}
      disabled
    />,
  );
  expect(
    screen.getByRole("textbox", { name: "Shared commit message" }),
  ).toBeDisabled();
  expect(
    screen.getByRole("button", { name: "Edit subject and body" }),
  ).toBeDisabled();
});
