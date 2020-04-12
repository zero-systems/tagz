use super::*;

#[derive(Clone, Debug, FromRow, serde::Serialize)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
}

impl Tag {
    pub fn create<N>(name: N, conn: &Connection) -> SqlResult<Self>
    where
        N: ToSql,
    {
        conn.execute("INSERT INTO `tags` (name) VALUES(?1)", params! {name})?;

        tagz_cg_serv::last_inserted!(&conn, "tags")
    }

    pub fn unlink_all_files(&self, conn: &Connection) -> SqlResult<()> {
        conn.execute(
            "DELETE FROM `file_tags` WHERE `tag_id`=?1",
            params! { self.id },
        )
        .map(|_| ())
    }

    pub fn delete(&self, conn: &Connection) -> SqlResult<()> {
        conn.execute("DELETE FROM `tags` WHERE `id`=?1", params! { self.id })
            .map(|_| ())
    }

    pub fn has_related_files(&self, conn: &Connection) -> SqlResult<bool> {
        conn.prepare("SELECT 1 FROM `file_tags` WHERE `tag_id`=?1 LIMIT 1")?
            .query_row(params! {self.id}, |row| row.get(0))
            .optional()
            .map(|x: Option<i32>| x.is_some())
    }

    pub fn extract_from_name<N>(
        name: N,
        conn: &Connection,
    ) -> Result<Self, serv_prelude::ServiceError>
    where
        N: ToSql,
    {
        Self::find_by_name(name, conn)?.ok_or_else(|| {
            serv_prelude::ServiceError::not_found("TAG_NOT_FOUND", "Specified tag cannot be found")
        })
    }

    pub fn find_by_name<N>(name: N, conn: &Connection) -> SqlResult<Option<Self>>
    where
        N: ToSql,
    {
        conn.prepare("SELECT * FROM `tags` WHERE `name`=?1 LIMIT 1")?
            .query_row(params! {name}, FromRow::from_row)
            .optional()
    }

    pub fn name_exists<N>(name: N, conn: &Connection) -> SqlResult<bool>
    where
        N: ToSql,
    {
        conn.prepare("SELECT 1 FROM `tags` WHERE `name`=?1 LIMIT 1")?
            .query_row(params! {name}, |row| row.get(0))
            .optional()
            .map(|x: Option<i32>| x.is_some())
    }

    pub fn find_all_where_in_names<S: ToString>(
        names: &[S],
        conn: &Connection,
    ) -> SqlResult<Vec<Self>> {
        let names = RuSqlArray::new(
            names
                .into_iter()
                .map(|x| RuSqlValue::Text(x.to_string()))
                .collect(),
        );

        conn.prepare("SELECT * FROM `tags` WHERE `name` IN rarray(?)")?
            .query_map(&[&names], FromRow::from_row)?
            .collect()
    }

    pub fn find_all_where_in_ids(ids: &[i32], conn: &Connection) -> SqlResult<Vec<Self>> {
        let ids = RuSqlArray::new(ids.iter().map(|x| RuSqlValue::Integer(*x as i64)).collect());

        conn.prepare("SELECT * FROM `tags` WHERE `id` IN rarray(?)")?
            .query_map(&[&ids], FromRow::from_row)?
            .collect()
    }

    pub fn find_related_to_file(file: i32, conn: &Connection) -> SqlResult<Vec<Self>> {
        conn.prepare(
            "SELECT `tags`.* FROM `file_tags` WHERE `file_id`=?1 INNER JOIN `tags` ON `id`=`tag_id`",
        )?
        .query_map(params! {file}, Self::from_row)?
        .collect()
    }

    pub fn all(conn: &Connection) -> SqlResult<Vec<Self>> {
        conn.prepare("SELECT * FROM `tags`")?
            .query_map(params! {}, Self::from_row)?
            .collect()
    }
}
