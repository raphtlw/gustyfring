use sqlx::FromRow;

#[derive(FromRow, Debug)]
#[sqlx(rename_all = "camelCase")]
pub struct Member {
    pub tg_user_id: String,
}

#[derive(FromRow, Debug)]
#[sqlx(rename_all = "camelCase")]
pub struct Stat {
    pub ls: i32,
}

#[derive(FromRow, Debug)]
#[sqlx(rename_all = "camelCase")]
pub struct MemberStat {
    pub tg_user_id: String,
    pub ls: i32,
}

#[derive(FromRow, Debug)]
#[sqlx(rename_all = "camelCase")]
pub struct Phrase {
    pub id: i32,
    pub author_id: i32,
    pub content: String,
}

#[derive(FromRow, Debug)]
#[sqlx(rename_all = "camelCase")]
pub struct DialogTurn {
    pub phrase: String,
    pub response: String,
}
