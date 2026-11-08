// Intentionally independent of React and repository-provided HTML. Only text is rendered.
import { invoke } from "@tauri-apps/api/core";
import type { ReviewDisplay } from "./services/response.generated";
import "./styles/review.css";
const root = document.getElementById("review")!;
function element<K extends keyof HTMLElementTagNameMap>(tag: K, text: string) {
  const node = document.createElement(tag);
  node.textContent = text;
  return node;
}
async function load() {
  const review = await invoke<ReviewDisplay & { can_confirm: boolean }>(
    "review_data",
  );
  root.replaceChildren();
  root.append(
    element("p", "COMMIT COMMAND · AUTHORITATIVE RUST REVIEW"),
    element("h1", review.title),
    element(
      "p",
      "This isolated view reads the stored plan directly from Rust. Main-window labels and checkboxes cannot authorize it.",
    ),
  );
  const summary = element("pre", review.summary);
  summary.className = "summary";
  root.append(summary);
  root.append(
    element(
      "p",
      `Review ${review.id} · expires ${new Date(review.expires_at_ms).toLocaleTimeString()}`,
    ),
  );
  const content = element("pre", review.content);
  content.tabIndex = 0;
  content.setAttribute("aria-label", "Exact stored operation review");
  root.append(content);
  const controls = document.createElement("div");
  controls.className = "controls";
  const cancel = element("button", "Cancel review");
  cancel.onclick = () => {
    invoke("review_cancel", { review: review.id }).catch((e) => {
      status.textContent = String(e);
    });
  };
  const confirm = element("button", "Continue to native confirmation");
  confirm.className = "primary";
  confirm.disabled = !review.can_confirm;
  const status = element("pre", "No operation is approved yet.");
  status.setAttribute("role", "status");
  if (!review.can_confirm)
    status.textContent =
      "Read-only desktop: execution is disabled pending native acceptance.";
  const expire = () => {
    confirm.disabled = true;
    cancel.textContent = "Close review";
    status.textContent =
      "Review expired. Prepare a fresh proposal. No operation is approved.";
  };
  const expiry = window.setTimeout(
    expire,
    Math.max(0, review.expires_at_ms - Date.now()),
  );
  if (Date.now() >= review.expires_at_ms) expire();
  confirm.onclick = async () => {
    if (Date.now() >= review.expires_at_ms) {
      expire();
      return;
    }
    // Core still checks expiry after the independent native dialog returns.
    window.clearTimeout(expiry);
    confirm.disabled = true;
    status.textContent =
      "Waiting for native approval or the approved operation to finish. Closing or cancelling requests a stop; it does not undo effects already made.";
    try {
      const result = await invoke<unknown>("review_confirm", {
        review: review.id,
      });
      status.textContent = `Core result — inspect the exact outcome below:\n${JSON.stringify(result, null, 2)}`;
      cancel.textContent = "Close review";
    } catch (e) {
      status.textContent = `Not completed: ${String(e)}\nA consumed or stale review requires a fresh proposal.`;
    }
  };
  controls.append(cancel, confirm);
  root.append(controls, status);
  cancel.focus();
}
load().catch((e) => {
  root.replaceChildren(
    element("h1", "Review unavailable"),
    element("p", String(e)),
  );
});
