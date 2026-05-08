use tauri::State;

use crate::error::AppResult;
use crate::persistence::settings as settings_repo;
use crate::persistence::Db;

#[tauri::command]
pub fn get_setting(key: String, db: State<'_, Db>) -> AppResult<Option<String>> {
    db.with(|c| settings_repo::get(c, &key))
}

#[tauri::command]
pub fn set_setting(key: String, value: String, db: State<'_, Db>) -> AppResult<()> {
    db.with(|c| settings_repo::set(c, &key, &value))
}
