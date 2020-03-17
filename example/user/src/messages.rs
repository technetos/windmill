use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct UserByIdReq {
    id: u64,
}

#[derive(Deserialize)]
pub struct User {
    user_id: u64,
    first_name: String,
    last_name: String,
}

#[derive(Deserialize)]
pub struct UserByIdRes {
    user: User,
}


