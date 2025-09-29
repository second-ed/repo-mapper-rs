use std::{collections::HashSet, path::PathBuf};

use regex::Regex;

pub fn to_collection_of_type<I, S, Out, C>(inp: I) -> C
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    Out: From<String>,
    C: FromIterator<Out>,
{
    inp.into_iter()
        .map(|s| Out::from(s.as_ref().to_string()))
        .collect()
}

pub fn to_pathbufs<I, S>(inp: I) -> Vec<PathBuf>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    to_collection_of_type::<_, _, PathBuf, Vec<_>>(inp)
}

pub fn to_strings<I, S>(inp: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    to_collection_of_type::<_, _, String, Vec<_>>(inp)
}

pub fn to_hashset<I, S>(inp: I) -> HashSet<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    to_collection_of_type::<_, _, String, HashSet<_>>(inp)
}

pub fn to_regex_vec<I, S>(inp: I) -> Vec<Regex>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    inp.into_iter()
        .filter_map(|s| Regex::new(s.as_ref()).ok())
        .collect::<Vec<Regex>>()
}
