use regex::Regex;

pub fn to_str_type<Out>(s: impl AsRef<str>) -> Out
where
    Out: From<String>,
{
    s.as_ref().to_owned().into()
}

pub fn to_collection_of_type<Out, C>(items: impl IntoIterator<Item = impl AsRef<str>>) -> C
where
    Out: From<String>,
    C: FromIterator<Out>,
{
    items.into_iter().map(to_str_type).collect()
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
