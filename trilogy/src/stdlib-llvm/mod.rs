use crate::{Builder, Cache, Location};

pub(crate) fn apply<C>(builder: Builder<C>) -> Builder<C>
where
    C: Cache,
{
    builder.source_module(
        Location::library("io").unwrap(),
        include_str!("./io.tri").to_owned(),
    )
}
