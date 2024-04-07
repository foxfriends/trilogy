#[trilogy_derive::module(crate_name=crate)]
pub mod sql {
    use crate::{Result, Runtime};
    use sqlx::{Column, Executor, Postgres, Row, TypeInfo};
    use tokio::runtime::Handle;
    use trilogy_vm::{Array, Callable, Record, Value};

    #[cfg(feature = "postgres")]
    #[derive(Clone)]
    pub struct Pool {
        pool: sqlx::Pool<Postgres>,
    }

    #[cfg(feature = "postgres")]
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn pgpool(rt: Runtime, conn_str: Value) -> Result<()> {
        let conn_str = rt.typecheck::<String>(conn_str)?;
        let pool = Handle::current().block_on(sqlx::Pool::connect(&conn_str));
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
        let mut iter = strings
            .to_vec()
            .into_iter()
            .map(|s| rt.typecheck::<String>(s))
            .enumerate();
        let Some((_, init)) = iter.next() else {
            return rt.r#return(Query {
                sql: "".to_owned(),
                variables,
            });
        };
        let sql = iter.try_fold(init?, |acc, (i, part)| Ok(format!("{acc}${i}{}", part?)))?;
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
            let mut conn = Handle::current()
                .block_on(self.pool.acquire())
                .map_err(|err| {
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
                        Value::Number(n) if n.is_integer() => {
                            let decimal = sqlx::types::BigDecimal::from(n.as_integer().unwrap());
                            Ok(query.bind(decimal))
                        }
                        Value::Bool(b) => Ok(query.bind(b)),
                        Value::Unit => Ok(query.bind(None::<bool>)), // NOTE: how best to send null?
                        _ => Err(rt.runtime_error(
                            rt.r#struct("SqlError", "Unsupported type bound to query"),
                        )),
                    })?;
            let result = Handle::current()
                .block_on(conn.fetch_all(query))
                .map_err(|err| {
                    rt.runtime_error(rt.r#struct("SqlError", format!("Error in query: {err}")))
                })?;
            let result = result
                .into_iter()
                .map(|row| {
                    Ok(Value::from(
                        row.columns()
                            .iter()
                            .map(|col| {
                                let value: Value = match col.type_info().name() {
                                    "CITEXT" | "TEXT" | "VARCHAR" | "CHAR" => row
                                        .try_get::<String, _>(col.ordinal())
                                        .map_err(|err| {
                                            rt.runtime_error(rt.r#struct(
                                                "SqlError",
                                                format!("Failed to retrieve value: {err}"),
                                            ))
                                        })?
                                        .into(),
                                    "INT4" => row
                                        .try_get::<i32, _>(col.ordinal())
                                        .map_err(|err| {
                                            rt.runtime_error(rt.r#struct(
                                                "SqlError",
                                                format!("Failed to retrieve value: {err}"),
                                            ))
                                        })?
                                        .into(),
                                    name => {
                                        return Err(rt.runtime_error(rt.r#struct(
                                            "SqlError",
                                            format!("Unsupported SQL type: {name}"),
                                        )))
                                    }
                                };
                                Ok((Value::from(col.name()), value))
                            })
                            .collect::<Result<Record>>()?,
                    ))
                })
                .collect::<Result<Array>>()?;
            rt.r#return(result)
        }
    }

    #[trilogy_derive::module(crate_name=crate)]
    impl Query {
        #[trilogy_derive::proc(crate_name=crate)]
        pub fn to_string(self, rt: Runtime) -> Result<()> {
            rt.r#return(self.sql)
        }
    }
}
