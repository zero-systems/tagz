use super::*;

pub fn find_tags_by_names<S>(
    names: &[S],
    conn: &Connection,
) -> std::result::Result<Vec<models::Tag>, ServiceError<'static>>
where
    S: serde::ser::Serialize + AsRef<str> + ToString,
{
    let tags = models::Tag::find_all_where_in_names(names, &conn)?;

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

    let mut file = models::File::create_with_tags(
        filej.name,
        &tags.iter().map(|t| t.id).collect::<Box<[i32]>>(),
        &mut conn,
    )?;

    file.tags = tags;

    res::json!(file)
}

// ---
#[derive(Deserialize)]
pub struct ListQuery {
    pub page: u16,
    pub tags: Box<str>,
    pub exact: Option<bool>,
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
        query.page as u32,
        query.exact.unwrap_or(false),
        &conn,
    )?;

    res::json!(files)
}

//---
#[delete("{name}")]
pub async fn delete(
    conn: ConnLock,
    filename: web::Path<Box<str>>,
) -> Result<'static, impl Responder> {
    let conn = conn.lock().await;

    let file = models::File::extract_from_name(filename.as_ref(), &conn)?;
    file.unlink_all_tags(&conn)?;
    file.delete(&conn)?;

    res::no_content!()
}

//---
#[delete("{name}")]
pub async fn remove(
    conn: ConnLock,
    info: web::Path<(i32, Box<str>)>,
) -> Result<'static, impl Responder> {
    let conn = conn.lock().await;
    let tag = models::Tag::extract_from_name(&info.1, &conn)?;

    if models::relationships::file_id_and_tag_id_exists(info.0, tag.id, &conn)? {
        models::relationships::delete_between_file_id_and_tag_id(info.0, tag.id, &conn)?;

        res::no_content!()
    } else {
        Err(ServiceError::not_found(
            "RELATIONSHIP_NOT_FOUND",
            "File does not have specified tag or does not exist.",
        ))
    }
}

//---
#[post("{name}")]
pub async fn add(
    conn: ConnLock,
    info: web::Path<(i32, Box<str>)>,
) -> Result<'static, impl Responder> {
    let conn = conn.lock().await;
    let tag = models::Tag::extract_from_name(&info.1, &conn)?;

    if models::relationships::file_id_and_tag_id_exists(info.0, tag.id, &conn)? {
        Err(ServiceError::bad_request(
            "RELATIONSHIP_EXISTS",
            "File already has specified tag.",
        ))
    } else {
        models::File::extract_id_exists(info.0, &conn)?;

        models::relationships::FileTag::create(info.0, tag.id, &conn)?;

        res::no_content!()
    }
}
