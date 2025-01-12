use crate::{Builder, Cache, Location};

pub(crate) fn apply<C>(builder: Builder<C>) -> Builder<C>
where
    C: Cache,
{
    builder
        .source_module(
            Location::library("io").unwrap(),
            include_str!("./io.tri").to_owned(),
        )
        .source_module(
            Location::library("debug").unwrap(),
            include_str!("./debug.tri").to_owned(),
        )
        .native_module(Location::library("c").unwrap())
}
