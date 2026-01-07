/**
 * # Shaum
 *
 * A Fiqh-compliant Islamic fasting rules engine for the web.
 * Determines Wajib, Sunnah, Makruh, and Haram fasting days.
 * Powered by Rust and WebAssembly.
 *
 * ## Features
 * - **Fiqh Compliant**: Accurate Islamic jurisprudence for fasting status.
 * - **Type Safe**: Full TypeScript definitions for all types.
 * - **Fast**: Core logic runs in WebAssembly.
 *
 * ## Usage
 *
 * ```typescript
 * import { Shaum, analyze, type WasmFastingAnalysis } from "@islam/shaum";
 *
 * // Function-based API
 * const result = analyze("2026-03-01");
 * console.log(result.primaryStatus); // "Wajib" during Ramadan
 *
 * // Class-based API
 * const shaum = new Shaum("2026-03-01");
 * const analysis = shaum.analyze();
 * console.log(shaum.explain());
 * ```
 *
 * @module
 */

// Re-export WASM bindings
// @deno-types="./shaum.d.ts"
export { Shaum, analyze } from "./shaum.js";

// Export TypeScript types
export * from "./types.ts";
