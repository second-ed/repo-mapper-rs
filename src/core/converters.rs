use std::{collections::HashSet, path::PathBuf};

use regex::Regex;

pub fn to_pathbufs<I, S>(inp: I) -> Vec<PathBuf>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    inp.into_iter().map(|s| PathBuf::from(s.as_ref())).collect()
}

pub fn to_strings<I, S>(inp: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    inp.into_iter().map(|s| s.as_ref().to_string()).collect()
}

pub fn to_hashset<I, S>(inp: I) -> HashSet<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    inp.into_iter()
        .map(|s| s.as_ref().to_string())
        .collect::<HashSet<_>>()
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
