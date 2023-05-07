use once_cell::sync::Lazy;
use rustflake::Snowflake;
use serde::Serialize;

use crate::SNOWFLAKE_EPOCH;

/// user id generator
// worker id is 1 for users
pub static mut SNOWFLAKE: Lazy<Snowflake> = Lazy::new(|| Snowflake::new(SNOWFLAKE_EPOCH, 1, 1));

#[derive(Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,

    /// the user's avatar url
    pub avatar: Option<String>,

    /**
     * 1 >> 0 = Staff
     * 1 >> 1 = Tester
     */
    pub flags: i32,
}
