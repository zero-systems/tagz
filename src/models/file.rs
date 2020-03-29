use super::*;
use serde::ser::{SerializeSeq, Serializer};
use std::collections::BTreeMap;

#[derive(serde::Serialize, FromRow)]
pub struct File {
    pub id: i32,
    pub name: String,

    #[serde(serialize_with = "serialize_tags_vec")]
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
    pub fn create_with_tags<P>(name: P, tags: &[i32], conn: &mut Connection) -> SqlResult<Self>
    where
        P: ToSql,
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

    pub fn delete(&self, conn: &Connection) -> SqlResult<()> {
        conn.execute("DELETE FROM `files` WHERE `id`=?1", params! { self.id })
            .map(|_| ())
    }

    pub fn unlink_all_tags(&self, conn: &Connection) -> SqlResult<()> {
        conn.execute(
            "DELETE FROM `file_tags` WHERE `file_id`=?1",
            params! { self.id },
        )
        .map(|_| ())
    }

    pub fn extract_from_id(
        id: i32,
        conn: &Connection,
    ) -> Result<Self, serv_prelude::ServiceError<'static>> {
        Self::find_by_id(id, conn)?.ok_or_else(|| {
            serv_prelude::ServiceError::not_found(
                "FILE_NOT_FOUND",
                "Specified file_id cannot be found",
            )
        })
    }

    pub fn extract_id_exists(
        id: i32,
        conn: &Connection,
    ) -> Result<(), serv_prelude::ServiceError<'static>> {
        if !Self::id_exists(id, conn)? {
            Err(serv_prelude::ServiceError::not_found(
                "FILE_NOT_FOUND",
                "Specified file_id cannot be found",
            ))
        } else {
            Ok(())
        }
    }

    pub fn find_specific_amount_where_in_ids_on_page(ids: &[i32], amount: u32, page: u32, conn: &Connection) -> SqlResult<Vec<Self>> {
        let mut qs = std::iter::repeat("?,").take(ids.len()).collect::<String>();

        qs.pop();

        conn.prepare(&["SELECT * FROM `files` WHERE `id` IN (", qs.as_ref(), ") ORDER BY ID DESC LIMIT ? OFFSET ?"].concat())?
            .query_map(ids.iter().chain(&[amount as i32, (page*amount) as i32]), FromRow::from_row)?
            .collect()
    }

    pub fn find_by_id(id: i32, conn: &Connection) -> SqlResult<Option<Self>> {
        conn.prepare("SELECT * FROM `files` WHERE `id`=? LIMIT 1")?
            .query_row(params! {id}, FromRow::from_row)
            .optional()
    }

    pub fn extract_from_name<N>(
        name: N,
        conn: &Connection,
    ) -> Result<Self, serv_prelude::ServiceError<'static>>
    where
        N: ToSql,
    {
        Self::find_by_name(name, conn)?.ok_or_else(|| {
            serv_prelude::ServiceError::not_found(
                "FILE_NOT_FOUND",
                "Specified file cannot be found",
            )
        })
    }

    pub fn find_by_name<N>(name: N, conn: &Connection) -> SqlResult<Option<Self>>
    where
        N: ToSql,
    {
        conn.prepare("SELECT * FROM `files` WHERE `name`=?1 LIMIT 1")?
            .query_row(params! {name}, FromRow::from_row)
            .optional()
    }

    pub fn find_specific_amount_by_tags_ids_on_page(
        tags: &[i32],
        amount: u32,
        page: u32,
        exact: bool,
        conn: &Connection,
    ) -> SqlResult<Vec<Self>> {
        let mut qs = std::iter::repeat("?,").take(tags.len()).collect::<String>();
        let mut tags = tags.to_vec();
        tags.sort();

        qs.pop();

        let mut files = if exact {
            let rss = relationships::FileTag::all_for_tags_ids(tags.iter(), &conn)?;

            let mut files = BTreeMap::<i32, Vec<i32>>::new();

            for relationships::FileTag { file_id, tag_id } in rss {
                match files.get_mut(&file_id) {
                    Some(v) => {
                        v.push(tag_id);
                    }
                    None => {
                        files.insert(file_id, {
                            let mut v = Vec::with_capacity(tags.len());
                            v.push(tag_id);
                            v
                        });
                    }
                }
            }

            let ids: Vec<_> = files
                .into_iter()
                .filter_map(|(file_id, list)| if list == tags { Some(file_id) } else { None })
                .collect();

            File::find_specific_amount_where_in_ids_on_page(&ids, amount, page, &conn)?
        } else {
            conn.prepare(
                &[
                    "SELECT DISTINCT `files`.* FROM `file_tags` INNER JOIN `files` ON `id`=`file_id` WHERE `file_tags`.`tag_id` IN (",
                    &qs,
                    ") ORDER BY `id` DESC LIMIT ? OFFSET ?",
                ]
                .concat(),
            )?
            .query_map(tags.iter().chain(&[amount as i32, (amount * page) as i32]), FromRow::from_row)?
            .collect::<SqlResult<Vec<Self>>>()?
        };

        if files.len() == 0 {
            return Ok(files);
        } else {
            let relationships =
                relationships::FileTag::all_for_files_ids(files.iter().map(|f| f.id), &conn)?;
            let tags = Tag::find_all_where_in_ids(
                &relationships.iter().map(|t| t.tag_id).collect::<Box<[_]>>(),
                &conn,
            )?;

            let tags_map = tags
                .iter()
                .map(|t| (t.id, t))
                .collect::<BTreeMap<i32, &Tag>>();
            let mut files_map = files
                .iter_mut()
                .map(|f| (f.id, f))
                .collect::<BTreeMap<i32, &mut File>>(); // FIXME: mut ???

            for relationships::FileTag { tag_id, file_id } in relationships {
                let file = files_map.get_mut(&file_id).unwrap(); // FIXME: get_mut ???
                let tag = tags_map.get(&tag_id).unwrap();

                file.tags.push(tag.to_owned().clone());
            }

            Ok(files)
        }
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

    pub fn id_exists(id: i32, conn: &Connection) -> SqlResult<bool> {
        conn.prepare("SELECT 1 FROM `files` WHERE `id`=? LIMIT 1")?
            .query_row(params! {id}, |row| row.get(0))
            .optional()
            .map(|x: Option<i32>| x.is_some())
    }

    pub fn update_tags(&mut self, conn: &Connection) -> SqlResult<()> {
        self.tags = Tag::find_related_to_file(self.id, conn)?;

        Ok(())
    }
}

fn serialize_tags_vec<S>(src: &Vec<Tag>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(src.len()))?;
    for e in src {
        seq.serialize_element(&e.name)?;
    }
    seq.end()
}
