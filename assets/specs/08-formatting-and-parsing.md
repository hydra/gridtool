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

**FMT-6 — Editable coordinates.** A single coordinate in an edit field is formatted with
up to 2 decimals (`FMT-1`): 12.34567 → "`12.35`", 40 → "`40`".

**FMT-7 — Measured lengths.** A measured real-world length is formatted with up to 3
decimals (`FMT-1`): 12.34567 → "`12.346`". Measured real-world areas (`AREA-7`) are
formatted the same way.

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

**FMT-8 — Plain number input.** Parsing a plain number:
1. Remove leading and trailing whitespace.
2. Parse a decimal number, with an optional sign and fractional part, using `.` as the
   decimal separator.

The result is valid only if it is finite. Examples: "`-12.5`" → −12.5, "`0`" → 0. Empty
text, "`abc`", "`inf`", "`NaN`" and "`1,5`" are invalid.

## Input fields

**FMT-9 — Commit-on-blur field.** Several values are edited in single-line text fields
that commit **once**, when editing ends, rather than on every keystroke. Editing ends
when the field loses keyboard focus, which includes pressing Enter.

- While the field is not focused, it shows the current value, formatted.
- When it gains focus, the text shown at that moment is remembered as the original, and
  the user's text is kept for as long as the field stays focused.
- When focus is lost:
  - if the text differs from the original, it is handed to the owner to parse and, if
    valid, to commit as one action
  - if the text is unchanged, nothing happens
- Afterwards, the field shows the current value again. So invalid input visibly reverts
  to the current value.
- Each field has its own state. Moving focus directly from one field to another commits
  the first field correctly.

These are commit-on-blur fields: the scale distances A and B (`SCALE-14`) and the
measurement coordinates (`MEAS-12`). The view rotation field is **not**, because it
applies valid input live (`VIEW-16`).
