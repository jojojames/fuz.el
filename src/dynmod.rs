use emacs::{defun, Env, Result, Value, IntoLisp};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::clangd::ClangdMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

fn find_indices_into_lisp<'a, F>(env: &'a Env, fun: F, pat: &str, source: &str)
    -> Option<Vec<Value<'a>>>
where
    F: Fn(&str, &str) -> Option<(i64, Vec<usize>)>,
{
    fun(source, pat).map(|(_score, indices)| {
        indices
            .into_iter()
            .map(|it| (it as i64).into_lisp(env).unwrap())
            .collect()
    })
}

/// Return the PATTERN fuzzy score about SOURCE, using skim's fuzzy algorithm.
///
/// Sign: (-> Str Str (Option Long))
///
/// Return nil if no match happened.
///
/// (fn PATTERN SOURCE)
#[defun]
fn calc_score_skim(_env: &Env, pattern: String, source: String) -> Result<Option<i64>> {
    Ok(SkimMatcherV2::default().fuzzy_match(&source, &pattern))
}

/// Return the PATTERN fuzzy score about SOURCE, using clangd's fuzzy algorithm.
///
/// Sign: (-> Str Str (Option Long))
///
/// See `fuz-calc-score-skim' for more information
///
/// (fn PATTERN SOURCE)
#[defun]
fn calc_score_clangd(_env: &Env, pattern: String, source: String) -> Result<Option<i64>> {
    Ok(ClangdMatcher::default().fuzzy_match(&source, &pattern))
}

/// Find the indices for a PATTERN matching SOURCE, using skim's fuzzy algorithm.
///
/// Sign: (-> Str Str (Listof Long))
///
/// Return a list of integer that marks the position of matched char.
///
/// Return nil if nothing was matched.
///
/// (fn PATTERN SOURCE)
#[defun]
fn find_indices_skim(env: &Env, pattern: String, source: String) -> Result<Option<Value<'_>>> {
    let matcher = SkimMatcherV2::default();
    match find_indices_into_lisp(env, |s, p| matcher.fuzzy_indices(s, p), &pattern, &source) {
        Some(val) => Ok(Some(env.list(&val[..])?)),
        None => Ok(None),
    }
}

/// Find the indices for a PATTERN matching SOURCE, using clangd's fuzzy algorithm.
///
/// Sign: (-> Str Str (Listof Long))
///
/// See `fuz-find-indices-skim' for more infomation
///
/// (fn PATTERN SOURCE)
#[defun]
fn find_indices_clangd(env: &Env, pattern: String, source: String) -> Result<Option<Value<'_>>> {
    let matcher = ClangdMatcher::default();
    match find_indices_into_lisp(env, |s, p| matcher.fuzzy_indices(s, p), &pattern, &source) {
        Some(val) => Ok(Some(env.list(&val[..])?)),
        None => Ok(None),
    }
}
