use std::fs;
use std::sync::Mutex;
use std::io::{Write as IoWrite, Cursor};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::Manager;
use base64::Engine;

struct DbState(Mutex<Connection>);

// ─── Data Structs ────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct User {
    user_id: i64,
    user_name: String,
}

#[derive(Serialize, Deserialize)]
struct Device {
    device_id: i64,
    device_name: String,
}

#[derive(Serialize, Deserialize)]
struct Category {
    category_id: i64,
    user_id: i64,
    category_name: String,
    category_icon: String,
    is_default: i64,
}

#[derive(Serialize, Deserialize)]
struct Item {
    item_id: i64,
    category_id: i64,
    user_id: i64,
    item_name: String,
    time_pref: String,
    vibe_pref: String,
    location: Option<String>,
    description: Option<String>,
    image_path: Option<String>,
    notes: Option<String>,
    is_picked: i64,
}

#[derive(Serialize, Deserialize)]
struct Pick {
    pick_id: i64,
    user_id: i64,
    item_id: i64,
    category_id: i64,
    item_name: String,
    category_name: String,
    category_icon: String,
    pick_date: String,
    time_pref: String,
    vibe_pref: String,
    ai_recommendation: Option<String>,
    notes: Option<String>,
    image_path: Option<String>,
    location: Option<String>,
    created_at: String,
}

#[derive(Serialize, Deserialize)]
struct AppSetting {
    setting_key: String,
    setting_value: String,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    message_id: i64,
    user_id: i64,
    role: String,
    content: String,
    created_at: String,
}

// ─── Device Commands ─────────────────────────────────────────

#[tauri::command]
fn has_device(db: tauri::State<DbState>) -> Result<bool, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM devices", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

#[tauri::command]
fn get_device(db: tauri::State<DbState>) -> Result<Option<Device>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT device_id, device_name FROM devices ORDER BY device_id ASC LIMIT 1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query_map([], |row| {
            Ok(Device {
                device_id: row.get(0)?,
                device_name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    match rows.next() {
        Some(Ok(d)) => Ok(Some(d)),
        _ => Ok(None),
    }
}

#[tauri::command]
fn create_device(
    db: tauri::State<DbState>,
    device_name: String,
    user_name: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO devices (device_name) VALUES (?1)",
        params![&device_name],
    )
    .map_err(|e| e.to_string())?;

    let device_id: i64 = conn
        .query_row(
            "SELECT device_id FROM devices WHERE device_name = ?1",
            params![&device_name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if let Some(u) = user_name {
        if !u.trim().is_empty() {
            conn.execute(
                "INSERT INTO users (user_name, user_device) VALUES (?1, ?2)",
                params![u.trim(), device_id],
            )
            .map_err(|e| e.to_string())?;

            let user_id = conn.last_insert_rowid();
            init_default_categories_for_user(&conn, user_id)?;
        }
    }

    Ok(())
}

#[tauri::command]
fn update_device_name(db: tauri::State<DbState>, device_name: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE devices SET device_name = ?1, updated_at = CURRENT_TIMESTAMP WHERE device_id = (SELECT MIN(device_id) FROM devices)",
        params![&device_name],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── User Commands ───────────────────────────────────────────

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
    let device_id: i64 = conn
        .query_row("SELECT device_id FROM devices ORDER BY device_id ASC LIMIT 1", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO users (user_name, user_device) VALUES (?1, ?2)",
        params![&user_name, device_id],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();

    init_default_categories_for_user(&conn, id)?;

    Ok(User { user_id: id, user_name })
}

#[tauri::command]
fn update_user(db: tauri::State<DbState>, user_id: i64, user_name: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE users SET user_name = ?1, updated_at = CURRENT_TIMESTAMP WHERE user_id = ?2",
        params![&user_name, user_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_user(db: tauri::State<DbState>, user_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM chat_messages WHERE user_id = ?1", params![user_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM picks WHERE user_id = ?1", params![user_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM items WHERE user_id = ?1", params![user_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM categories WHERE user_id = ?1", params![user_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM settings WHERE user_id = ?1", params![user_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM users WHERE user_id = ?1", params![user_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Category Commands ───────────────────────────────────────

fn init_default_categories_for_user(conn: &Connection, user_id: i64) -> Result<(), String> {
    let defaults = vec![
        ("Activity", "activity"),
        ("Gift", "gift"),
        ("Restaurant", "restaurant"),
        ("Travel", "travel"),
        ("Valentine Day", "valentine"),
    ];
    for (name, icon) in defaults {
        conn.execute(
            "INSERT INTO categories (user_id, category_name, category_icon, is_default) VALUES (?1, ?2, ?3, 1)",
            params![user_id, name, icon],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_categories(db: tauri::State<DbState>, user_id: i64) -> Result<Vec<Category>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT category_id, user_id, category_name, category_icon, is_default FROM categories WHERE user_id = ?1 AND deleted_at IS NULL ORDER BY is_default DESC, category_name ASC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![user_id], |row| {
            Ok(Category {
                category_id: row.get(0)?,
                user_id: row.get(1)?,
                category_name: row.get(2)?,
                category_icon: row.get(3)?,
                is_default: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut cats = Vec::new();
    for r in rows {
        cats.push(r.map_err(|e| e.to_string())?);
    }
    Ok(cats)
}

#[tauri::command]
fn create_category(db: tauri::State<DbState>, user_id: i64, category_name: String, category_icon: String) -> Result<Category, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO categories (user_id, category_name, category_icon, is_default) VALUES (?1, ?2, ?3, 0)",
        params![user_id, &category_name, &category_icon],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    Ok(Category {
        category_id: id,
        user_id,
        category_name,
        category_icon,
        is_default: 0,
    })
}

#[tauri::command]
fn update_category(db: tauri::State<DbState>, category_id: i64, category_name: String, category_icon: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE categories SET category_name = ?1, category_icon = ?2, updated_at = CURRENT_TIMESTAMP WHERE category_id = ?3",
        params![&category_name, &category_icon, category_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_category(db: tauri::State<DbState>, category_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE categories SET deleted_at = CURRENT_TIMESTAMP WHERE category_id = ?1",
        params![category_id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE items SET deleted_at = CURRENT_TIMESTAMP WHERE category_id = ?1 AND deleted_at IS NULL",
        params![category_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn restore_category(db: tauri::State<DbState>, category_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE categories SET deleted_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE category_id = ?1",
        params![category_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_deleted_categories(db: tauri::State<DbState>, user_id: i64) -> Result<Vec<Category>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT category_id, user_id, category_name, category_icon, is_default FROM categories WHERE user_id = ?1 AND deleted_at IS NOT NULL ORDER BY category_name ASC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![user_id], |row| {
            Ok(Category {
                category_id: row.get(0)?,
                user_id: row.get(1)?,
                category_name: row.get(2)?,
                category_icon: row.get(3)?,
                is_default: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut cats = Vec::new();
    for r in rows {
        cats.push(r.map_err(|e| e.to_string())?);
    }
    Ok(cats)
}

// ─── Item Commands ───────────────────────────────────────────

#[tauri::command]
fn get_items(db: tauri::State<DbState>, user_id: i64, category_id: Option<i64>) -> Result<Vec<Item>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let sql = if category_id.is_some() {
        "SELECT item_id, category_id, user_id, item_name, time_pref, vibe_pref, location, description, image_path, notes, is_picked FROM items WHERE user_id = ?1 AND category_id = ?2 AND deleted_at IS NULL ORDER BY item_name ASC"
    } else {
        "SELECT item_id, category_id, user_id, item_name, time_pref, vibe_pref, location, description, image_path, notes, is_picked FROM items WHERE user_id = ?1 AND deleted_at IS NULL ORDER BY item_name ASC"
    };
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let map_row = |row: &rusqlite::Row| -> rusqlite::Result<Item> {
        Ok(Item {
            item_id: row.get(0)?,
            category_id: row.get(1)?,
            user_id: row.get(2)?,
            item_name: row.get(3)?,
            time_pref: row.get(4)?,
            vibe_pref: row.get(5)?,
            location: row.get(6)?,
            description: row.get(7)?,
            image_path: row.get(8)?,
            notes: row.get(9)?,
            is_picked: row.get(10)?,
        })
    };
    let rows = if let Some(cid) = category_id {
        stmt.query_map(params![user_id, cid], map_row)
    } else {
        stmt.query_map(params![user_id], map_row)
    }.map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for r in rows {
        items.push(r.map_err(|e| e.to_string())?);
    }
    Ok(items)
}

#[tauri::command]
fn create_item(
    db: tauri::State<DbState>,
    user_id: i64,
    category_id: i64,
    item_name: String,
    time_pref: String,
    vibe_pref: String,
    location: Option<String>,
    description: Option<String>,
    notes: Option<String>,
) -> Result<Item, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO items (category_id, user_id, item_name, time_pref, vibe_pref, location, description, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![category_id, user_id, &item_name, &time_pref, &vibe_pref, &location, &description, &notes],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    Ok(Item {
        item_id: id,
        category_id,
        user_id,
        item_name,
        time_pref,
        vibe_pref,
        location,
        description,
        image_path: None,
        notes,
        is_picked: 0,
    })
}

#[tauri::command]
fn update_item(
    db: tauri::State<DbState>,
    item_id: i64,
    item_name: String,
    category_id: i64,
    time_pref: String,
    vibe_pref: String,
    location: Option<String>,
    description: Option<String>,
    notes: Option<String>,
    is_picked: Option<i64>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE items SET item_name = ?1, category_id = ?2, time_pref = ?3, vibe_pref = ?4, location = ?5, description = ?6, notes = ?7, is_picked = COALESCE(?8, is_picked), updated_at = CURRENT_TIMESTAMP WHERE item_id = ?9",
        params![&item_name, category_id, &time_pref, &vibe_pref, &location, &description, &notes, &is_picked, item_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_item(db: tauri::State<DbState>, item_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE items SET deleted_at = CURRENT_TIMESTAMP WHERE item_id = ?1",
        params![item_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn restore_item(db: tauri::State<DbState>, item_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE items SET deleted_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE item_id = ?1",
        params![item_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_deleted_items(db: tauri::State<DbState>, user_id: i64) -> Result<Vec<Item>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT item_id, category_id, user_id, item_name, time_pref, vibe_pref, location, description, image_path, notes, is_picked FROM items WHERE user_id = ?1 AND deleted_at IS NOT NULL ORDER BY item_name ASC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![user_id], |row| {
            Ok(Item {
                item_id: row.get(0)?,
                category_id: row.get(1)?,
                user_id: row.get(2)?,
                item_name: row.get(3)?,
                time_pref: row.get(4)?,
                vibe_pref: row.get(5)?,
                location: row.get(6)?,
                description: row.get(7)?,
                image_path: row.get(8)?,
                notes: row.get(9)?,
                is_picked: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for r in rows {
        items.push(r.map_err(|e| e.to_string())?);
    }
    Ok(items)
}

#[tauri::command]
fn permanent_delete_item(db: tauri::State<DbState>, item_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM picks WHERE item_id = ?1", params![item_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM items WHERE item_id = ?1", params![item_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn permanent_delete_category(db: tauri::State<DbState>, category_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM picks WHERE category_id = ?1", params![category_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM items WHERE category_id = ?1", params![category_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM categories WHERE category_id = ?1", params![category_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Pick / Roll Commands ────────────────────────────────────

#[tauri::command]
fn roll_dice(
    db: tauri::State<DbState>,
    user_id: i64,
    time_pref: String,
    vibe_pref: String,
    category_id: Option<i64>,
) -> Result<Option<Pick>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let cid = category_id.unwrap_or(0);
    let has_cat = category_id.is_some();

    let cat_filter = if has_cat { "AND i.category_id = ?4" } else { "" };

    // Priority 1: Exact match on non-Mixed fields, not picked (items with exactly the chosen pref)
    // Priority 2: Flexible match (exact OR item is Mixed), not picked  
    // Priority 3: Any not-picked item (in category if selected)
    // Priority 4: Any item including already picked
    let queries: Vec<String> = vec![
        // P1: Strict - item.time_pref = chosen (not Mixed items), same for vibe
        format!("SELECT i.item_id, i.category_id, i.item_name, c.category_name, c.category_icon, i.time_pref, i.vibe_pref
         FROM items i JOIN categories c ON i.category_id = c.category_id
         WHERE i.user_id = ?1 AND i.deleted_at IS NULL AND c.deleted_at IS NULL AND i.is_picked = 0
         {cat}
         AND (?2 = 'Mixed' OR i.time_pref = ?2)
         AND (?3 = 'Mixed' OR i.vibe_pref = ?3)
         ORDER BY RANDOM() LIMIT 1", cat = cat_filter),
        // P2: Flexible - item pref matches OR item pref is Mixed
        format!("SELECT i.item_id, i.category_id, i.item_name, c.category_name, c.category_icon, i.time_pref, i.vibe_pref
         FROM items i JOIN categories c ON i.category_id = c.category_id
         WHERE i.user_id = ?1 AND i.deleted_at IS NULL AND c.deleted_at IS NULL AND i.is_picked = 0
         {cat}
         AND (?2 = 'Mixed' OR i.time_pref = ?2 OR i.time_pref = 'Mixed')
         AND (?3 = 'Mixed' OR i.vibe_pref = ?3 OR i.vibe_pref = 'Mixed')
         ORDER BY RANDOM() LIMIT 1", cat = cat_filter),
        // P3: Any not-picked in category
        format!("SELECT i.item_id, i.category_id, i.item_name, c.category_name, c.category_icon, i.time_pref, i.vibe_pref
         FROM items i JOIN categories c ON i.category_id = c.category_id
         WHERE i.user_id = ?1 AND i.deleted_at IS NULL AND c.deleted_at IS NULL AND i.is_picked = 0
         {cat}
         ORDER BY RANDOM() LIMIT 1", cat = cat_filter),
        // P4: Any item (including picked)
        format!("SELECT i.item_id, i.category_id, i.item_name, c.category_name, c.category_icon, i.time_pref, i.vibe_pref
         FROM items i JOIN categories c ON i.category_id = c.category_id
         WHERE i.user_id = ?1 AND i.deleted_at IS NULL AND c.deleted_at IS NULL
         {cat}
         ORDER BY RANDOM() LIMIT 1", cat = cat_filter),
    ];

    for sql in &queries {
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let result = if has_cat {
            stmt.query_row(params![user_id, &time_pref, &vibe_pref, cid], |row| {
                Ok(Pick {
                    pick_id: 0, user_id,
                    item_id: row.get(0)?, category_id: row.get(1)?,
                    item_name: row.get(2)?, category_name: row.get(3)?,
                    category_icon: row.get(4)?, pick_date: String::new(),
                    time_pref: row.get(5)?, vibe_pref: row.get(6)?,
                    ai_recommendation: None, notes: None, image_path: None, location: None, created_at: String::new(),
                })
            })
        } else {
            stmt.query_row(params![user_id, &time_pref, &vibe_pref], |row| {
                Ok(Pick {
                    pick_id: 0, user_id,
                    item_id: row.get(0)?, category_id: row.get(1)?,
                    item_name: row.get(2)?, category_name: row.get(3)?,
                    category_icon: row.get(4)?, pick_date: String::new(),
                    time_pref: row.get(5)?, vibe_pref: row.get(6)?,
                    ai_recommendation: None, notes: None, image_path: None, location: None, created_at: String::new(),
                })
            })
        };
        match result {
            Ok(p) => return Ok(Some(p)),
            Err(rusqlite::Error::QueryReturnedNoRows) => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(None)
}

#[tauri::command]
fn create_pick(
    db: tauri::State<DbState>,
    user_id: i64,
    item_id: i64,
    category_id: i64,
    pick_date: String,
    time_pref: String,
    vibe_pref: String,
    ai_recommendation: Option<String>,
    location: Option<String>,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO picks (user_id, item_id, category_id, pick_date, time_pref, vibe_pref, ai_recommendation, location) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![user_id, item_id, category_id, &pick_date, &time_pref, &vibe_pref, &ai_recommendation, &location],
    )
    .map_err(|e| e.to_string())?;
    // Mark item as picked
    conn.execute("UPDATE items SET is_picked = 1 WHERE item_id = ?1", params![item_id])
        .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
fn get_picks(db: tauri::State<DbState>, user_id: i64, category_id: Option<i64>) -> Result<Vec<Pick>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let sql = if category_id.is_some() {
        "SELECT p.pick_id, p.user_id, p.item_id, p.category_id, i.item_name, c.category_name, c.category_icon, p.pick_date, p.time_pref, p.vibe_pref, p.ai_recommendation, p.notes, p.image_path, p.location, p.created_at
         FROM picks p
         JOIN items i ON p.item_id = i.item_id
         JOIN categories c ON p.category_id = c.category_id
         WHERE p.user_id = ?1 AND p.category_id = ?2 AND p.deleted_at IS NULL
         ORDER BY p.created_at DESC"
    } else {
        "SELECT p.pick_id, p.user_id, p.item_id, p.category_id, i.item_name, c.category_name, c.category_icon, p.pick_date, p.time_pref, p.vibe_pref, p.ai_recommendation, p.notes, p.image_path, p.location, p.created_at
         FROM picks p
         JOIN items i ON p.item_id = i.item_id
         JOIN categories c ON p.category_id = c.category_id
         WHERE p.user_id = ?1 AND p.deleted_at IS NULL
         ORDER BY p.created_at DESC"
    };
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let map_row = |row: &rusqlite::Row| -> rusqlite::Result<Pick> {
        Ok(Pick {
            pick_id: row.get(0)?,
            user_id: row.get(1)?,
            item_id: row.get(2)?,
            category_id: row.get(3)?,
            item_name: row.get(4)?,
            category_name: row.get(5)?,
            category_icon: row.get(6)?,
            pick_date: row.get(7)?,
            time_pref: row.get(8)?,
            vibe_pref: row.get(9)?,
            ai_recommendation: row.get(10)?,
            notes: row.get(11)?,
            image_path: row.get(12)?,
            location: row.get(13)?,
            created_at: row.get(14)?,
        })
    };
    let rows = if let Some(cid) = category_id {
        stmt.query_map(params![user_id, cid], map_row)
    } else {
        stmt.query_map(params![user_id], map_row)
    }.map_err(|e| e.to_string())?;
    let mut picks = Vec::new();
    for r in rows {
        picks.push(r.map_err(|e| e.to_string())?);
    }
    Ok(picks)
}

#[tauri::command]
fn delete_pick(db: tauri::State<DbState>, pick_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE picks SET deleted_at = CURRENT_TIMESTAMP WHERE pick_id = ?1", params![pick_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_deleted_picks(db: tauri::State<DbState>, user_id: i64) -> Result<Vec<Pick>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT p.pick_id, p.user_id, p.item_id, p.category_id, i.item_name, c.category_name, c.category_icon, p.pick_date, p.time_pref, p.vibe_pref, p.ai_recommendation, p.notes, p.image_path, p.location, p.created_at
         FROM picks p
         JOIN items i ON p.item_id = i.item_id
         JOIN categories c ON p.category_id = c.category_id
         WHERE p.user_id = ?1 AND p.deleted_at IS NOT NULL
         ORDER BY p.deleted_at DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![user_id], |row| {
        Ok(Pick {
            pick_id: row.get(0)?, user_id: row.get(1)?, item_id: row.get(2)?,
            category_id: row.get(3)?, item_name: row.get(4)?, category_name: row.get(5)?,
            category_icon: row.get(6)?, pick_date: row.get(7)?, time_pref: row.get(8)?,
            vibe_pref: row.get(9)?, ai_recommendation: row.get(10)?, notes: row.get(11)?,
            image_path: row.get(12)?, location: row.get(13)?, created_at: row.get(14)?,
        })
    }).map_err(|e| e.to_string())?;
    let mut picks = Vec::new();
    for r in rows { picks.push(r.map_err(|e| e.to_string())?); }
    Ok(picks)
}

#[tauri::command]
fn restore_pick(db: tauri::State<DbState>, pick_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE picks SET deleted_at = NULL WHERE pick_id = ?1", params![pick_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn permanent_delete_pick(db: tauri::State<DbState>, pick_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM picks WHERE pick_id = ?1", params![pick_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn export_data(db: tauri::State<DbState>, user_ids: Vec<i64>, category_ids: Vec<i64>) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut export: Vec<serde_json::Value> = Vec::new();
    for uid in &user_ids {
        let uname: String = conn.query_row("SELECT user_name FROM users WHERE user_id = ?1", params![uid], |r| r.get(0)).map_err(|e| e.to_string())?;
        let mut cats_out: Vec<serde_json::Value> = Vec::new();
        let mut cat_stmt = conn.prepare("SELECT category_id, category_name, category_icon FROM categories WHERE user_id = ?1 AND deleted_at IS NULL ORDER BY category_name").map_err(|e| e.to_string())?;
        let cat_rows = cat_stmt.query_map(params![uid], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?))).map_err(|e| e.to_string())?;
        for cr in cat_rows {
            let (cid, cname, cicon) = cr.map_err(|e| e.to_string())?;
            if !category_ids.is_empty() && !category_ids.contains(&cid) { continue; }
            let mut items_out: Vec<serde_json::Value> = Vec::new();
            let mut item_stmt = conn.prepare("SELECT item_name, time_pref, vibe_pref, notes FROM items WHERE category_id = ?1 AND deleted_at IS NULL ORDER BY item_name").map_err(|e| e.to_string())?;
            let item_rows = item_stmt.query_map(params![cid], |r| Ok(serde_json::json!({
                "name": r.get::<_,String>(0)?,
                "time_pref": r.get::<_,String>(1)?,
                "vibe_pref": r.get::<_,String>(2)?,
                "notes": r.get::<_,Option<String>>(3)?
            }))).map_err(|e| e.to_string())?;
            for ir in item_rows { items_out.push(ir.map_err(|e| e.to_string())?); }
            cats_out.push(serde_json::json!({ "name": cname, "icon": cicon, "items": items_out }));
        }
        // Include picks with image data
        let mut picks_out: Vec<serde_json::Value> = Vec::new();
        let mut pick_stmt = conn.prepare(
            "SELECT p.pick_id, i.item_name, c.category_name, c.category_icon, p.pick_date, p.time_pref, p.vibe_pref, p.notes, p.image_path, p.ai_recommendation, p.location
             FROM picks p JOIN items i ON p.item_id = i.item_id JOIN categories c ON p.category_id = c.category_id
             WHERE p.user_id = ?1 AND p.deleted_at IS NULL ORDER BY p.pick_date DESC"
        ).map_err(|e| e.to_string())?;
        let pick_rows = pick_stmt.query_map(params![uid], |r| Ok(serde_json::json!({
            "pick_id": r.get::<_,i64>(0)?,
            "item_name": r.get::<_,String>(1)?,
            "category_name": r.get::<_,String>(2)?,
            "category_icon": r.get::<_,String>(3)?,
            "pick_date": r.get::<_,String>(4)?,
            "time_pref": r.get::<_,Option<String>>(5)?,
            "vibe_pref": r.get::<_,Option<String>>(6)?,
            "notes": r.get::<_,Option<String>>(7)?,
            "image_path": r.get::<_,Option<String>>(8)?,
            "ai_recommendation": r.get::<_,Option<String>>(9)?,
            "location": r.get::<_,Option<String>>(10)?
        }))).map_err(|e| e.to_string())?;
        for pr in pick_rows { picks_out.push(pr.map_err(|e| e.to_string())?); }
        export.push(serde_json::json!({ "user": uname, "categories": cats_out, "picks": picks_out }));
    }
    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_data(db: tauri::State<DbState>, user_id: i64, data: String) -> Result<String, String> {
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&data).map_err(|e| format!("Invalid JSON: {}", e))?;
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut count = 0u32;
    for user_data in &parsed {
        let cats = user_data.get("categories").and_then(|c| c.as_array()).cloned().unwrap_or_default();
        for cat in &cats {
            let cat_name = cat.get("name").and_then(|n| n.as_str()).unwrap_or("Imported");
            let cat_icon = cat.get("icon").and_then(|i| i.as_str()).unwrap_or("misc");
            // Check if category already exists for this user
            let existing_cid: Option<i64> = conn.query_row(
                "SELECT category_id FROM categories WHERE user_id = ?1 AND category_name = ?2 AND deleted_at IS NULL",
                params![user_id, cat_name],
                |row| row.get(0),
            ).ok();
            let cid = if let Some(id) = existing_cid {
                id
            } else {
                conn.execute("INSERT INTO categories (user_id, category_name, category_icon, is_default) VALUES (?1, ?2, ?3, 0)", params![user_id, cat_name, cat_icon]).map_err(|e| e.to_string())?;
                conn.last_insert_rowid()
            };
            let items = cat.get("items").and_then(|i| i.as_array()).cloned().unwrap_or_default();
            for item in &items {
                let iname = item.get("name").and_then(|n| n.as_str()).unwrap_or("Unnamed");
                let tp = item.get("time_pref").and_then(|n| n.as_str()).unwrap_or("Mixed");
                let vp = item.get("vibe_pref").and_then(|n| n.as_str()).unwrap_or("Mixed");
                let notes = item.get("notes").and_then(|n| n.as_str());
                conn.execute("INSERT INTO items (category_id, user_id, item_name, time_pref, vibe_pref, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![cid, user_id, iname, tp, vp, notes]).map_err(|e| e.to_string())?;
                count += 1;
            }
        }
    }
    Ok(format!("Imported {} items successfully", count))
}

#[tauri::command]
fn update_pick(
    db: tauri::State<DbState>,
    pick_id: i64,
    notes: Option<String>,
    image_path: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE picks SET notes = ?1, image_path = ?2 WHERE pick_id = ?3",
        params![&notes, &image_path, pick_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Export File Commands ─────────────────────────────────────

#[tauri::command]
fn read_file_bytes(path: String) -> Result<String, String> {
    use base64::Engine;
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

#[tauri::command]
fn save_export_file(app: tauri::AppHandle, content: String, filename: String, target: Option<String>) -> Result<String, String> {
    let dir = match target.as_deref() {
        Some("downloads") => {
            let dl = std::path::PathBuf::from("/storage/emulated/0/Download");
            if dl.exists() { dl } else { app.path().app_cache_dir().map_err(|e| e.to_string())? }
        },
        _ => app.path().app_cache_dir().map_err(|e| e.to_string())?,
    };
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(&filename);
    fs::write(&path, content.as_bytes()).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn export_zip(
    app: tauri::AppHandle,
    db: tauri::State<DbState>,
    user_ids: Vec<i64>,
    category_ids: Vec<i64>,
    target: Option<String>,
) -> Result<String, String> {
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut export: Vec<serde_json::Value> = Vec::new();
    let mut images: Vec<(String, Vec<u8>)> = Vec::new();

    for uid in &user_ids {
        let uname: String = conn.query_row(
            "SELECT user_name FROM users WHERE user_id = ?1", params![uid], |r| r.get(0)
        ).map_err(|e| e.to_string())?;

        let mut cats_out: Vec<serde_json::Value> = Vec::new();
        let mut cat_stmt = conn.prepare(
            "SELECT category_id, category_name, category_icon FROM categories WHERE user_id = ?1 AND deleted_at IS NULL ORDER BY category_name"
        ).map_err(|e| e.to_string())?;
        let cat_rows = cat_stmt.query_map(params![uid], |r| {
            Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?))
        }).map_err(|e| e.to_string())?;
        for cr in cat_rows {
            let (cid, cname, cicon) = cr.map_err(|e| e.to_string())?;
            if !category_ids.is_empty() && !category_ids.contains(&cid) { continue; }
            let mut items_out: Vec<serde_json::Value> = Vec::new();
            let mut item_stmt = conn.prepare(
                "SELECT item_name, time_pref, vibe_pref, notes FROM items WHERE category_id = ?1 AND deleted_at IS NULL ORDER BY item_name"
            ).map_err(|e| e.to_string())?;
            let item_rows = item_stmt.query_map(params![cid], |r| Ok(serde_json::json!({
                "name": r.get::<_,String>(0)?,
                "time_pref": r.get::<_,String>(1)?,
                "vibe_pref": r.get::<_,String>(2)?,
                "notes": r.get::<_,Option<String>>(3)?
            }))).map_err(|e| e.to_string())?;
            for ir in item_rows { items_out.push(ir.map_err(|e| e.to_string())?); }
            cats_out.push(serde_json::json!({ "name": cname, "icon": cicon, "items": items_out }));
        }

        // Picks with image extraction
        let mut picks_out: Vec<serde_json::Value> = Vec::new();
        let mut pick_stmt = conn.prepare(
            "SELECT p.pick_id, i.item_name, c.category_name, c.category_icon, p.pick_date, p.time_pref, p.vibe_pref, p.notes, p.image_path, p.ai_recommendation, p.location
             FROM picks p JOIN items i ON p.item_id = i.item_id JOIN categories c ON p.category_id = c.category_id
             WHERE p.user_id = ?1 AND p.deleted_at IS NULL ORDER BY p.pick_date DESC"
        ).map_err(|e| e.to_string())?;
        let pick_rows = pick_stmt.query_map(params![uid], |r| {
            Ok((
                r.get::<_,i64>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?,
                r.get::<_,String>(3)?, r.get::<_,String>(4)?, r.get::<_,Option<String>>(5)?,
                r.get::<_,Option<String>>(6)?, r.get::<_,Option<String>>(7)?, r.get::<_,Option<String>>(8)?,
                r.get::<_,Option<String>>(9)?, r.get::<_,Option<String>>(10)?,
            ))
        }).map_err(|e| e.to_string())?;
        for pr in pick_rows {
            let (pick_id, item_name, cat_name, cat_icon, pick_date, time_pref, vibe_pref, notes, image_path, ai_recommendation, location) =
                pr.map_err(|e| e.to_string())?;
            let mut image_file = serde_json::Value::Null;
            if let Some(ref img) = image_path {
                if let Some(comma_pos) = img.find(',') {
                    let header = &img[..comma_pos];
                    let b64_data = &img[comma_pos + 1..];
                    let ext = if header.contains("png") { "png" }
                        else if header.contains("gif") { "gif" }
                        else if header.contains("webp") { "webp" }
                        else { "jpg" };
                    let filename = format!("images/pick_{}.{}", pick_id, ext);
                    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(b64_data) {
                        images.push((filename.clone(), decoded));
                        image_file = serde_json::Value::String(filename);
                    }
                }
            }
            picks_out.push(serde_json::json!({
                "pick_id": pick_id, "item_name": item_name,
                "category_name": cat_name, "category_icon": cat_icon,
                "pick_date": pick_date, "time_pref": time_pref,
                "vibe_pref": vibe_pref, "notes": notes, "image_file": image_file,
                "ai_recommendation": ai_recommendation, "location": location
            }));
        }
        export.push(serde_json::json!({ "user": uname, "categories": cats_out, "picks": picks_out }));
    }

    let data_json = serde_json::to_string_pretty(&export).map_err(|e| e.to_string())?;

    let buf = Vec::new();
    let mut zip = ZipWriter::new(Cursor::new(buf));
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("data.json", options).map_err(|e| e.to_string())?;
    zip.write_all(data_json.as_bytes()).map_err(|e| e.to_string())?;
    for (fname, data) in &images {
        zip.start_file(fname.as_str(), options).map_err(|e| e.to_string())?;
        zip.write_all(data).map_err(|e| e.to_string())?;
    }
    let cursor = zip.finish().map_err(|e| e.to_string())?;
    let zip_bytes = cursor.into_inner();

    let dir = match target.as_deref() {
        Some("downloads") => {
            let dl = std::path::PathBuf::from("/storage/emulated/0/Download");
            if dl.exists() { dl } else { app.path().app_cache_dir().map_err(|e| e.to_string())? }
        },
        _ => app.path().app_cache_dir().map_err(|e| e.to_string())?,
    };
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let zip_filename = format!("hazardo_export_{}.zip", ts);
    let path = dir.join(&zip_filename);
    fs::write(&path, &zip_bytes).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

// ─── Settings Commands ───────────────────────────────────────

#[tauri::command]
fn get_all_settings(db: tauri::State<DbState>, user_id: i64) -> Result<Vec<AppSetting>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT setting_key, setting_value FROM settings WHERE user_id = ?1")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![user_id], |row| {
            Ok(AppSetting {
                setting_key: row.get(0)?,
                setting_value: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut settings = Vec::new();
    for r in rows {
        settings.push(r.map_err(|e| e.to_string())?);
    }
    Ok(settings)
}

#[tauri::command]
fn get_setting(db: tauri::State<DbState>, user_id: i64, key: String) -> Result<Option<String>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let result = conn.query_row(
        "SELECT setting_value FROM settings WHERE user_id = ?1 AND setting_key = ?2",
        params![user_id, &key],
        |row| row.get(0),
    );
    match result {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn set_setting(db: tauri::State<DbState>, user_id: i64, key: String, value: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (user_id, setting_key, setting_value) VALUES (?1, ?2, ?3) ON CONFLICT(user_id, setting_key) DO UPDATE SET setting_value = ?3",
        params![user_id, &key, &value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Chat Commands ───────────────────────────────────────────

#[tauri::command]
fn get_chat_messages(db: tauri::State<DbState>, user_id: i64) -> Result<Vec<ChatMessage>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT message_id, user_id, role, content, created_at FROM chat_messages WHERE user_id = ?1 ORDER BY created_at ASC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![user_id], |row| {
            Ok(ChatMessage {
                message_id: row.get(0)?,
                user_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut msgs = Vec::new();
    for r in rows {
        msgs.push(r.map_err(|e| e.to_string())?);
    }
    Ok(msgs)
}

#[tauri::command]
fn save_chat_message(db: tauri::State<DbState>, user_id: i64, role: String, content: String) -> Result<ChatMessage, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO chat_messages (user_id, role, content) VALUES (?1, ?2, ?3)",
        params![user_id, &role, &content],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let created: String = conn
        .query_row("SELECT created_at FROM chat_messages WHERE message_id = ?1", params![id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(ChatMessage {
        message_id: id,
        user_id,
        role,
        content,
        created_at: created,
    })
}

#[tauri::command]
fn clear_chat(db: tauri::State<DbState>, user_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM chat_messages WHERE user_id = ?1", params![user_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── App Entry ───────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            let data_dir = app.path().app_local_data_dir()?;
            fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("hazardo.db");
            println!("Opening DB at: {:?}", db_path);

            let conn = Connection::open(&db_path)?;

            conn.execute_batch("PRAGMA journal_mode=WAL;")?;
            conn.execute_batch("PRAGMA foreign_keys=ON;")?;

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
                );
                CREATE TABLE IF NOT EXISTS categories (
                    category_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id INTEGER NOT NULL,
                    category_name TEXT NOT NULL,
                    category_icon TEXT DEFAULT 'misc',
                    is_default INTEGER DEFAULT 0,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    deleted_at TEXT,
                    FOREIGN KEY(user_id) REFERENCES users(user_id)
                );
                CREATE TABLE IF NOT EXISTS items (
                    item_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    category_id INTEGER NOT NULL,
                    user_id INTEGER NOT NULL,
                    item_name TEXT NOT NULL,
                    time_pref TEXT DEFAULT 'Mixed',
                    vibe_pref TEXT DEFAULT 'Mixed',
                    location TEXT,
                    description TEXT,
                    image_path TEXT,
                    notes TEXT,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    deleted_at TEXT,
                    FOREIGN KEY(category_id) REFERENCES categories(category_id),
                    FOREIGN KEY(user_id) REFERENCES users(user_id)
                );
                CREATE TABLE IF NOT EXISTS picks (
                    pick_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id INTEGER NOT NULL,
                    item_id INTEGER NOT NULL,
                    category_id INTEGER NOT NULL,
                    pick_date TEXT NOT NULL,
                    time_pref TEXT,
                    vibe_pref TEXT,
                    ai_recommendation TEXT,
                    notes TEXT,
                    image_path TEXT,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY(user_id) REFERENCES users(user_id),
                    FOREIGN KEY(item_id) REFERENCES items(item_id),
                    FOREIGN KEY(category_id) REFERENCES categories(category_id)
                );
                CREATE TABLE IF NOT EXISTS settings (
                    setting_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id INTEGER,
                    setting_key TEXT NOT NULL,
                    setting_value TEXT,
                    UNIQUE(user_id, setting_key)
                );
                CREATE TABLE IF NOT EXISTS chat_messages (
                    message_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id INTEGER NOT NULL,
                    role TEXT NOT NULL,
                    content TEXT NOT NULL,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY(user_id) REFERENCES users(user_id)
                );"
            )?;

            app.manage(DbState(Mutex::new(conn)));

            // Migrations: add columns for existing databases (ignore errors if already present)
            {
                let db = app.state::<DbState>();
                let conn = db.0.lock().unwrap();
                let _ = conn.execute_batch("ALTER TABLE picks ADD COLUMN notes TEXT;");
                let _ = conn.execute_batch("ALTER TABLE picks ADD COLUMN image_path TEXT;");
                let _ = conn.execute_batch("ALTER TABLE picks ADD COLUMN deleted_at TEXT;");
                let _ = conn.execute_batch("ALTER TABLE items ADD COLUMN is_picked INTEGER DEFAULT 0;");
                let _ = conn.execute_batch("ALTER TABLE picks ADD COLUMN location TEXT;");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Device
            has_device, create_device, get_device, update_device_name,
            // User
            get_users, create_user, update_user, delete_user,
            // Category
            get_categories, create_category, update_category, delete_category,
            restore_category, get_deleted_categories, permanent_delete_category,
            // Item
            get_items, create_item, update_item, delete_item,
            restore_item, get_deleted_items, permanent_delete_item,
            // Pick / Roll
            roll_dice, create_pick, get_picks, delete_pick, update_pick,
            get_deleted_picks, restore_pick, permanent_delete_pick,
            // Export / Import
            export_data, import_data, save_export_file, export_zip, read_file_bytes,
            // Settings
            get_all_settings, get_setting, set_setting,
            // Chat
            get_chat_messages, save_chat_message, clear_chat,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
