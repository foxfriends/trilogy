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
        .source_module(
            Location::library("atom").unwrap(),
            include_str!("./atom.tri").to_owned(),
        )
        .source_module(
            Location::library("str").unwrap(),
            include_str!("./str.tri").to_owned(),
        )
        .source_module(
            Location::library("array").unwrap(),
            include_str!("./array.tri").to_owned(),
        )
        .source_module(
            Location::library("parsec").unwrap(),
            include_str!("./parsec.tri").to_owned(),
        )
        .source_module(
            Location::library("core").unwrap(),
            include_str!("./core.tri").to_owned(),
        )
}
