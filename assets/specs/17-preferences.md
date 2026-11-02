# 17 — Theme and Language

## Requirements

**PREF-1 — Settings menu.** The Settings menu (`SHELL-5`) is always enabled, with or without a
document. It lists the themes under the heading "`Theme`" and the languages under the heading
"`Language`". The current theme and the current language are shown selected. Choosing a row takes
effect immediately, for the whole application, and closes the menu.

**PREF-2 — Themes.** The themes, in this order:

| Theme | Text | Meaning |
|---|---|---|
| System | "`💻 System`" | Follow the light or dark setting of the operating system. |
| Light | "`☀ Light`" | Always light. |
| Dark | "`🌙 Dark`" | Always dark. |

The default is System.

**PREF-3 — Languages.** The languages, in this order:

| Language | Identifier | Text |
|---|---|---|
| English (United States) | `en-US` | "`English (United States)`" |
| Spanish (Spain) | `es-ES` | "`Español (España)`" |

Each language is named in that language, whichever language is in use, so that it can be recognised
by someone who cannot read the current one. The default is `en-US`. The operating system's language
is not consulted.

**PREF-4 — Kept between runs.** The chosen theme and language are saved with the panel layout
(`WORK`) and restored at the next start, before the first frame is shown. A saved value that is
missing or unknown is replaced by the default. They are preferences of the application: they are not
stored in documents, and choosing one is not an action (`HIST`), so it does not make a document
unsaved and cannot be undone.

**PREF-5 — Translation files.** Everything the application says is held in one translation file per
language, as named messages, each of which may have named values filled in when it is shown. The
files are part of the program, so an installed application needs no other files, and it works from
any working directory.

**PREF-6 — What is translated.** These are shown in the current language:

- labels, tooltips, table headers, placeholder and help text;
- window title text, dialogs and native file dialogs' titles and filters;
- the crash dialog (`SHELL-1c`);
- history descriptions (`HIST`), which change along with the language, including those already listed;
- the reasons given for errors, and for a scale or grid not being available.

These stay as they are in every language:

- the application's name, file names, the author and the repository link;
- the About dialog's description, which is the README's About section (`SHELL-13`);
- unit symbols, the line and dimension letters, column headings `#`, `x1`, `y1`, `x2`, `y2`, and `px`;
- how numbers are written and read (`FMT`): a decimal point is always "`.`";
- the contents of document files (`FILE`), which do not depend on the language they were saved in.

**PREF-7 — Complete translations.** Every language has the same messages as `en-US`, each with the
same values. A message missing from a language would be shown in `en-US`, but the automated tests
MUST catch it before release.

## Acceptance criteria

- Theme:
  - The Settings menu offers each theme, and choosing one reports that theme.
  - Choosing a theme puts it into use and it is what the next tool bar shows.
- Language:
  - The Settings menu offers each language, and choosing one reports that language.
  - After choosing `es-ES`, a history description and an error are in Spanish, and each language is
    still named in itself. After choosing `en-US`, they are in English again.
- Between runs:
  - With nothing saved, the theme is System and the language is `en-US`.
  - A saved theme and language are loaded again.
  - An unknown saved theme or language gives the default.
- Translations:
  - Every language has the same messages, with the same values, as `en-US`.
  - Every message the application uses exists, and every message in the files is used.
  - Values are filled in with no extra characters around them.
