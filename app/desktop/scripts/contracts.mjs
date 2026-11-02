import { execFileSync } from "node:child_process";
import { compile } from "json-schema-to-typescript";
import { writeFileSync, readFileSync } from "node:fs";
const schemas = JSON.parse(
  execFileSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "../core/Cargo.toml",
      "--features",
      "application-services",
      "--bin",
      "gitfleet-contracts",
    ],
    { encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
  ),
);
function normalize(value) {
  if (Array.isArray(value)) return value.map(normalize);
  if (value && typeof value === "object") {
    const o = {};
    for (const [k, v] of Object.entries(value)) {
      o[k === "$defs" ? "definitions" : k] =
        k === "$ref" && typeof v === "string"
          ? v.replace("#/$defs/", "#/definitions/")
          : normalize(v);
    }
    return o;
  }
  return value;
}
for (const [name, schema] of Object.entries(schemas)) {
  const text = await compile(
    normalize(schema),
    name === "request" ? "Request" : "Response",
    {
      bannerComment:
        "/* Generated from Rust Core JSON Schema. Run npm run contracts. */",
      unreachableDefinitions: true,
    },
  );
  const path = `src/services/${name}.generated.ts`;
  if (process.argv.includes("--check")) {
    if (readFileSync(path, "utf8") !== text)
      throw new Error(`${path} is stale`);
  } else writeFileSync(path, text);
}
