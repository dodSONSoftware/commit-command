import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { ReviewedHttpsAuthentication } from "./ReviewedHttpsAuthentication";
afterEach(cleanup);
it("shows only reviewed account scope as inert text and grants no new lookup", () => {
  const { container, rerender } = render(<ReviewedHttpsAuthentication />);
  expect(container).toBeEmptyDOMElement();
  rerender(
    <ReviewedHttpsAuthentication
      scope={{
        account: "<img src=x onerror=alert(1)>",
        host: "example.invalid",
        path: "team/repo.git",
      }}
    />,
  );
  expect(screen.getByText(/Reviewed HTTPS account:/)).toHaveTextContent(
    "<img src=x onerror=alert(1)>",
  );
  expect(container.querySelector("img")).toBeNull();
  expect(
    screen.getByText(/does not grant another credential lookup/),
  ).toBeVisible();
});
