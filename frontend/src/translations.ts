export type Language =
  | { language: null }
  | { language: string; script: null }
  | { language: string; script: string };

type MatchLevel = 1 | 2 | 3 | 0;

let LANGUAGE = "";

function matches(spec: Language, language: Language): MatchLevel {
  if (spec.language == null) return 1;
  if (spec.script == null) {
    return spec.language == language.language ? 2 : 0;
  }
  return "script" in language &&
    language.script !== null &&
    spec.language == language.language &&
    spec.script == language.script
    ? 3
    : 0;
}

export function _parse(specifier: string): Language {
  const [language, script] = specifier.split(/[_-]/);
  if (!language) return { language: null };
  return { language, script: script?.toUpperCase() ?? null };
}

export function _findBestLanguage(
  available: Language[],
  wantedByUser: Language[]
): Language {
  let best: [MatchLevel, Language] = [0, available[0] ?? { language: null }];
  for (const spec of wantedByUser) {
    for (const language of available) {
      const match = matches(spec, language);
      if (match > best[0]) {
        best = [match, language];
      }
    }

    if (best[0] > 0) {
      return best[1];
    }
  }

  return best[1];
}

export function setup(known: string[]) {
  if (known.length === 0) throw new Error("must use at least one language");

  const knownParsed = known.map(_parse);
  const language = _findBestLanguage(
    knownParsed,
    navigator.languages.map(_parse)
  );
  const index = knownParsed.indexOf(language);
  LANGUAGE = known[index];
}

export function t(args: Record<string, string>): string {
  if (!LANGUAGE) throw new Error("No language selected");
  if (!(LANGUAGE in args))
    throw new Error(`${LANGUAGE} is not present in args`);
  return args[LANGUAGE];
}
