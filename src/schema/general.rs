table! {
    forumrs (id) {
        id -> Bigint,
        name -> Text,
        ver -> Text,
        updated ->Bigint,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "forumrs"]
pub struct ForumRSTable {
    pub id: i64,
    pub name: String,
    pub ver: String
}