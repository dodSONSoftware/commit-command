import { useState } from "react";

/** Keep the separator verbatim; viewing a draft must never rewrite its bytes. */
export function splitMessage(message: string) {
  const separator = /\r?\n[ \t]*\r?\n/.exec(message);
  if (!separator) return { subject: message, separator: "", body: "" };
  return {
    subject: message.slice(0, separator.index),
    separator: separator[0],
    body: message.slice(separator.index + separator[0].length),
  };
}
export function replaceSubject(message: string, subject: string): string {
  const parts = splitMessage(message);
  return subject + parts.separator + parts.body;
}
export function replaceBody(message: string, body: string): string {
  const parts = splitMessage(message);
  return parts.subject + (parts.separator || (body ? "\n\n" : "")) + body;
}

const MAX_MESSAGE = 512 * 1024;
export function CommitMessageEditor({
  label,
  value,
  onChange,
  rows = 4,
  disabled = false,
  placeholder,
}: {
  label: string;
  value: string;
  onChange: (message: string) => void;
  rows?: number;
  disabled?: boolean;
  placeholder?: string;
}) {
  const [structured, setStructured] = useState(false);
  const parts = splitMessage(value);
  const bodySeparator = parts.separator || "\n\n";
  return (
    <fieldset className="commit-message-editor" disabled={disabled}>
      <legend>{label}</legend>
      <button type="button" onClick={() => setStructured(!structured)}>
        {structured ? "Edit raw message" : "Edit subject and body"}
      </button>
      {structured ? (
        <>
          <label>
            Subject
            <textarea
              aria-label={`${label} subject`}
              rows={2}
              value={parts.subject}
              maxLength={Math.max(
                0,
                MAX_MESSAGE - parts.separator.length - parts.body.length,
              )}
              onChange={(event) =>
                onChange(replaceSubject(value, event.target.value))
              }
            />
          </label>
          <label>
            Body (optional)
            <textarea
              aria-label={`${label} body`}
              rows={Math.max(3, rows - 2)}
              value={parts.body}
              maxLength={Math.max(
                0,
                MAX_MESSAGE - parts.subject.length - bodySeparator.length,
              )}
              onChange={(event) =>
                onChange(replaceBody(value, event.target.value))
              }
            />
          </label>
        </>
      ) : (
        <label>
          {label}
          <textarea
            rows={rows}
            value={value}
            maxLength={MAX_MESSAGE}
            placeholder={placeholder}
            onChange={(event) => onChange(event.target.value)}
          />
        </label>
      )}
    </fieldset>
  );
}
