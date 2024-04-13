#![allow(clippy::module_inception, clippy::wrong_self_convention)]

mod array;
mod atom;
mod bits;
mod env;
mod fs;
mod io;
mod num;
mod str;
mod time;

#[cfg(feature = "regex")]
mod regex;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "sql")]
mod sql;

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
        .asm_module(
            Location::library("struct/asm").unwrap(),
            include_str!("./struct.asm").to_owned(),
        )
        .source_module(
            Location::library("struct").unwrap(),
            include_str!("./struct.tri").to_owned(),
        )
        .native_module(Location::library("fs/native").unwrap(), fs::fs())
        .source_module(
            Location::library("fs").unwrap(),
            include_str!("./fs.tri").to_owned(),
        )
        .native_module(Location::library("env/native").unwrap(), env::env())
        .source_module(
            Location::library("env").unwrap(),
            include_str!("./env.tri").to_owned(),
        )
        .native_module(Location::library("time/native").unwrap(), time::time())
        .source_module(
            Location::library("time").unwrap(),
            include_str!("./time.tri").to_owned(),
        )
        .source_module(
            Location::library("iter").unwrap(),
            include_str!("./iter.tri").to_owned(),
        )
        .source_module(
            Location::library("fp").unwrap(),
            include_str!("./fp.tri").to_owned(),
        )
        .source_module(
            Location::library("range").unwrap(),
            include_str!("./range.tri").to_owned(),
        )
        .source_module(
            Location::library("btree").unwrap(),
            include_str!("./btree.tri").to_owned(),
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

    #[cfg(feature = "json")]
    {
        builder = builder
            .native_module(Location::library("json/native").unwrap(), json::json())
            .source_module(
                Location::library("json").unwrap(),
                include_str!("./json.tri").to_owned(),
            );
    }

    #[cfg(feature = "sql")]
    {
        builder = builder
            .native_module(Location::library("sql/native").unwrap(), sql::sql())
            .source_module(
                Location::library("sql").unwrap(),
                include_str!("./sql.tri").to_owned(),
            );
    }

    builder
}
