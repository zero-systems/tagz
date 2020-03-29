use super::*;

pub fn find_tags_by_names<S>(
    names: &[S],
    conn: &Connection,
) -> std::result::Result<Vec<models::Tag>, ServiceError<'static>>
where
    S: serde::ser::Serialize + AsRef<str> + rusqlite::ToSql,
{
    let tags = models::Tag::find_where_in_name(&names, &conn)?;

    if tags.len() != names.len() {
        let mut hashset = std::collections::HashSet::new();

        for tag in &tags {
            hashset.insert(tag.name.as_ref());
        }

        let mut lost = Vec::with_capacity(names.len() - tags.len());

        for tag in names {
            if !hashset.contains(tag.as_ref()) {
                lost.push(tag);
            }
        }

        Err(ServiceError::bad_request(
            "TAGS_NOT_FOUND",
            "Could not found some tags to create file. Check details to get list of unknown tags.",
        )
        .with_details(lost))
    } else {
        Ok(tags)
    }
}

#[derive(Deserialize)]
pub struct File {
    pub name: Box<str>,
    pub tags: Vec<Box<str>>,
}

#[post("create")]
pub async fn create(conn: ConnLock, filej: web::Json<File>) -> Result<'static, impl Responder> {
    let filej = filej.0;
    let mut conn = conn.lock().await;

    if models::File::name_exists(&filej.name, &conn)? {
        return Err(ServiceError::bad_request("DUPLICATION", "File with specified name already exists. You must edit or delete it instead of recreating."));
    }

    let tags = find_tags_by_names(&filej.tags, &conn)?;

    let mut file =
        models::File::create_with_tags(filej.name, tags.iter().map(|t| t.id), &mut conn)?;

    file.tags = tags;

    res::json!(file)
}

// ---
#[derive(Deserialize)]
pub struct ListQuery {
    pub page: u32,
    pub tags: Box<str>,
}

#[get("list")]
pub async fn list(conn: ConnLock, query: web::Query<ListQuery>) -> Result<'static, impl Responder> {
    let conn = conn.lock().await;
    let tags = query.tags.split(',').collect::<Box<[_]>>();
    let tags = find_tags_by_names(tags.as_ref(), &conn)?;

    let ids = &tags.iter().map(|t| t.id).collect::<Box<[_]>>(); // FIXME: rusqlite: ToSql for Iterators ???

    let files = models::File::find_specific_amount_by_tags_ids_on_page(
        &ids,
        *crate::config::LIST_FILES_BY_TAG_PER_PAGE.lock().await,
        query.page,
        &conn,
    )?;

    res::json!(files)
}
