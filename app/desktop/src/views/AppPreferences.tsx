import type { AppPreferences as StoredPreferences } from "../services/response.generated";
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
  type TextareaHTMLAttributes,
} from "react";
import { value, type Bridge } from "../services/bridge";
import type {
  PreferenceChange,
  TextareaField,
} from "../services/request.generated";

type AppPreferences = Required<StoredPreferences>;
const defaults: AppPreferences = {
  textarea_heights: {},
  generation_timeout_seconds: 60,
};
type PreferencesContext = {
  preferences: AppPreferences;
  update: (change: PreferenceChange) => void;
  register: (flush: () => void) => () => void;
  flush: () => Promise<void>;
};
const Context = createContext<PreferencesContext>({
  preferences: defaults,
  update: () => {},
  register: () => () => {},
  flush: async () => {},
});
export const useAppPreferences = () => useContext(Context);

export function AppPreferencesProvider({
  bridge,
  children,
}: {
  bridge: Bridge;
  children: ReactNode;
}) {
  const [preferences, setPreferences] = useState(defaults);
  const [error, setError] = useState("");
  const queue = useRef(Promise.resolve());
  const revision = useRef(0);
  const callbacks = useRef(new Set<() => void>());
  useEffect(() => {
    let live = true;
    void bridge
      .request({ kind: "app_preferences" })
      .then((data) => {
        const saved = value(data, "app_preferences");
        if (live && revision.current === 0)
          setPreferences({ ...defaults, ...saved });
      })
      .catch((e) => {
        if (live)
          setError(`Layout preferences could not be loaded: ${String(e)}`);
      });
    return () => {
      live = false;
    };
  }, [bridge]);
  const update = useCallback(
    (change: PreferenceChange) => {
      revision.current++;
      setPreferences((current) =>
        change.kind === "textarea_height"
          ? {
              ...current,
              textarea_heights: {
                ...current.textarea_heights,
                [change.field]: change.height,
              },
            }
          : { ...current, generation_timeout_seconds: change.seconds },
      );
      queue.current = queue.current
        .catch(() => {})
        .then(async () => {
          await bridge.request({ kind: "save_app_preference", change });
          setError("");
        });
      void queue.current.catch((e) =>
        setError(`Preferences were not saved: ${String(e)}`),
      );
    },
    [bridge],
  );
  const register = useCallback((flush: () => void) => {
    callbacks.current.add(flush);
    return () => {
      callbacks.current.delete(flush);
    };
  }, []);
  const flush = useCallback(async () => {
    callbacks.current.forEach((save) => save());
    await queue.current;
  }, []);
  useEffect(() => bridge.registerPreferencesFlush?.(flush), [bridge, flush]);
  return (
    <Context.Provider value={{ preferences, update, register, flush }}>
      {error && (
        <div role="alert" className="error">
          {error}
        </div>
      )}
      {children}
    </Context.Provider>
  );
}

export function RememberedTextarea({
  field,
  style,
  ...props
}: TextareaHTMLAttributes<HTMLTextAreaElement> & { field: TextareaField }) {
  const { preferences, update, register } = useAppPreferences();
  const ref = useRef<HTMLTextAreaElement>(null);
  const saved = preferences.textarea_heights[field];
  useEffect(() => {
    const element = ref.current;
    if (!element) return;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const save = () => {
      clearTimeout(timer);
      const height = Math.round(parseFloat(element.style.height));
      if (
        Number.isFinite(height) &&
        height >= 48 &&
        height <= 4096 &&
        height !== saved
      )
        update({ kind: "textarea_height", field, height });
    };
    const observer =
      typeof ResizeObserver === "undefined"
        ? null
        : new ResizeObserver(() => {
            clearTimeout(timer);
            timer = setTimeout(save, 250);
          });
    observer?.observe(element);
    window.addEventListener("pointerup", save, true);
    const unregister = register(save);
    return () => {
      clearTimeout(timer);
      observer?.disconnect();
      window.removeEventListener("pointerup", save, true);
      unregister();
    };
  }, [field, saved, update, register]);
  return (
    <textarea
      {...props}
      ref={ref}
      style={{
        ...style,
        height: saved,
        minHeight: 48,
        maxHeight: 4096,
        resize: "vertical",
      }}
    />
  );
}

export function GenerationTimeoutSetting() {
  const { preferences, update } = useAppPreferences();
  const [draft, setDraft] = useState<string | null>(null);
  return (
    <label>
      Message generation timeout (seconds)
      <input
        type="number"
        min={1}
        max={3600}
        step={1}
        value={draft ?? String(preferences.generation_timeout_seconds)}
        onChange={(event) => {
          setDraft(event.target.value);
          const seconds = event.target.valueAsNumber;
          if (Number.isInteger(seconds) && seconds >= 1 && seconds <= 3600)
            update({ kind: "generation_timeout", seconds });
        }}
        onBlur={() => setDraft(null)}
      />
      <small>Default: 60 seconds. Model loading has its own timeout.</small>
    </label>
  );
}
