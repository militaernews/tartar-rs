
    use serde::{Deserialize, Serialize};
    use tokio_pg_mapper_derive::PostgresMapper;
    use chrono::prelude::*;
    use chrono::NaiveDateTime;

    #[derive(Deserialize)]

    pub struct InputUser {
        pub id: i32,
        pub msg: String,

    }

    #[derive(Serialize)]
    pub struct User {
        pub id: i32,
        pub msg: String,
        pub date: NaiveDateTime,
    }


    /*
    #[derive(Serialize, PostgresMapper,  Debug)]
    #[serde(crate = "reports")]
    pub struct Report {
        pub id: i32,
        pub author: i32,
        pub date: NaiveDateTime,
        pub user_id: i32,
        pub user_msg: String,
    }*/