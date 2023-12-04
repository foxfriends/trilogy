#![allow(clippy::module_inception, clippy::wrong_self_convention)]

mod array;
mod atom;
mod bits;
mod fs;
mod io;
mod num;
mod str;
mod time;

#[cfg(feature = "regex")]
mod regex;

use crate::{Builder, Cache, Location};

pub(crate) fn apply<C>(builder: Builder<C>) -> Builder<C>
where
    C: Cache,
{
    let mut builder = builder
        .native_module(Location::library("io/native").unwrap(), io::io())
        .source_module(
            Location::library("io").unwrap(),
            include_str!("./io.tri").to_owned(),
        )
        .native_module(Location::library("str/native").unwrap(), str::str())
        .source_module(
            Location::library("str").unwrap(),
            include_str!("./str.tri").to_owned(),
        )
        .native_module(Location::library("num/native").unwrap(), num::num())
        .source_module(
            Location::library("num").unwrap(),
            include_str!("./num.tri").to_owned(),
        )
        .native_module(Location::library("bits/native").unwrap(), bits::bits())
        .source_module(
            Location::library("bits").unwrap(),
            include_str!("./bits.tri").to_owned(),
        )
        .native_module(Location::library("array/native").unwrap(), array::array())
        .source_module(
            Location::library("array").unwrap(),
            include_str!("./array.tri").to_owned(),
        )
        .source_module(
            Location::library("tuple").unwrap(),
            include_str!("./tuple.tri").to_owned(),
        )
        .source_module(
            Location::library("set").unwrap(),
            include_str!("./set.tri").to_owned(),
        )
        .source_module(
            Location::library("record").unwrap(),
            include_str!("./record.tri").to_owned(),
        )
        .native_module(Location::library("atom/native").unwrap(), atom::atom())
        .source_module(
            Location::library("atom").unwrap(),
            include_str!("./atom.tri").to_owned(),
        )
        .native_module(Location::library("fs/native").unwrap(), fs::fs())
        .source_module(
            Location::library("fs").unwrap(),
            include_str!("./fs.tri").to_owned(),
        )
        .source_module(
            Location::library("iter").unwrap(),
            include_str!("./iter.tri").to_owned(),
        )
        .source_module(
            Location::library("fp").unwrap(),
            include_str!("./fp.tri").to_owned(),
        )
        .native_module(Location::library("time/native").unwrap(), time::time())
        .source_module(
            Location::library("time").unwrap(),
            include_str!("./time.tri").to_owned(),
        );

    #[cfg(feature = "regex")]
    {
        builder = builder
            .native_module(Location::library("regex/native").unwrap(), regex::regex())
            .source_module(
                Location::library("regex").unwrap(),
                include_str!("./regex.tri").to_owned(),
            );
    }

    builder
}
