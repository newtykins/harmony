// Sun Jan 01 2023 00:00:00 GMT+0000 (Greenwich Mean Time)
pub const SNOWFLAKE_EPOCH: i64 = 1672531200;

pub fn get_db<'t>(ctx: &async_graphql::Context<'t>) -> &'t tokio_postgres::Client {
    ctx.data::<tokio_postgres::Client>().unwrap()
}
