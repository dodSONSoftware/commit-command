import { afterEach, expect, it, vi } from "vitest";
import {
  cleanup,
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { Bridge } from "../services/bridge";
import type { AiSettingsSnapshot } from "../services/response.generated";
import { SingleAiSetup } from "./SingleAiSetup";
afterEach(cleanup);
const initial: AiSettingsSnapshot = {
  fingerprint: "before",
  settings: {
    version: 1,
    revision: "revision",
    connections: [],
    profiles: [],
    assignments: [],
    instructions: [],
    templates: [],
    global_concurrency: 1,
    connection_concurrency: 1,
  },
};
function fixture(
  failed = false,
  saved = initial,
  options: { beforeSave?: () => Promise<void> } = {},
) {
  let flushSave: () => Promise<boolean> = async () => true;
  let current = structuredClone(saved);
  const request = vi.fn<Bridge["request"]>(async (r) => {
    if (
      r.kind === "review_ai_discovery" ||
      (r.kind === "save_ai_settings" && r.change.kind === "single_setup")
    ) {
      const connection =
        r.kind === "review_ai_discovery"
          ? r.connection
          : r.change.kind === "single_setup"
            ? r.change.connection
            : null;
      if (
        connection &&
        Object.keys(connection).some(
          (k) =>
            ![
              "id",
              "name",
              "provider",
              "endpoint",
              "timeout_ms",
              "authentication",
            ].includes(k),
        )
      )
        throw new Error("Malformed or unsupported request");
    }
    if (r.kind === "ai_settings")
      return { kind: "ai_settings", value: current };
    if (r.kind === "review_ai_discovery")
      return {
        kind: "executed",
        value: {
          result: { ai_job: "test" },
          ai_job_id: "test",
          ai_request_fingerprint: "fingerprint",
        },
      };
    if (r.kind === "ai_jobs")
      return {
        kind: "ai_jobs",
        value: [
          {
            id: "test",
            phase: failed ? "failed" : "complete",
            error: failed ? "Connection refused" : null,
            review: {
              purpose: "discover",
              endpoint: "http://10.10.10.72:11434/",
              route: "http://10.10.10.72:11434/api/tags",
              request_bytes: 0,
              fingerprint: "fingerprint",
              authentication: "none",
              unencrypted: true,
            },
            result: failed
              ? null
              : {
                  kind: "models",
                  value: [
                    { id: "qwen3.5:9b-q4_K_M" },
                    { id: "mara-lab:latest" },
                  ],
                },
          },
        ],
      };
    if (r.kind === "save_ai_settings") {
      await options.beforeSave?.();
      expect(r.fingerprint).toBe(current.fingerprint);
      expect(r.revision).toBe(current.settings.revision);
      if (r.change.kind !== "connection" && r.change.kind !== "single_setup")
        throw new Error("Unexpected change");
      const draft =
        r.change.kind === "single_setup" ? r.change.connection : r.change;
      const server = {
        id: draft.id ?? "connection",
        revision: "next",
        name: draft.name,
        provider: draft.provider,
        endpoint: draft.endpoint,
        timeout_ms: draft.timeout_ms,
        authentication: draft.authentication,
        endpoint_approval: "",
      };
      current = {
        ...current,
        fingerprint: current.fingerprint + "-next",
        settings: {
          ...current.settings,
          revision: current.settings.revision + "-next",
          connections: [server],
        },
      };
      if (r.change.kind === "single_setup")
        current.settings = {
          ...current.settings,
          profiles: [
            {
              id: "p",
              revision: "next",
              name: "AI",
              connection: server.id,
              model: r.change.model,
              system_message: r.change.system_message,
              parameters: r.change.parameters,
              template: "t",
              context_bytes: 16384,
            },
          ],
          assignments: [{ scope: { kind: "application" }, profile: "p" }],
        };
      return { kind: "ai_settings", value: structuredClone(current) };
    }
    throw new Error(r.kind);
  });
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: false }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  render(
    <SingleAiSetup
      bridge={bridge}
      onFlushReady={(flush) => {
        flushSave = flush;
      }}
    />,
  );
  return { request, user: userEvent.setup(), flush: () => flushSave() };
}
it("tests the entered server, selects its model, edits parameters and saves one configuration", async () => {
  const { request, user } = fixture();
  const address = await screen.findByLabelText("Server address");
  await user.clear(address);
  await user.type(address, "http://10.10.10.72");
  await user.click(
    screen.getByRole("button", { name: "Connect & Load Models" }),
  );
  await screen.findByText("Provider reachable · 2 models available.");
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "review_ai_discovery",
        connection: expect.objectContaining({
          endpoint: "http://10.10.10.72:11434/",
        }),
      }),
    ),
  );
  await user.click(screen.getByRole("tab", { name: "Model" }));
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Model" }),
    "qwen3.5:9b-q4_K_M",
  );
  if (
    screen.getByRole("tab", { name: "Model settings" }).hasAttribute("disabled")
  ) {
    await user.click(screen.getByRole("tab", { name: "Server" }));
    await user.click(
      screen.getByRole("button", { name: "Connect & Load Models" }),
    );
    await screen.findByText(/Provider reachable/);
  }
  await user.click(screen.getByRole("tab", { name: "Model settings" }));
  fireEvent.change(screen.getByLabelText("Temperature"), {
    target: { value: "0.3" },
  });
  fireEvent.change(screen.getByLabelText("Top P"), {
    target: { value: "0.8" },
  });
  fireEvent.change(screen.getByLabelText("System instructions"), {
    target: { value: "Explain observed changes" },
  });
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith({
      kind: "save_ai_settings",
      fingerprint: expect.any(String),
      revision: expect.any(String),
      change: {
        kind: "single_setup",
        connection: expect.objectContaining({
          endpoint: "http://10.10.10.72:11434/",
        }),
        model: "qwen3.5:9b-q4_K_M",
        system_message: "Explain observed changes",
        parameters: { temperature: 0.3, top_p: 0.8 },
      },
    }),
  );
  expect(screen.queryByText(/profile|assignment|template/i)).toBeNull();
});
it("invalidates discovered models after the server address changes", async () => {
  const { user } = fixture();
  await screen.findByLabelText("Server address");
  expect(screen.queryByRole("button", { name: "Save AI settings" })).toBeNull();
  await user.click(
    screen.getByRole("button", { name: "Connect & Load Models" }),
  );
  await screen.findByText(/Provider reachable/);
  expect(screen.queryByRole("button", { name: "Save AI settings" })).toBeNull();
  fireEvent.change(screen.getByLabelText("Port"), {
    target: { value: "11435" },
  });
  expect(screen.queryByRole("button", { name: "Save AI settings" })).toBeNull();
  fireEvent.change(screen.getByLabelText("Port"), {
    target: { value: "70000" },
  });
  expect(screen.getByRole("alert")).toHaveTextContent("Port must be");
});
it("reports a failed test and does not allow model selection", async () => {
  const { user, request } = fixture(true);
  await screen.findByLabelText("Server address");
  await user.click(
    screen.getByRole("button", { name: "Connect & Load Models" }),
  );
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Connection refused",
  );
  expect(screen.getByRole("tab", { name: "Model" })).toBeDisabled();
  expect(screen.getByRole("tab", { name: "Model settings" })).toBeDisabled();
  expect(screen.queryByRole("button", { name: "Save AI settings" })).toBeNull();
  expect(
    request.mock.calls.some(([r]) => r.kind === "review_ai_messages"),
  ).toBe(false);
});
it("loads existing settings with an editable server address and preserves model options", async () => {
  const saved: AiSettingsSnapshot = {
    ...initial,
    settings: {
      ...initial.settings,
      connections: [
        {
          id: "connection",
          revision: "r",
          name: "Old provider",
          provider: "ollama",
          endpoint: "http://10.10.10.72:11434/",
          timeout_ms: 30000,
          authentication: "none",
          endpoint_approval: "",
        },
      ],
      profiles: [
        {
          id: "p",
          revision: "r",
          name: "Old profile",
          connection: "connection",
          model: "mara-lab:latest",
          parameters: { temperature: 0.4 },
          system_message: "Saved instructions",
          template: "t",
          context_bytes: 16384,
        },
      ],
      assignments: [{ scope: { kind: "application" }, profile: "p" }],
    },
  };
  const { user, request } = fixture(false, saved);
  await screen.findByLabelText("Server address");
  if (
    screen.getByRole("tab", { name: "Model settings" }).hasAttribute("disabled")
  ) {
    await user.click(screen.getByRole("tab", { name: "Server" }));
    await user.click(
      screen.getByRole("button", { name: "Connect & Load Models" }),
    );
    await screen.findByText(/Provider reachable/);
  }
  await user.click(screen.getByRole("tab", { name: "Model settings" }));
  expect(screen.getByLabelText("Temperature")).toHaveValue(0.4);
  expect(screen.queryByRole("button", { name: "Save AI settings" })).toBeNull();
  expect(
    request.mock.calls.some(([r]) => r.kind === "review_ai_discovery"),
  ).toBe(true);
  await user.click(screen.getByRole("tab", { name: "Server" }));

  await waitFor(() =>
    expect(screen.getByLabelText("Server address")).toHaveValue(
      "http://10.10.10.72",
    ),
  );
  fireEvent.change(screen.getByLabelText("Server address"), {
    target: { value: "http://10.10.10.73" },
  });
  expect(screen.getByLabelText("Server address")).toHaveValue(
    "http://10.10.10.73",
  );
  await user.click(
    screen.getByRole("button", { name: "Connect & Load Models" }),
  );
  await screen.findByText(/Provider reachable/);
  await user.click(screen.getByRole("tab", { name: "Model" }));
  expect(screen.getByRole("combobox", { name: "Model" })).toHaveValue(
    "mara-lab:latest",
  );
  if (
    screen.getByRole("tab", { name: "Model settings" }).hasAttribute("disabled")
  ) {
    await user.click(screen.getByRole("tab", { name: "Server" }));
    await user.click(
      screen.getByRole("button", { name: "Connect & Load Models" }),
    );
    await screen.findByText(/Provider reachable/);
  }
  await user.click(screen.getByRole("tab", { name: "Model settings" }));
  expect(screen.getByLabelText("Temperature")).toHaveValue(0.4);
  expect(screen.getByLabelText("System instructions")).toHaveValue(
    "Saved instructions",
  );
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "save_ai_settings",
        change: expect.objectContaining({
          kind: "single_setup",
          connection: expect.objectContaining({
            id: "connection",
            endpoint: "http://10.10.10.73:11434/",
          }),
        }),
      }),
    ),
  );
});

it("unlocks tabs after connecting and preserves edits with keyboard navigation", async () => {
  const { user, request } = fixture();
  await screen.findByLabelText("Server address");
  if (
    screen.getByRole("tab", { name: "Model settings" }).hasAttribute("disabled")
  ) {
    await user.click(screen.getByRole("tab", { name: "Server" }));
    await user.click(
      screen.getByRole("button", { name: "Connect & Load Models" }),
    );
    await screen.findByText(/Provider reachable/);
  }
  await user.click(screen.getByRole("tab", { name: "Model settings" }));
  fireEvent.change(screen.getByLabelText("System instructions"), {
    target: { value: "Unsaved instructions" },
  });
  await user.click(screen.getByRole("tab", { name: "Model" }));
  expect(screen.getByRole("combobox", { name: "Model" })).toBeEnabled();
  await user.keyboard("{ArrowRight}");
  expect(screen.getByRole("tab", { name: "Model settings" })).toHaveFocus();
  expect(screen.getByLabelText("System instructions")).toHaveValue(
    "Unsaved instructions",
  );
  await user.keyboard("{Home}");
  expect(screen.getByRole("tab", { name: "Server" })).toHaveFocus();
  expect(screen.queryByRole("button", { name: /^(Next|Back)$/ })).toBeNull();
  expect(
    request.mock.calls.some(([r]) => r.kind === "review_ai_discovery"),
  ).toBe(true);
});
it("starts parameters blank, clears overrides, and resets them when choosing another model", async () => {
  const { user, request } = fixture();
  await screen.findByLabelText("Server address");
  if (
    screen.getByRole("tab", { name: "Model settings" }).hasAttribute("disabled")
  ) {
    await user.click(screen.getByRole("tab", { name: "Server" }));
    await user.click(
      screen.getByRole("button", { name: "Connect & Load Models" }),
    );
    await screen.findByText(/Provider reachable/);
  }
  await user.click(screen.getByRole("tab", { name: "Model settings" }));
  for (const label of [
    "Temperature",
    "Top P",
    "Top K",
    "Min P",
    "Context size",
    "Maximum output tokens",
    "Seed",
    "Repetition penalty",
  ])
    expect(screen.getByLabelText(label)).toHaveValue(null);
  fireEvent.change(screen.getByLabelText("Temperature"), {
    target: { value: "0" },
  });
  expect(screen.getByLabelText("Temperature")).toHaveValue(0);
  await user.click(screen.getByRole("button", { name: "Use model defaults" }));
  expect(screen.getByLabelText("Temperature")).toHaveValue(null);
  fireEvent.change(screen.getByLabelText("Temperature"), {
    target: { value: "0" },
  });
  await user.click(screen.getByRole("tab", { name: "Server" }));
  await user.click(
    screen.getByRole("button", { name: "Connect & Load Models" }),
  );
  await screen.findByText(/Provider reachable/);
  await user.click(screen.getByRole("tab", { name: "Model" }));
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Model" }),
    "mara-lab:latest",
  );
  if (
    screen.getByRole("tab", { name: "Model settings" }).hasAttribute("disabled")
  ) {
    await user.click(screen.getByRole("tab", { name: "Server" }));
    await user.click(
      screen.getByRole("button", { name: "Connect & Load Models" }),
    );
    await screen.findByText(/Provider reachable/);
  }
  await user.click(screen.getByRole("tab", { name: "Model settings" }));
  expect(screen.getByLabelText("Temperature")).toHaveValue(null);
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "save_ai_settings",
        change: expect.objectContaining({ parameters: {} }),
      }),
    ),
  );
});
it("unsets one parameter without changing the others and shows model default ghost text", async () => {
  const { user, request } = fixture();
  await screen.findByLabelText("Server address");
  await user.click(
    screen.getByRole("button", { name: "Connect & Load Models" }),
  );
  await screen.findByText(/Provider reachable/);
  await user.click(screen.getByRole("tab", { name: "Model" }));
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Model" }),
    "mara-lab:latest",
  );
  if (
    screen.getByRole("tab", { name: "Model settings" }).hasAttribute("disabled")
  ) {
    await user.click(screen.getByRole("tab", { name: "Server" }));
    await user.click(
      screen.getByRole("button", { name: "Connect & Load Models" }),
    );
    await screen.findByText(/Provider reachable/);
  }
  await user.click(screen.getByRole("tab", { name: "Model settings" }));
  fireEvent.change(screen.getByLabelText("Temperature"), {
    target: { value: "0" },
  });
  fireEvent.change(screen.getByLabelText("Top P"), {
    target: { value: "0.8" },
  });
  await user.click(screen.getByRole("button", { name: "Unset Temperature" }));
  expect(screen.getByLabelText("Temperature")).toHaveValue(null);
  expect(screen.getByLabelText("Temperature")).toHaveAttribute(
    "placeholder",
    "Model default",
  );
  expect(screen.getByLabelText("Top P")).toHaveValue(0.8);
  expect(
    screen.getByRole("button", { name: "Unset Temperature" }),
  ).toBeDisabled();
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "save_ai_settings",
        change: expect.objectContaining({
          parameters: { temperature: null, top_p: 0.8 },
        }),
      }),
    ),
  );
});

it("serializes automatic saves and flushes the latest edit before closing", async () => {
  let release = () => {};
  const gate = new Promise<void>((resolve) => {
    release = resolve;
  });
  let first = true;
  const { request, flush } = fixture(false, initial, {
    beforeSave: async () => {
      if (first) {
        first = false;
        await gate;
      }
    },
  });
  await screen.findByLabelText("Port");
  fireEvent.change(screen.getByLabelText("Port"), {
    target: { value: "11435" },
  });
  await waitFor(() =>
    expect(
      request.mock.calls.filter(([r]) => r.kind === "save_ai_settings"),
    ).toHaveLength(1),
  );
  fireEvent.change(screen.getByLabelText("Port"), {
    target: { value: "11436" },
  });
  fireEvent.change(screen.getByLabelText("Port"), {
    target: { value: "11437" },
  });
  const completed = flush();
  release();
  await completed;
  const writes = request.mock.calls
    .map(([r]) => r)
    .filter((r) => r.kind === "save_ai_settings");
  expect(writes).toHaveLength(2);
  expect(writes[1]).toMatchObject({
    fingerprint: "before-next",
    revision: "revision-next",
    change: {
      kind: "connection",
      id: "connection",
      endpoint: "http://127.0.0.1:11437/",
    },
  });
});
it("reports an automatic save failure and refuses a successful close flush", async () => {
  const { flush } = fixture(false, initial, {
    beforeSave: async () => {
      throw new Error("Disk full");
    },
  });
  await screen.findByLabelText("Port");
  fireEvent.change(screen.getByLabelText("Port"), {
    target: { value: "11435" },
  });
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Could not save AI settings: Error: Disk full",
  );
  expect(await flush()).toBe(false);
  expect(screen.queryByText("Saved automatically")).toBeNull();
});

it("offers context presets and useful token increments without setting defaults", async () => {
  const { user } = fixture();
  await screen.findByLabelText("Server address");
  if (
    screen.getByRole("tab", { name: "Model settings" }).hasAttribute("disabled")
  ) {
    await user.click(screen.getByRole("tab", { name: "Server" }));
    await user.click(
      screen.getByRole("button", { name: "Connect & Load Models" }),
    );
    await screen.findByText(/Provider reachable/);
  }
  await user.click(screen.getByRole("tab", { name: "Model settings" }));
  const context = screen.getByLabelText("Context size");
  const output = screen.getByLabelText("Maximum output tokens");
  expect(context).toHaveAttribute("step", "1024");
  expect(output).toHaveAttribute("step", "256");
  expect(context).toHaveValue(null);
  expect(output).toHaveValue(null);
  const presets = screen.getByRole("combobox", {
    name: "Context size presets",
  });
  await user.selectOptions(presets, "262144");
  expect(context).toHaveValue(262144);
  expect(context).toBeValid();
  fireEvent.change(context, { target: { value: "9216" } });
  expect(presets).toHaveValue("9216");
  expect(context).toBeValid();
  fireEvent.change(output, { target: { value: "768" } });
  expect(output).toBeValid();
  await user.selectOptions(presets, "");
  expect(context).toHaveValue(null);
  await user.click(
    screen.getByRole("button", { name: "Unset Maximum output tokens" }),
  );
  expect(output).toHaveValue(null);
});

it("allows integer seeds and randomizes across the full seed range", async () => {
  const { user } = fixture();
  await screen.findByLabelText("Server address");
  if (
    screen.getByRole("tab", { name: "Model settings" }).hasAttribute("disabled")
  ) {
    await user.click(screen.getByRole("tab", { name: "Server" }));
    await user.click(
      screen.getByRole("button", { name: "Connect & Load Models" }),
    );
    await screen.findByText(/Provider reachable/);
  }
  await user.click(screen.getByRole("tab", { name: "Model settings" }));
  const seed = screen.getByLabelText("Seed");
  expect(seed).toHaveValue(null);
  expect(seed).toHaveAttribute("min", "0");
  expect(seed).toHaveAttribute("max", "2147483647");
  expect(seed).toHaveAttribute("step", "1");
  fireEvent.change(seed, { target: { value: "12345" } });
  expect(seed).toHaveValue(12345);
  const random = vi.spyOn(crypto, "getRandomValues");
  try {
    random.mockReturnValueOnce(new Uint32Array([0]));
    await user.click(screen.getByRole("button", { name: "Randomize Seed" }));
    expect(seed).toHaveValue(0);
    random.mockReturnValueOnce(new Uint32Array([4294967295]));
    await user.click(screen.getByRole("button", { name: "Randomize Seed" }));
    expect(seed).toHaveValue(2147483647);
    expect(seed).toBeValid();
    await user.click(screen.getByRole("button", { name: "Unset Seed" }));
    expect(seed).toHaveValue(null);
  } finally {
    random.mockRestore();
  }
});

it("keeps model tabs locked until a live connection succeeds and relocks on edits", async () => {
  const { user } = fixture();
  await screen.findByLabelText("Server address");
  const modelTab = screen.getByRole("tab", { name: "Model" });
  const settingsTab = screen.getByRole("tab", { name: "Model settings" });
  expect(modelTab).toBeDisabled();
  expect(settingsTab).toBeDisabled();
  screen.getByRole("tab", { name: "Server" }).focus();
  await user.keyboard("{End}");
  expect(screen.getByRole("tab", { name: "Server" })).toHaveFocus();
  expect(screen.getByText(/A successful connection unlocks/)).toBeVisible();
  await user.click(
    screen.getByRole("button", { name: "Connect & Load Models" }),
  );
  await screen.findByText(/Provider reachable/);
  expect(modelTab).toBeEnabled();
  expect(settingsTab).toBeEnabled();
  expect(screen.getByRole("tab", { name: "Server" })).toHaveAttribute(
    "aria-selected",
    "true",
  );
  fireEvent.change(screen.getByLabelText("Port"), {
    target: { value: "11435" },
  });
  expect(modelTab).toBeDisabled();
  expect(settingsTab).toBeDisabled();
  fireEvent.change(screen.getByLabelText("Port"), {
    target: { value: "11434" },
  });
  expect(modelTab).toBeDisabled();
});

it("defaults Thinking off and automatically saves toggling it on and off", async () => {
  const { user, request } = fixture();
  await screen.findByLabelText("Server address");
  await user.click(
    screen.getByRole("button", { name: "Connect & Load Models" }),
  );
  await screen.findByText(/Provider reachable/);
  await user.click(screen.getByRole("tab", { name: "Model" }));
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Model" }),
    "mara-lab:latest",
  );
  await user.click(screen.getByRole("tab", { name: "Model settings" }));
  const thinking = screen.getByRole("checkbox", { name: "Thinking" });
  expect(thinking).not.toBeChecked();
  await user.click(thinking);
  expect(thinking).toBeChecked();
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "save_ai_settings",
        change: expect.objectContaining({
          kind: "single_setup",
          parameters: expect.objectContaining({ thinking: true }),
        }),
      }),
    ),
  );
  await user.click(thinking);
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "save_ai_settings",
        change: expect.objectContaining({
          kind: "single_setup",
          parameters: expect.objectContaining({ thinking: false }),
        }),
      }),
    ),
  );
});

it("treats the empty model option as a placeholder after selecting and saving a model", async () => {
  const { request, user, flush } = fixture();
  await screen.findByLabelText("Server address");
  fireEvent.change(screen.getByLabelText("Server address"), {
    target: { value: "http://127.0.0.2" },
  });
  await waitFor(() =>
    expect(
      request.mock.calls.some(([r]) => r.kind === "save_ai_settings"),
    ).toBe(true),
  );
  await act(async () => {
    expect(await flush()).toBe(true);
  });
  const connectionSave = request.mock.calls.find(
    ([r]) => r.kind === "save_ai_settings",
  );
  expect(connectionSave?.[0]).toMatchObject({
    kind: "save_ai_settings",
    change: { kind: "connection" },
  });
  await user.click(
    screen.getByRole("button", { name: "Connect & Load Models" }),
  );
  await screen.findByText(/Provider reachable/);
  await user.click(screen.getByRole("tab", { name: "Model" }));
  const model = screen.getByRole("combobox", { name: "Model" });
  await user.selectOptions(model, "qwen3.5:9b-q4_K_M");
  await act(async () => {
    expect(await flush()).toBe(true);
  });
  await user.selectOptions(model, "");
  await act(async () => {
    expect(await flush()).toBe(true);
  });
  const index =
    request.mock.calls.length -
    1 -
    [...request.mock.calls]
      .reverse()
      .findIndex(([r]) => r.kind === "save_ai_settings");
  const saved = await request.mock.results[index].value;
  if (saved.kind !== "ai_settings") throw new Error("Expected saved settings");
  expect(saved.value.settings.profiles[0].model).toBe("qwen3.5:9b-q4_K_M");
  expect(model).toHaveValue("qwen3.5:9b-q4_K_M");
  expect(
    screen.getByRole("option", { name: "Choose an installed model" }),
  ).toBeDisabled();
  cleanup();
  const reopened = fixture(false, saved.value);
  await screen.findByLabelText("Server address");
  await reopened.user.click(
    screen.getByRole("button", { name: "Connect & Load Models" }),
  );
  await screen.findByText(/Provider reachable/);
  await reopened.user.click(screen.getByRole("tab", { name: "Model" }));
  expect(screen.getByRole("combobox", { name: "Model" })).toHaveValue(
    "qwen3.5:9b-q4_K_M",
  );
});
