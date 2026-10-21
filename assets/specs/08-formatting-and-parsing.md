# 08 — Formatting and Parsing

## Formatting

**FMT-1 — Trimmed decimals.** "Format with up to *n* decimals" means:
1. Round to *n* decimal places.
2. If there is a decimal point, remove trailing zeros, and then remove the decimal point
   if nothing follows it.
3. Show negative zero ("`-0`") as "`0`".

Examples with *n* = 2: 45 → "`45`", 12.5 → "`12.5`", 1/3 → "`0.33`", −0.001 → "`0`".
Zeros in whole numbers are kept: 100 → "`100`".

**FMT-2 — Angles.** Angles are formatted with up to 2 decimals (`FMT-1`): 12.345 →
"`12.35`".

**FMT-5 — Image coordinates.** Image coordinates are formatted with exactly 1 decimal
place, as "`(<x>, <y>)`": (12, 3.456) → "`(12.0, 3.5)`".

## Parsing

**FMT-3 — Angle input.** Parsing an angle:
1. Remove leading and trailing whitespace.
2. Remove any trailing degree signs (`°`), then any whitespace before them.
3. Parse the rest as a decimal number. It may have a sign and a fractional part.

The result is valid only if it is finite. Examples: "`45`" → 45, "`  -12.5 `" → −12.5,
"`90°`" → 90, "`90 °`" → 90. Empty text, "`abc`", "`inf`" and "`NaN`" are invalid.

**FMT-4 — Distance input.** Parsing a distance:
1. Remove leading and trailing whitespace.
2. If nothing is left, the result is **clear** (no distance).
3. Otherwise, parse a decimal number. It is valid only if it is finite and greater than
   0.

Anything else is invalid.
