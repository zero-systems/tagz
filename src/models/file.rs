use super::*;

#[derive(serde::Serialize, FromRow)]
pub struct File {
    pub id: i32,
    pub name: String,
    #[field_default]
    pub tags: Vec<Tag>,
    pub updated_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

macro_rules! insert {
    ($conn:expr, $name:expr) => {
        $conn.execute(
            "INSERT INTO `files` (name) VALUES(?1)",
            params! {
                $name
            },
        )
    };
}

impl File {
    pub fn create_with_tags<P, I>(name: P, tags: I, conn: &mut Connection) -> SqlResult<Self>
    where
        P: ToSql,
        I: Iterator<Item = i32>,
    {
        insert!(conn, name)?;

        let inst = tagz_cg_serv::last_inserted!(&conn, "files")?;

        let tx = conn.transaction()?;
        let mut stmt = tx.prepare("INSERT INTO `file_tags` (file_id, tag_id) VALUES(?1, ?2)")?;

        for tag in tags {
            stmt.execute(params![inst.id, tag])?;
        }

        drop(stmt);
        tx.commit()?;

        Ok(inst)
    }

    pub fn create<P>(name: P, conn: &Connection) -> SqlResult<Self>
    where
        P: ToSql,
    {
        insert!(conn, name)?;

        tagz_cg_serv::last_inserted!(&conn, "files")
    }

    pub fn find_with_tags_ids(tags: &[i32], conn: &Connection) -> SqlResult<Vec<Self>> {
        let mut qs = std::iter::repeat("?,").take(tags.len()).collect::<String>();

        qs.pop();

        conn.prepare(
            &[
                "SELECT `files`.* FROM `file_tags` INNER JOIN `files` ON `id`=`file_id` WHERE `file_tags`.`tag_id` IN (",
                &qs,
                ")",
            ]
            .concat(),
        )?
        .query_map(tags, Self::from_row)?
        .collect()
    }

    pub fn name_exists<P>(name: P, conn: &Connection) -> SqlResult<bool>
    where
        P: ToSql,
    {
        conn.prepare("SELECT 1 FROM `files` WHERE `name`=?1 LIMIT 1")?
            .query_row(params! {name}, |row| row.get(0))
            .optional()
            .map(|x: Option<i32>| x.is_some())
    }

    pub fn update_tags(&mut self, conn: &Connection) -> SqlResult<()> {
        self.tags = Tag::find_related_to_file(self.id, conn)?;

        Ok(())
    }
}
