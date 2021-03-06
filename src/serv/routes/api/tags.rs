use super::*;

#[derive(Deserialize)]
pub struct Tag {
    pub name: Box<str>,
}

#[post("")]
pub async fn create(conn: ConnLock, tagj: web::Json<Tag>) -> Result<impl Responder> {
    let conn = conn.lock().await;

    if models::Tag::name_exists(tagj.name.as_ref(), &conn)? {
        Err(service_error::consts::TAG_DUPLICATION.clone())
    } else {
        res::json!(models::Tag::create(tagj.name.as_ref(), &conn)?)
    }
}

// ---
#[derive(Deserialize)]
pub struct DeleteQuery {
    pub confirm: Option<bool>,
}

#[delete("{name}")]
pub async fn delete(
    conn: ConnLock,
    query: web::Query<DeleteQuery>,
    name: web::Path<Box<str>>,
) -> Result<impl Responder> {
    let conn = conn.lock().await;

    // get tag
    let tag = models::Tag::extract_from_name(name.as_ref().as_ref(), &conn)?;

    // check if there are related files
    let has_files = tag.has_related_files(&conn)?;

    if has_files && query.confirm != Some(true) {
        Err(service_error::consts::CONFIRMATION_REQUIRED.clone().with_message("Tag has related files, so all files with this tag will be unlinked. Confirm action by adding `?confirm=true` to query url."))
    } else {
        if has_files {
            tag.unlink_all_files(&conn)?;
        }

        tag.delete(&conn)?;

        res::no_content!()
    }
}

//---
#[get("")]
pub async fn list(conn: ConnLock) -> Result<impl Responder> {
    let conn = conn.lock().await;

    res::json!(models::Tag::all(&conn)?
        .iter()
        .map(|tag| &tag.name)
        .collect::<Box<[_]>>())
}
