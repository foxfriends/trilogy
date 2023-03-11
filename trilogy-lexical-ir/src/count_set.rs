use std::{collections::HashMap, hash::Hash};

pub(crate) struct CountSet<K>(pub HashMap<K, usize>);

fn group_by<T, F, K>(key_fn: F) -> impl Fn(HashMap<K, Vec<T>>, T) -> HashMap<K, Vec<T>>
where
    K: Hash + Eq,
    F: Fn(&T) -> K,
{
    move |mut map, item| {
        let key = key_fn(&item);
        map.entry(key).or_default().push(item);
        map
    }
}
