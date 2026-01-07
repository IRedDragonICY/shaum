/**
 * TypeScript type definitions for Shaum
 * 
 * These types are auto-generated from Rust via tsify-next
 * and will be regenerated on each build.
 */

/** Fasting status according to Islamic jurisprudence. */
export enum FastingStatus {
    /** Mubah - Permissible, no special ruling */
    Mubah = "mubah",
    /** Makruh - Disliked, better to avoid */
    Makruh = "makruh",
    /** Sunnah - Recommended, rewarded but not obligatory */
    Sunnah = "sunnah",
    /** Sunnah Muakkadah - Highly recommended */
    SunnahMuakkadah = "sunnahMuakkadah",
    /** Wajib - Obligatory (e.g., Ramadan) */
    Wajib = "wajib",
    /** Haram - Forbidden (e.g., Eid days) */
    Haram = "haram",
}

/** Analysis result for a specific date's fasting status. */
export interface WasmFastingAnalysis {
    /** Primary fasting status for this date. */
    primaryStatus: string;
    /** Hijri year (e.g., 1447). */
    hijriYear: number;
    /** Hijri month (1-12). */
    hijriMonth: number;
    /** Hijri day (1-30). */
    hijriDay: number;
    /** List of fasting type reasons (e.g., ["Ramadhan", "Monday"]). */
    reasons: string[];
    /** Human-readable explanation of the fasting ruling. */
    explanation: string;
}

/** 
 * Fasting type/reason for a particular ruling.
 * Examples: "Ramadhan", "Arafah", "Ashura", "Monday", "Thursday", "EidAlFitr"
 */
export type FastingType = string;
