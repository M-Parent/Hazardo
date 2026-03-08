use std::fs;
use std::sync::Mutex;
use rusqlite::{params, Connection};
use serde::Serialize;
use tauri::Manager;

struct DbState(Mutex<Connection>);

#[derive(Serialize)]
struct User {
    user_id: i64,
    user_name: String,
}

#[tauri::command]
fn has_device(db: tauri::State<DbState>) -> Result<bool, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM devices", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

#[tauri::command]
fn get_users(db: tauri::State<DbState>) -> Result<Vec<User>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT user_id, user_name FROM users ORDER BY user_id ASC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(User {
                user_id: row.get(0)?,
                user_name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut users = Vec::new();
    for r in rows {
        users.push(r.map_err(|e| e.to_string())?);
    }
    Ok(users)
}

#[tauri::command]
fn create_user(db: tauri::State<DbState>, user_name: String) -> Result<User, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    // attach to the first device
    let device_id: i64 = conn
        .query_row("SELECT device_id FROM devices ORDER BY device_id ASC LIMIT 1", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO users (user_name, user_device) VALUES (?1, ?2)",
        params![&user_name, device_id],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    Ok(User { user_id: id, user_name })
}

#[tauri::command]
fn create_device(
    db: tauri::State<DbState>,
    device_name: String,
    user_name: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO devices (device_name) VALUES (?1)",
        params![&device_name],
    )
    .map_err(|e| e.to_string())?;

    if let Some(u) = user_name {
        let device_id: i64 = conn
            .query_row(
                "SELECT device_id FROM devices WHERE device_name = ?1",
                params![&device_name],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO users (user_name, user_device) VALUES (?1, ?2)",
            params![u, device_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_local_data_dir()?;
            fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("hazardo.db");
            println!("Opening DB at: {:?}", db_path);

            let conn = Connection::open(&db_path)?;
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS devices (
                    device_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    device_name TEXT NOT NULL UNIQUE,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
                );
                CREATE TABLE IF NOT EXISTS users (
                    user_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_name TEXT NOT NULL,
                    user_device INTEGER,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY(user_device) REFERENCES devices(device_id)
                );"
            )?;

            app.manage(DbState(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![has_device, create_device, get_users, create_user])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
