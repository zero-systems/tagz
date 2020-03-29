use futures::lock::Mutex;
use std::sync::Arc;

lazy_static! {
    pub static ref LIST_TAGS_PER_PAGE: Arc<Mutex<u32>> = Arc::new(Mutex::new(50));
    pub static ref LIST_FILES_PER_PAGE: Arc<Mutex<u32>> = Arc::new(Mutex::new(50));
    pub static ref LIST_FILES_BY_TAG_PER_PAGE: Arc<Mutex<u32>> = Arc::new(Mutex::new(2));
}

pub static TABLES: &'static [&'static str] = &[
    // `files`
    r#"
		CREATE TABLE `files` (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			name VACHAR(4096) UNIQUE NOT NULL,
			updated_at TIMESTAMP NOT NULL DEFAULT(CURRENT_TIMESTAMP),
			created_at TIMESTAMP NOT NULL DEFAULT(CURRENT_TIMESTAMP)
		)
	"#,
    // `tags`
    r#"
		CREATE TABLE `tags` (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			name VACHAR(256) UNIQUE NOT NULL,
			created_at TIMESTAMP NOT NULL DEFAULT(CURRENT_TIMESTAMP)
		)
	"#,
    //// relations
    // `file_tags`
    r#"
		CREATE TABLE `file_tags` (
			file_id INTEGER NOT NULL,
			tag_id INTEGER NOT NULL,
			
		    PRIMARY KEY (file_id, tag_id)
		)
	"#,
];
