#[trilogy_derive::module(crate_name=crate)]
pub mod sql {
    use super::super::RUNTIME;
    use crate::{Result, Runtime};
    use sqlx::postgres::PgValueFormat;
    use sqlx::{Column, Executor, Postgres, Row};
    use trilogy_vm::{Array, Bits, Callable, Record, Value};

    #[cfg(feature = "postgres")]
    #[derive(Clone)]
    pub struct Pool {
        pool: sqlx::Pool<Postgres>,
    }

    #[cfg(feature = "postgres")]
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn pgpool(rt: Runtime, conn_str: Value) -> Result<()> {
        let conn_str = rt.typecheck::<String>(conn_str)?;
        let pool = RUNTIME.block_on(sqlx::Pool::connect(&conn_str));
        let pool = pool.map_err(|error| {
            rt.runtime_error(rt.r#struct(
                "SqlError",
                format!("Failed to connect to database: {error}"),
            ))
        })?;

        rt.r#return(Pool { pool })
    }

    #[cfg(not(feature = "postgres"))]
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn pgpool(rt: Runtime, conn_str: Value) -> Result<()> {
        rt.runtime_error(rt.r#struct("SqlError", "postgres support is not enabled"))
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn sql(rt: Runtime, strings: Value, variables: Value) -> Result<()> {
        let strings = rt.typecheck::<Array>(strings)?;
        let variables = rt.typecheck::<Array>(variables)?;
        let sql = strings
            .to_vec()
            .into_iter()
            .map(|s| rt.typecheck::<String>(s))
            .enumerate()
            .try_fold(String::new(), |acc, (i, part)| {
                Ok(format!("{acc} ${i} {}", part?))
            })?;
        rt.r#return(Query { sql, variables })
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn sql_raw(rt: Runtime, string: Value) -> Result<()> {
        let string = rt.typecheck::<String>(string)?;
        rt.r#return(Query {
            sql: string,
            variables: Array::new(),
        })
    }

    #[derive(Clone)]
    pub struct Query {
        sql: String,
        variables: Array,
    }

    #[trilogy_derive::module(crate_name=crate)]
    impl Pool {
        #[trilogy_derive::proc(crate_name=crate)]
        pub fn query(self, rt: Runtime, query: Value) -> Result<()> {
            let query = rt
                .typecheck::<Callable>(query)?
                .as_native()
                .ok_or_else(|| {
                    rt.runtime_error(
                        rt.r#struct("SqlError", "query must be created with `sql::sql`"),
                    )
                })?
                .downcast::<Query>()
                .ok_or_else(|| {
                    rt.runtime_error(
                        rt.r#struct("SqlError", "query must be created with `sql::sql`"),
                    )
                })?;
            let query = query.lock().unwrap();
            let mut conn = RUNTIME.block_on(self.pool.acquire()).map_err(|err| {
                rt.runtime_error(
                    rt.r#struct("SqlError", format!("Failed to acquire connection: {err}")),
                )
            })?;
            let query =
                query
                    .variables
                    .into_iter()
                    .try_fold(sqlx::query(&query.sql), |query, var| match var {
                        Value::String(s) => Ok(query.bind(s.as_ref().to_owned())),
                        Value::Bool(b) => Ok(query.bind(b)),
                        Value::Unit => Ok(query.bind(None::<bool>)), // NOTE: how best to send null?
                        _ => Err(rt.runtime_error(
                            rt.r#struct("SqlError", "Unsupported type bound to query"),
                        )),
                    })?;
            let result = RUNTIME.block_on(conn.fetch_all(query)).map_err(|err| {
                rt.runtime_error(rt.r#struct("SqlError", format!("Error in query: {err}")))
            })?;
            let result = result
                .into_iter()
                .map(|row| {
                    Ok(Value::from(
                        row.columns()
                            .iter()
                            .map(|col| {
                                Ok((Value::from(col.name()), {
                                    let raw = row.try_get_raw(col.ordinal()).unwrap();
                                    match raw.format() {
                                        PgValueFormat::Text => {
                                            Value::from(raw.as_str().map_err(|err| {
                                                rt.runtime_error(rt.r#struct(
                                                    "SqlError",
                                                    format!("Failed to retrieve value: {err}"),
                                                ))
                                            })?)
                                        }
                                        PgValueFormat::Binary => Value::from(
                                            raw.as_bytes()
                                                .map_err(|err| {
                                                    rt.runtime_error(rt.r#struct(
                                                        "SqlError",
                                                        format!("Failed to retrieve value: {err}"),
                                                    ))
                                                })?
                                                .iter()
                                                .collect::<Bits>(),
                                        ),
                                    }
                                }))
                            })
                            .collect::<Result<Record>>()?,
                    ))
                })
                .collect::<Result<Array>>()?;
            rt.r#return(result)
        }
    }

    #[trilogy_derive::module(crate_name=crate)]
    impl Query {}
}
