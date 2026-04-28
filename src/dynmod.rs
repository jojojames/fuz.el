use emacs::{defun, Env, Result, Value, Vector, IntoLisp};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::clangd::ClangdMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use rayon::prelude::*;

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

fn score_all_impl<'e, M: FuzzyMatcher + Default>(
    env: &'e Env,
    collection: Value<'e>,
    pattern: String,
) -> Result<Value<'e>> {
    // Convert list or vector to an indexed Lisp vector.
    let vec: Vector = env.call("vconcat", [collection])?.into_rust()?;
    let len = vec.len();

    // Extract strings on the main thread — Emacs API is not thread-safe.
    let strings: Vec<Option<String>> = (0..len)
        .map(|i| vec.get::<String>(i).ok())
        .collect();

    // Score candidates in parallel; no Emacs API calls in this section.
    let matcher = M::default();
    let mut scored: Vec<(usize, i64)> = strings
        .par_iter()
        .enumerate()
        .filter_map(|(i, s)| {
            s.as_deref()
             .and_then(|s| matcher.fuzzy_match(s, &pattern).map(|score| (i, score)))
        })
        .collect();

    // Best score first.
    scored.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    // Build a cons list.  Iterating in reverse and prepending yields highest-score first.
    let completion_score = env.intern("completion-score")?;
    let mut result = env.intern("nil")?;
    for &(idx, score) in scored.iter().rev() {
        let elem: Value = vec.get(idx)?;
        // put-text-property requires the string to have at least one character.
        if strings[idx].as_deref().map_or(false, |s| !s.is_empty()) {
            env.call("put-text-property", (0i64, 1i64, completion_score, score, elem))?;
        }
        result = env.cons(elem, result)?;
    }

    Ok(result)
}

/// Score all items in COLLECTION against PATTERN using skim's fuzzy algorithm.
///
/// Returns matched items sorted by score descending, with `completion-score'
/// text property set on each.
///
/// Sign: (-> (Seqof Str) Str (Listof Str))
///
/// (fn COLLECTION PATTERN)
#[defun]
fn score_all_skim<'e>(env: &'e Env, collection: Value<'e>, pattern: String) -> Result<Value<'e>> {
    score_all_impl::<SkimMatcherV2>(env, collection, pattern)
}

/// Score all items in COLLECTION against PATTERN using clangd's fuzzy algorithm.
///
/// Returns matched items sorted by score descending, with `completion-score'
/// text property set on each.
///
/// Sign: (-> (Seqof Str) Str (Listof Str))
///
/// (fn COLLECTION PATTERN)
#[defun]
fn score_all_clangd<'e>(env: &'e Env, collection: Value<'e>, pattern: String) -> Result<Value<'e>> {
    score_all_impl::<ClangdMatcher>(env, collection, pattern)
}
