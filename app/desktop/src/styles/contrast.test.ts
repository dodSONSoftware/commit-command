import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
const app = readFileSync("src/styles/app.css", "utf8");
const review = readFileSync("src/styles/review.css", "utf8");
function declarations(css: string, selector: string) {
  const start = css.indexOf(`${selector} {`);
  if (start < 0) throw new Error(`Missing CSS selector ${selector}`);
  return Object.fromEntries(
    css
      .slice(start + selector.length + 2, css.indexOf("}", start))
      .split(";")
      .flatMap((line) => {
        const colon = line.indexOf(":");
        return colon < 0
          ? []
          : [[line.slice(0, colon).trim(), line.slice(colon + 1).trim()]];
      }),
  );
}
function rgb(hex: string) {
  const full =
    hex.length === 4 ? `#${[...hex.slice(1)].map((c) => c + c).join("")}` : hex;
  if (!/^#[\da-f]{6}$/i.test(full))
    throw new Error(`Expected hex color: ${hex}`);
  return [1, 3, 5].map((i) => parseInt(full.slice(i, i + 2), 16) / 255);
}
function luminance(color: number[]) {
  return color
    .map((c) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4))
    .reduce((sum, c, i) => sum + c * [0.2126, 0.7152, 0.0722][i], 0);
}
function contrast(a: number[], b: number[]) {
  const [low, high] = [luminance(a), luminance(b)].sort((x, y) => x - y);
  return (high + 0.05) / (low + 0.05);
}
function mix(a: number[], fraction: number, b: number[]) {
  return a.map((c, i) => c * fraction + b[i] * (1 - fraction));
}
function minimum(label: string, a: number[], b: number[], required: number) {
  expect(contrast(a, b), label).toBeGreaterThanOrEqual(required);
}
it.each([":root", ':root[data-theme="light"]', ':root[data-theme="system"]'])(
  "keeps theme text, graph colors, input boundaries and focus visible for %s",
  (selector) => {
    const palette = declarations(app, selector);
    const color = (name: string) => rgb(palette[`--${name}`]);
    for (const surface of ["bg", "panel", "panel2"]) {
      for (const text of ["text", "muted"])
        minimum(`${text} on ${surface}`, color(text), color(surface), 4.5);
      minimum(
        `input boundary against ${surface}`,
        color("control-line"),
        color(surface),
        3,
      );
      minimum(
        `focus ring against ${surface}`,
        color("blue"),
        color(surface),
        3,
      );
      for (const lane of ["blue", "green", "yellow", "purple"])
        minimum(`graph ${lane} on ${surface}`, color(lane), color(surface), 3);
    }
    for (const [tone, fractions] of Object.entries({
      blue: [0.07, 0.12, 0.14, 0.15],
      green: [0.07],
      yellow: [0.07, 0.08],
      red: [0.08],
    })) {
      for (const fraction of fractions)
        minimum(
          `${tone} text on tinted panel`,
          color(tone),
          mix(color(tone), fraction, color("panel")),
          4.5,
        );
    }
    expect(app).toContain("border: 1px solid var(--control-line)");
    expect(declarations(app, "textarea::placeholder")).toEqual({
      color: "var(--muted)",
      opacity: "1",
    });
  },
);
it("keeps primary labels readable including Light-mode hover and the authoritative review", () => {
  minimum(
    "main primary",
    rgb("#ffffff"),
    rgb(declarations(app, ".primary").background),
    4.5,
  );
  minimum(
    "main primary hover",
    rgb("#ffffff"),
    rgb(declarations(app, ".primary:hover:not(:disabled)").background),
    4.5,
  );
  minimum(
    "review primary",
    rgb(declarations(review, "button").color),
    rgb(declarations(review, ".primary").background),
    4.5,
  );
  minimum(
    "review body",
    rgb(declarations(review, ":root").color),
    rgb(declarations(review, ":root").background),
    4.5,
  );
  minimum(
    "review secondary text",
    rgb(declarations(review, "p").color),
    rgb(declarations(review, ":root").background),
    4.5,
  );
});
