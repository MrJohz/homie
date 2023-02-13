export type Language =
  | { language: null }
  | { language: string; script: null }
  | { language: string; script: string };

type MatchLevel = 1 | 2 | 3 | 0;

let LANGUAGE = "";

function matches(spec: Language, language: Language): MatchLevel {
  if (spec.language == null) return 1;
  if (spec.language !== language.language) return 0;
  if (spec.script !== language.script) return 2;
  return 3;
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
  for (const lang of wantedByUser) {
    for (const s of available) {
      const match = matches(s, lang);
      if (match > best[0]) {
        best = [match, s];
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
  if (!(LANGUAGE in args))
    throw new Error(`${LANGUAGE} is not present in args`);
  return args[LANGUAGE];
}
