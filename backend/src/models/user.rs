use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct User {
    pub snowflake: i64,
    pub username: String,
    pub avatar: String
}
