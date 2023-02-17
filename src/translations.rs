use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum Language {
    #[default]
    Catchall,
    LanguageOnly(String),
    LanguageScript(String, String),
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Catchall => "".fmt(f),
            Language::LanguageOnly(l) => l.fmt(f),
            Language::LanguageScript(l, s) => write!(f, "{l}-{s}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum MatchLevel {
    NoMatch,
    MatchesCatchall,
    MatchesLanguage,
    MatchesScript,
}

fn matches(spec: &Language, language: &Language) -> MatchLevel {
    match spec {
        Language::Catchall => MatchLevel::MatchesCatchall,
        Language::LanguageOnly(l_spec) => match language {
            Language::LanguageOnly(l) | Language::LanguageScript(l, _) if l == l_spec => {
                MatchLevel::MatchesLanguage
            }
            _ => MatchLevel::NoMatch,
        },
        Language::LanguageScript(l_spec, s_spec) => match language {
            Language::LanguageScript(l, s) if l == l_spec && s == s_spec => {
                MatchLevel::MatchesScript
            }
            _ => MatchLevel::NoMatch,
        },
    }
}

fn parse(specifier: &str) -> Language {
    let mut parts = specifier.split(&['_', '-']);
    match parts.next() {
        None => Language::Catchall,
        Some(lang) if lang.is_empty() => Language::Catchall,
        Some(lang) => match parts.next() {
            None => Language::LanguageOnly(lang.to_ascii_lowercase()),
            Some(script) => {
                Language::LanguageScript(lang.to_ascii_lowercase(), script.to_ascii_uppercase())
            }
        },
    }
}

fn find_best_language(available: &[Language], wanted_by_user: &[Language]) -> Language {
    let mut best = (
        MatchLevel::NoMatch,
        available.get(0).cloned().unwrap_or_default(),
    );

    for spec in wanted_by_user {
        for language in available {
            let found = matches(spec, language);
            if found > best.0 {
                best = (found, language.clone());
            }
        }

        if best.0 > MatchLevel::NoMatch {
            return best.1;
        }
    }

    best.1
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    fn any_language_strategy() -> impl Strategy<Value = Language> {
        prop_oneof![
            Just(Language::Catchall),
            ".*".prop_map(Language::LanguageOnly),
            (".*", ".*").prop_map(|(l, s)| Language::LanguageScript(l, s))
        ]
    }

    fn fixed_language_strategy() -> impl Strategy<Value = (String, Language)> {
        prop_oneof![
            ".*".prop_map(|l| (l.clone(), Language::LanguageOnly(l))),
            (".*", ".*").prop_map(move |(l, s)| (l.clone(), Language::LanguageScript(l, s)))
        ]
    }

    fn never_script_strategy() -> impl Strategy<Value = Language> {
        prop_oneof![
            Just(Language::Catchall),
            ".*".prop_map(Language::LanguageOnly),
        ]
    }

    mod matches {
        use super::*;

        proptest! {
            #[test]
            fn matches_any_language_when_language_is_catchall(language in any_language_strategy()) {
                assert_eq!(matches(&Language::Catchall, &language), MatchLevel::MatchesCatchall)
            }

            #[test]
            fn matches_when_languages_are_equal((spec_lang, language) in fixed_language_strategy()) {
                assert_eq!(matches(&Language::LanguageOnly(spec_lang), &language), MatchLevel::MatchesLanguage)
            }

            #[test]
            fn never_matches_if_script_is_in_spec_but_not_testee(spec_lang in ".*", spec_script in ".*", language in never_script_strategy()) {
                assert_eq!(matches(&Language::LanguageScript(spec_lang, spec_script), &language), MatchLevel::NoMatch)
            }

            #[test]
            fn always_matches_itself(language in any_language_strategy()) {
                match language {
                    Language::Catchall => assert_eq!(matches(&language, &language), MatchLevel::MatchesCatchall),
                    Language::LanguageOnly(_) => assert_eq!(matches(&language, &language), MatchLevel::MatchesLanguage),
                    Language::LanguageScript(_, _) => assert_eq!(matches(&language, &language), MatchLevel::MatchesScript),
                }
            }
        }
    }

    mod parse {
        use super::*;

        #[test]
        fn returns_catch_all_language_if_empty_string_is_passed() {
            assert_eq!(parse(""), Language::Catchall);
        }

        #[test]
        fn returns_catch_language_if_only_language_is_passed() {
            assert_eq!(parse("en"), Language::LanguageOnly("en".to_owned()));
            assert_eq!(parse("*"), Language::LanguageOnly("*".to_owned()));
            assert_eq!(
                parse("NONSENSE"),
                Language::LanguageOnly("nonsense".to_owned())
            );
        }

        #[test]
        fn returns_language_and_script_if_both_are_provided() {
            assert_eq!(
                parse("en-US"),
                Language::LanguageScript("en".to_owned(), "US".to_owned())
            );
            assert_eq!(
                parse("*_%"),
                Language::LanguageScript("*".to_owned(), "%".to_owned())
            );
            assert_eq!(
                parse("a b c-D E F"),
                Language::LanguageScript("a b c".to_owned(), "D E F".to_owned())
            );
        }

        #[test]
        fn returns_the_language_and_script_but_ignores_further_subcomponents() {
            assert_eq!(
                parse("en-US-unknown"),
                Language::LanguageScript("en".to_owned(), "US".to_owned())
            );
            assert_eq!(
                parse("en_GB-unknown"),
                Language::LanguageScript("en".to_owned(), "GB".to_owned())
            );
            assert_eq!(
                parse("en-US_unknown"),
                Language::LanguageScript("en".to_owned(), "US".to_owned())
            );
            assert_eq!(
                parse("en_GB_unknown"),
                Language::LanguageScript("en".to_owned(), "GB".to_owned())
            );
        }

        #[test]
        fn normalises_the_case_of_the_tags() {
            assert_eq!(
                parse("en_us"),
                Language::LanguageScript("en".to_owned(), "US".to_owned())
            );
            assert_eq!(
                parse("EN_GB"),
                Language::LanguageScript("en".to_owned(), "GB".to_owned())
            );
            assert_eq!(
                parse("eN-Us"),
                Language::LanguageScript("en".to_owned(), "US".to_owned())
            );
            assert_eq!(
                parse("En-gB"),
                Language::LanguageScript("en".to_owned(), "GB".to_owned())
            );
        }
    }

    mod find_best_language {
        use super::*;

        fn lang(language: impl Into<String>) -> Language {
            Language::LanguageOnly(language.into())
        }
        fn lang_sc(language: impl Into<String>, script: impl Into<String>) -> Language {
            Language::LanguageScript(language.into(), script.into())
        }

        proptest! {
            #[test]
            fn returns_catch_all_if_no_options_given(wanted_by_user in prop::collection::vec(any_language_strategy(), 0..5)) {
                assert_eq!(find_best_language(&[], &wanted_by_user), Language::Catchall);
            }
        }

        proptest! {
            #[test]
            fn never_returns_a_language_that_was_not_present_in_original_array(available in prop::collection::vec(any_language_strategy(), 1..5), wanted_by_user in prop::collection::vec(any_language_strategy(), 0..5)) {
                let matched = find_best_language(&available, &wanted_by_user);
                assert!(available.contains(&matched));
            }
        }

        #[test]
        fn returns_first_exact_match_language_when_it_exists() {
            assert_eq!(
                find_best_language(&[lang_sc("en", "GB")], &[lang_sc("en", "GB"), lang("en")]),
                lang_sc("en", "GB")
            );
            assert_eq!(
                find_best_language(
                    &[lang("en"), lang_sc("en", "GB")],
                    &[lang_sc("en", "GB"), lang("en")]
                ),
                lang_sc("en", "GB")
            );
            assert_eq!(
                find_best_language(
                    &[Language::Catchall, lang_sc("en", "GB"), lang_sc("de", "DE")],
                    &[lang_sc("en", "GB"), lang_sc("de", "DE"), lang("en")]
                ),
                lang_sc("en", "GB")
            );
        }

        #[test]
        fn does_not_return_exact_match_language_if_no_part_matches() {
            assert_eq!(
                find_best_language(
                    &[Language::Catchall, lang_sc("de", "DE"), lang("en")],
                    &[lang_sc("en", "GB"), lang("en")]
                ),
                lang("en")
            );
        }

        #[test]
        fn returns_the_language_preferred_by_the_user_not_by_implementor() {
            assert_eq!(
                find_best_language(
                    &[Language::Catchall, lang_sc("de", "DE"), lang_sc("en", "GB")],
                    &[lang_sc("en", "GB"), lang_sc("de", "DE"), lang("en")]
                ),
                lang_sc("en", "GB")
            );
            assert_eq!(
                find_best_language(
                    &[Language::Catchall, lang_sc("de", "DE"), lang("en")],
                    &[lang_sc("en", "GB"), lang_sc("de", "DE"), lang("en")]
                ),
                lang_sc("de", "DE")
            );
            assert_eq!(
                find_best_language(
                    &[Language::Catchall, lang_sc("de", "DE"), lang("en")],
                    &[lang_sc("en", "GB"), lang("en"), lang_sc("de", "DE")]
                ),
                lang("en")
            );
            assert_eq!(
                find_best_language(
                    &[Language::Catchall, lang_sc("de", "DE"), lang_sc("en", "US")],
                    &[lang_sc("en", "GB"), lang("en"), lang_sc("de", "DE")]
                ),
                lang_sc("en", "US")
            );
        }
    }
}
