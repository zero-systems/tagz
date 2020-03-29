use super::*;

pub fn file_id_and_tag_id_exists(file_id: i32, tag_id: i32, conn: &Connection) -> SqlResult<bool> {
    conn.prepare("SELECT 1 FROM `file_tags` WHERE `tag_id`=? AND `file_id`=? LIMIT 1")?
        .query_row(params! {file_id, tag_id}, |row| row.get(0))
        .optional()
        .map(|x: Option<i32>| x.is_some())
}

pub fn delete_between_file_id_and_tag_id(
    file_id: i32,
    tag_id: i32,
    conn: &Connection,
) -> SqlResult<bool> {
    conn.prepare("DELETE FROM `file_tags` WHERE `tag_id`=? AND `file_id`=?")?
        .query_row(params! {file_id, tag_id}, |row| row.get(0))
        .optional()
        .map(|x: Option<i32>| x.is_some())
}

#[derive(FromRow)]
pub struct FileTag {
    pub tag_id: i32,
    pub file_id: i32,
}

impl FileTag {
    pub fn create(file_id: i32, tag_id: i32, conn: &Connection) -> SqlResult<()> {
        conn.execute("INSERT INTO `file_tags` (`file_id`, `tag_id`) VALUES (?, ?)", params! {file_id, tag_id})
        .map(|_| ())
    }

    pub fn all_in_file_ids<I, S>(files: I, conn: &Connection) -> SqlResult<Vec<Self>>
    where
        I: ExactSizeIterator + Iterator<Item = S>,
        S: ToSql,
    {
        let mut qs = std::iter::repeat("?,")
            .take(files.len())
            .collect::<String>();

        qs.pop();

        conn.prepare(&["SELECT * FROM `file_tags` WHERE `file_id` IN (", &qs, ")"].concat())?
            .query_map(files, FromRow::from_row)?
            .collect()
    }
}
