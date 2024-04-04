#[trilogy_derive::module(crate_name=crate)]
pub mod sql {
    use crate::{Result, Runtime};
    use sqlx::Postgres;
    use trilogy_vm::Value;

    #[derive(Clone)]
    pub struct Pool {
        pool: sqlx::Pool<Postgres>,
    }

    #[cfg(feature = "postgres")]
    #[trilogy_derive::proc(crate_name=crate)]
    pub async fn pgpool(rt: Runtime, conn_str: Value) -> Result<()> {
        let conn_str = rt.typecheck::<String>(conn_str)?;
        let pool = sqlx::Pool::connect(&conn_str).await.map_err(|error| {
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

    #[trilogy_derive::module(crate_name=crate)]
    impl Pool {}
}
