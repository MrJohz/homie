import * as fc from "fast-check";
import {
  beforeEach,
  describe,
  expect,
  SpyInstance,
  test,
  vitest,
} from "vitest";
import { Language, setup, t, _findBestLanguage, _parse } from "./translations";

function lng(language: string | null, script?: string): Language {
  if (language === null) return { language };
  return { language, script: script ?? null };
}

const partialLanguageCode = fc.stringOf(
  fc
    .integer({ min: "a".charCodeAt(0), max: "z".charCodeAt(0) })
    .map(String.fromCharCode),
  { minLength: 2, maxLength: 2 }
);
const fullLanguageCode = fc
  .tuple(
    partialLanguageCode,
    fc.constantFrom("_", "-"),
    partialLanguageCode.map((code) => code.toUpperCase())
  )
  .map(([language, sep, script]) => `${language}${sep}${script}`);
const language = fc
  .oneof(partialLanguageCode, fullLanguageCode, fc.constantFrom(""))
  .map(_parse);

let LANGUAGE_GETTER: SpyInstance<[], readonly string[]>;

beforeEach(() => {
  LANGUAGE_GETTER = vitest.spyOn(window.navigator, "languages", "get");
});

describe("parse", () => {
  test("returns the catch-all language if the empty string is passed", () => {
    expect(_parse("")).toEqual({ language: null });
  });

  test("returns the language and a null script if only the language is passed", () => {
    expect(_parse("en")).toEqual({ language: "en", script: null });
    expect(_parse("*")).toEqual({ language: "*", script: null });
    expect(_parse("THIS IS NONSENSE")).toEqual({
      language: "THIS IS NONSENSE",
      script: null,
    });
  });

  test("returns the language and a script if both are provided", () => {
    expect(_parse("en-US")).toEqual({ language: "en", script: "US" });
    expect(_parse("en_GB")).toEqual({ language: "en", script: "GB" });
    expect(_parse("a b c-D E F")).toEqual({
      language: "a b c",
      script: "D E F",
    });
  });

  test("returns the language and script but ignores further subcomponents", () => {
    expect(_parse("en-US-unknown")).toEqual({ language: "en", script: "US" });
    expect(_parse("en_GB-unknown")).toEqual({ language: "en", script: "GB" });
    expect(_parse("en-US_unknown")).toEqual({ language: "en", script: "US" });
    expect(_parse("en_GB_unknown")).toEqual({ language: "en", script: "GB" });
  });

  test("normalises the case of the script tag", () => {
    expect(_parse("en-us")).toEqual({ language: "en", script: "US" });
    expect(_parse("en_gB")).toEqual({ language: "en", script: "GB" });
    expect(_parse("a b c-d E f")).toEqual({
      language: "a b c",
      script: "D E F",
    });
  });
});

describe("findBestLanguage", () => {
  test("returns the catch-all language if no options are given", () => {
    expect(_findBestLanguage([], [lng("en", "GB"), lng("en")])).toEqual(
      lng(null)
    );
  });

  test("returns the first exact-match language when it exists", () => {
    expect(
      _findBestLanguage([lng("en", "GB")], [lng("en", "GB"), lng("en")])
    ).toEqual(lng("en", "GB"));
    expect(
      _findBestLanguage(
        [lng("en"), lng("en", "GB")],
        [lng("en", "GB"), lng("en")]
      )
    ).toEqual(lng("en", "GB"));
    expect(
      _findBestLanguage(
        [lng(null), lng("en", "GB"), lng("de", "DE")],
        [lng("en", "GB"), lng("de", "DE"), lng("en")]
      )
    ).toEqual(lng("en", "GB"));
  });

  test("does not return an exact-match language if no part of it matches", () => {
    expect(
      _findBestLanguage(
        [lng(null), lng("de", "DE"), lng("en")],
        [lng("en", "GB"), lng("en")]
      )
    ).toEqual(lng("en"));
  });

  test("returns the language preferred by the user, not by the implementer", () => {
    expect(
      _findBestLanguage(
        [lng(null), lng("de", "DE"), lng("en", "GB")],
        [lng("en", "GB"), lng("de", "DE"), lng("en")]
      )
    ).toEqual(lng("en", "GB"));
    expect(
      _findBestLanguage(
        [lng(null), lng("de", "DE"), lng("en")],
        [lng("en", "GB"), lng("de", "DE"), lng("en")]
      )
    ).toEqual(lng("de", "DE"));
    expect(
      _findBestLanguage(
        [lng(null), lng("de", "DE"), lng("en")],
        [lng("en", "GB"), lng("en"), lng("de", "DE")]
      )
    ).toEqual(lng("en"));
    expect(
      _findBestLanguage(
        [lng(null), lng("de", "DE"), lng("en", "US")],
        [lng("en", "GB"), lng("en"), lng("de", "DE")]
      )
    ).toEqual(lng("en", "US"));
  });

  test("never returns a language that wasn't present in the original array", () => {
    fc.assert(
      fc.property(
        fc.tuple(fc.array(language, { minLength: 1 }), fc.array(language)),
        ([available, wantedByUser]) => {
          expect(available).toContain(
            _findBestLanguage(available, wantedByUser)
          );
        }
      )
    );
  });
});

describe("setup/t", () => {
  test("sets a language based on the best known option", () => {
    LANGUAGE_GETTER.mockReturnValue(["en_GB", "en"]);
    setup(["en_GB", "de_DE"]);
    expect(t({ en_GB: "success" })).toEqual("success");
  });

  test("returns a default value if no options are valid", () => {
    LANGUAGE_GETTER.mockReturnValue(["fr_FR"]);
    setup(["en_GB", "de_DE"]);

    expect(t({ en_GB: "success" })).toEqual("success");
  });

  test("throws an error if the translated key is not present", () => {
    LANGUAGE_GETTER.mockReturnValue(["fr_FR"]);
    setup(["en_GB", "de_DE"]);

    expect(() => t({ fr_FR: "success" })).toThrow(/en_GB/);
  });

  test("throws an error if no languages submitted", () => {
    LANGUAGE_GETTER.mockReturnValue(["fr_FR"]);
    expect(() => setup([])).toThrow();
  });
});
