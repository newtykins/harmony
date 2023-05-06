use async_graphql::SimpleObject;
use harmony::SNOWFLAKE_EPOCH;

#[derive(SimpleObject)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub avatar: String,

    /*
        1 >> 0 = Staff
        1 >> 1 = Tester
    */
    pub flags: i32,
}

impl User {
    fn is_staff(&self) -> bool {
        self.flags << 0 == 1
    }

    fn is_tester(&self) -> bool {
        self.flags << 1 == 1
    }

    pub fn generate_snowflake() -> i64 {
        let mut snowflake_generator = rustflake::Snowflake::default();
        snowflake_generator.worker_id(1); // user ids are worker 1
        snowflake_generator.epoch(SNOWFLAKE_EPOCH);

        snowflake_generator.generate()
    }
}
