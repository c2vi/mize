use std::sync::Mutex;

use chrono::{Timelike, Utc};
use clap::{Arg, Command};
use rusqlite::{params, Connection};
use serde_json::{json, Value};

use mize::{mize_err, mize_part, Mize, MizeError, MizePart, MizeResult};

use crate::cli::CliPart;
use crate::habitica::Habitica;

#[mize_part]
#[derive(Default)]
pub struct C2vi {
    mize: Mize,
    db: Mutex<Option<Connection>>,
}

impl MizePart for C2vi {
    fn opts(&self, mize: &mut Mize) {
        mize.new_opt("c2vi.local_storage_path");
    }
}

impl C2vi {
    fn with_db<F, R>(&self, f: F) -> MizeResult<R>
    where
        F: FnOnce(&Connection) -> MizeResult<R>,
    {
        let guard = self
            .db
            .lock()
            .map_err(|e| mize_err!("DB lock poisoned: {}", e))?;
        let conn = guard
            .as_ref()
            .ok_or_else(|| mize_err!("DB not initialized"))?;
        f(conn)
    }
}

pub fn c2vi(mize: &mut Mize) -> MizeResult<()> {
    let storage_path = mize.get_config("c2vi.local_storage_path")?.to_string();
    let db_path = std::path::PathBuf::from(&storage_path).join("data.db");
    let conn = Connection::open(&db_path)
        .map_err(|e| mize_err!("Failed to open C2vi database at {:?}: {}", db_path, e))?;

    setup_tables(&conn)?;

    let c2vi_part = C2vi {
        mize: mize.clone(),
        db: Mutex::new(Some(conn)),
    };
    mize.register_part(Box::new(c2vi_part))?;

    // Register CLI subcommands
    let mut cli = mize.get_part_native::<CliPart>("cli")?;
    register_subcommands(&mut cli);

    Ok(())
}

fn setup_tables(conn: &Connection) -> MizeResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habitica_dailies (
            id TEXT PRIMARY KEY,
            text TEXT NOT NULL,
            notes TEXT,
            priority TEXT,
            frequency TEXT
        )",
        [],
    )
    .map_err(|e| mize_err!("DB error creating habitica_dailies: {}", e))?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habitica_log (
            date TEXT PRIMARY KEY,
            data TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| mize_err!("DB error creating habitica_log: {}", e))?;
    Ok(())
}

fn register_subcommands(cli: &mut CliPart) {
    cli.subcommand(
        Command::new("bed").about("Command to run when c2vi goes to bed"),
        |_matches, mut mize| {
            save_habitica_dailies(&mut mize)?;
            save_habitica_logs(&mut mize)?;
            Ok(())
        },
    );

    cli.subcommand(
        Command::new("clearTodos")
            .about("Clear all Habitica todos")
            .alias("clt"),
        |_matches, mut mize| {
            let todos = {
                let mut hab = mize.get_part_native::<Habitica>("habitica")?;
                hab.get_tasks("todos")?
            };
            let mut hab = mize.get_part_native::<Habitica>("habitica")?;
            if let Some(arr) = todos.as_array() {
                for todo in arr {
                    if let Some(id) = todo["id"].as_str() {
                        hab.delete_task(id)?;
                    }
                }
            }
            Ok(())
        },
    );

    cli.subcommand(
        Command::new("listTodos")
            .about("List all Habitica todos")
            .alias("dut"),
        |_matches, mut mize| {
            let mut hab = mize.get_part_native::<Habitica>("habitica")?;
            let todos = hab.get_tasks("todos")?;
            if let Some(arr) = todos.as_array() {
                for todo in arr {
                    if let Some(text) = todo["text"].as_str() {
                        println!("{}", text);
                    }
                }
            }
            Ok(())
        },
    );

    cli.subcommand(
        Command::new("buyHealthPotion")
            .about("Buy a health potion")
            .alias("buyh"),
        |_matches, mut mize| {
            let mut hab = mize.get_part_native::<Habitica>("habitica")?;
            hab.api_request(
                reqwest::Method::POST,
                "user/buy-health-potion".to_string(),
                json!({}),
            )?;
            Ok(())
        },
    );

    cli.subcommand(
        Command::new("addActionItems")
            .about("Add action items from temp file")
            .alias("aal"),
        |_matches, mut mize| add_action_items(&mut mize),
    );

    cli.subcommand(
        Command::new("skip")
            .about("Skip a Habitica todo")
            .alias("sk")
            .arg(Arg::new("num").default_value("1")),
        |matches, mut mize| {
            let num: usize = matches
                .get_one::<String>("num")
                .unwrap()
                .parse()
                .unwrap_or(1);
            skip_habitica_todo(&mut mize, num)
        },
    );

    cli.subcommand(
        Command::new("dumpLog").about("Dump the complete Habitica log"),
        |_matches, mut mize| dump_habitica_logs(&mut mize),
    );

    cli.subcommand(
        Command::new("listDailies").about("List all dailies from local DB"),
        |_matches, mut mize| {
            let c2vi = mize.get_part_native::<C2vi>("c2vi")?;
            c2vi.with_db(|conn| {
                let mut stmt = conn
                    .prepare("SELECT id, text FROM habitica_dailies")
                    .map_err(|e| mize_err!("DB error: {}", e))?;
                let rows = stmt
                    .query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                        ))
                    })
                    .map_err(|e| mize_err!("DB error: {}", e))?;
                for row in rows {
                    let (id, text) = row.map_err(|e| mize_err!("DB error: {}", e))?;
                    println!("{}: {}", id, text);
                }
                Ok(())
            })
        },
    );

    cli.subcommand(
        Command::new("printTask")
            .about("Print a specific task from Habitica")
            .arg(Arg::new("id").required(true)),
        |matches, mut mize| {
            let id = matches.get_one::<String>("id").unwrap();
            let mut hab = mize.get_part_native::<Habitica>("habitica")?;
            let data = hab.api_request(
                reqwest::Method::GET,
                format!("tasks/{}", id),
                json!({}),
            )?;
            if let Some(history) = data.get("history") {
                println!("{}", serde_json::to_string_pretty(history).unwrap_or_default());
            } else {
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            }
            Ok(())
        },
    );
}

// ============= Business Logic =============

fn format_date(dt: chrono::DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d").to_string()
}

/// Get the target date, adjusted for "save for yesterday" logic (before noon = yesterday)
fn get_target_date() -> (String, bool) {
    let now = Utc::now();
    let save_for_yesterday = now.hour() < 12;
    let date = if save_for_yesterday {
        now - chrono::Duration::days(1)
    } else {
        now
    };
    if save_for_yesterday {
        println!(
            "Saving logs for yesterday because it is {} hours of the day",
            now.hour()
        );
    }
    (format_date(date), save_for_yesterday)
}

/// Load existing log entry from DB, or return a default empty one
fn get_or_create_log(conn: &Connection, date: &str) -> MizeResult<Value> {
    let mut stmt = conn
        .prepare("SELECT data FROM habitica_log WHERE date = ?1")
        .map_err(|e| mize_err!("DB error: {}", e))?;
    let result: Option<String> = stmt
        .query_row(params![date], |row| row.get(0))
        .ok();
    match result {
        Some(data_str) => {
            serde_json::from_str(&data_str).map_err(|e| mize_err!("JSON parse error: {}", e))
        }
        None => Ok(json!({
            "dailies_done": [],
            "dailies_skipped": [],
            "todos_done": [],
            "todos_skipped": []
        })),
    }
}

/// Upsert a log entry into the database
fn upsert_log(conn: &Connection, date: &str, data: &Value) -> MizeResult<()> {
    let data_str =
        serde_json::to_string(data).map_err(|e| mize_err!("JSON serialize error: {}", e))?;
    conn.execute(
        "INSERT INTO habitica_log (date, data) VALUES (?1, ?2)
         ON CONFLICT(date) DO UPDATE SET data=excluded.data",
        params![date, data_str],
    )
    .map_err(|e| mize_err!("DB error upserting log: {}", e))?;
    Ok(())
}

fn save_habitica_dailies(mize: &mut Mize) -> MizeResult<()> {
    println!("######### cli bed cmd #########");

    // Fetch dailies from Habitica
    let dailies = {
        let mut hab = mize.get_part_native::<Habitica>("habitica")?;
        hab.get_tasks("dailys")?
    };

    // Write to database
    let c2vi = mize.get_part_native::<C2vi>("c2vi")?;
    c2vi.with_db(|conn| {
        if let Some(arr) = dailies.as_array() {
            println!("Syncing {} dailies to database...", arr.len());
            for daily in arr {
                let id = daily["id"].as_str().unwrap_or("");
                let text = daily["text"].as_str().unwrap_or("");
                let notes = daily["notes"].as_str().unwrap_or("");
                let priority = daily["priority"].to_string();
                let frequency = daily["frequency"].as_str().unwrap_or("daily");

                conn.execute(
                    "INSERT INTO habitica_dailies (id, text, notes, priority, frequency)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(id) DO UPDATE SET
                        text=excluded.text,
                        notes=excluded.notes,
                        priority=excluded.priority,
                        frequency=excluded.frequency",
                    params![id, text, notes, priority, frequency],
                )
                .map_err(|e| mize_err!("DB error writing daily: {}", e))?;
            }
        }
        Ok(())
    })
}

fn save_habitica_logs(mize: &mut Mize) -> MizeResult<()> {
    let (date_str, save_for_yesterday) = get_target_date();

    // Fetch dailies and completed todos from Habitica
    let (dailies, completed_todos) = {
        let mut hab = mize.get_part_native::<Habitica>("habitica")?;
        let dailies = hab.get_tasks("dailys")?;
        let completed_todos = hab.get_tasks("completedTodos")?;
        (dailies, completed_todos)
    };

    let c2vi = mize.get_part_native::<C2vi>("c2vi")?;
    c2vi.with_db(|conn| {
        let mut data = get_or_create_log(conn, &date_str)?;

        // Process dailies
        if let Some(arr) = dailies.as_array() {
            for daily in arr {
                let is_due = if save_for_yesterday {
                    // Look at history for yesterday's status
                    daily["history"]
                        .as_array()
                        .and_then(|h| h.iter().max_by_key(|e| e["date"].as_i64().unwrap_or(0)))
                        .map(|e| e["isDue"].as_bool().unwrap_or(false))
                        .unwrap_or(false)
                } else {
                    daily["isDue"].as_bool().unwrap_or(false)
                };

                if !is_due {
                    continue;
                }

                let completed = if save_for_yesterday {
                    daily["history"]
                        .as_array()
                        .and_then(|h| h.iter().max_by_key(|e| e["date"].as_i64().unwrap_or(0)))
                        .map(|e| e["completed"].as_bool().unwrap_or(false))
                        .unwrap_or(false)
                } else {
                    daily["completed"].as_bool().unwrap_or(false)
                };

                let daily_id = daily["id"].as_str().unwrap_or("");
                let daily_text = daily["text"].as_str().unwrap_or("");

                if completed {
                    data["dailies_done"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!(daily_id));
                    println!("daily-done: {}", daily_text);
                } else {
                    data["dailies_skipped"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!(daily_id));
                    println!("daily-skipped: {}", daily_text);
                }
            }
        }

        // Process completed todos
        if let Some(arr) = completed_todos.as_array() {
            let today_str = format_date(Utc::now());
            for todo in arr {
                let date_completed_str = todo["dateCompleted"].as_str().unwrap_or("");

                // Parse the completion date (ISO 8601 format from Habitica)
                let date_completed = chrono::DateTime::parse_from_rfc3339(date_completed_str)
                    .map(|d| format_date(d.with_timezone(&Utc)))
                    .unwrap_or_default();

                // Check if already in the log for this date
                if save_for_yesterday {
                    let existing_todos = data["todos_done"].as_array().unwrap();
                    let todo_id = todo["id"].as_str().unwrap_or("");
                    if existing_todos.iter().any(|t| t["id"].as_str() == Some(todo_id)) {
                        continue;
                    }
                }

                let todo_entry = json!({
                    "id": todo["id"],
                    "text": todo["text"],
                    "notes": todo.get("notes").unwrap_or(&json!("")),
                    "checklist": todo.get("checklist").unwrap_or(&json!([])),
                    "tags": todo.get("tags").unwrap_or(&json!([])),
                });

                // Save if completed on the target date
                if date_completed == date_str {
                    println!("todo-done: {}", todo["text"].as_str().unwrap_or(""));
                    data["todos_done"]
                        .as_array_mut()
                        .unwrap()
                        .push(todo_entry.clone());
                }

                // If saving for yesterday but todo was completed today, also record it
                if save_for_yesterday && date_completed == today_str {
                    println!("todo-done: {}", todo["text"].as_str().unwrap_or(""));
                    data["todos_done"]
                        .as_array_mut()
                        .unwrap()
                        .push(todo_entry);
                }
            }
        }

        upsert_log(conn, &date_str, &data)?;
        Ok(())
    })
}

fn skip_habitica_todo(mize: &mut Mize, num: usize) -> MizeResult<()> {
    let (date_str, _) = get_target_date();

    // Fetch todos from Habitica
    let todos = {
        let mut hab = mize.get_part_native::<Habitica>("habitica")?;
        hab.get_tasks("todos")?
    };

    let arr = todos.as_array().ok_or_else(|| mize_err!("Expected array of todos"))?;

    // Delete and log each skipped todo
    for i in 0..num {
        let todo = match arr.get(i) {
            Some(t) => t,
            None => continue,
        };

        let todo_id = todo["id"].as_str().unwrap_or("");

        // Delete the todo and score the skip habit
        {
            let mut hab = mize.get_part_native::<Habitica>("habitica")?;
            hab.delete_task(todo_id)?;
        }
        increment_habit(mize)?;

        // Record in log
        let c2vi = mize.get_part_native::<C2vi>("c2vi")?;
        c2vi.with_db(|conn| {
            let mut data = get_or_create_log(conn, &date_str)?;
            data["todos_skipped"].as_array_mut().unwrap().push(json!({
                "id": todo["id"],
                "text": todo["text"],
                "notes": todo.get("notes").unwrap_or(&json!("")),
                "checklist": todo.get("checklist").unwrap_or(&json!([])),
                "tags": todo.get("tags").unwrap_or(&json!([])),
            }));
            upsert_log(conn, &date_str, &data)?;
            Ok(())
        })?;
    }

    Ok(())
}

fn increment_habit(mize: &mut Mize) -> MizeResult<()> {
    let mut hab = mize.get_part_native::<Habitica>("habitica")?;
    let habits = hab.get_tasks("habits")?;
    if let Some(arr) = habits.as_array() {
        for habit in arr {
            if habit["text"].as_str() == Some("skipped a planned task") {
                let id = habit["id"].as_str().unwrap_or("");
                hab.api_request(
                    reqwest::Method::POST,
                    format!("tasks/{}/score/down", id),
                    json!({}),
                )?;
                return Ok(());
            }
        }
    }
    Ok(())
}

fn add_action_items(mize: &mut Mize) -> MizeResult<()> {
    let initial_content = "\
gu 10min TIMER
uw 15min FROG
b coffee/tee/kakau
uw FROG
b breakfast, brush teeth, lüften
uw FROG

lunch

uw 15min TIMER of next days FROG
fw
dt 30min TIMER

fun 20:00 (smth that can be stopped easily)
se 21:40
read
bed 22:10";

    let tmp_file = std::env::temp_dir().join(format!(
        "habitica-tasks-{}.md",
        Utc::now().timestamp()
    ));
    std::fs::write(&tmp_file, initial_content)
        .map_err(|e| mize_err!("Failed to write temp file: {}", e))?;

    // Open in nvim for editing
    std::process::Command::new("nvim")
        .arg(&tmp_file)
        .status()
        .map_err(|e| mize_err!("Failed to run nvim: {}", e))?;

    // Read back and parse
    let list_str =
        std::fs::read_to_string(&tmp_file).map_err(|e| mize_err!("Failed to read temp file: {}", e))?;
    let _ = std::fs::remove_file(&tmp_file);

    let tasks: Vec<&str> = list_str
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    // Add tasks to Habitica
    let mut task_ids = Vec::new();
    {
        let mut hab = mize.get_part_native::<Habitica>("habitica")?;
        for todo_text in &tasks {
            let data = hab.api_request(
                reqwest::Method::POST,
                "tasks/user".to_string(),
                json!({ "text": todo_text, "type": "todo" }),
            )?;
            let id = data["id"].as_str().unwrap_or("").to_string();
            println!("Added to-do: '{}' with id '{}'", todo_text, id);
            task_ids.push(id);
        }

        // Move all tasks to bottom
        for task_id in &task_ids {
            hab.api_request(
                reqwest::Method::POST,
                format!("tasks/{}/move/to/-1", task_id),
                json!({}),
            )?;
        }
    }

    Ok(())
}

fn dump_habitica_logs(mize: &mut Mize) -> MizeResult<()> {
    let c2vi = mize.get_part_native::<C2vi>("c2vi")?;
    c2vi.with_db(|conn| {
        let mut stmt = conn
            .prepare("SELECT date, data FROM habitica_log ORDER BY date")
            .map_err(|e| mize_err!("DB error: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                ))
            })
            .map_err(|e| mize_err!("DB error: {}", e))?;
        for row in rows {
            let (date, data_str) = row.map_err(|e| mize_err!("DB error: {}", e))?;
            let data: Value =
                serde_json::from_str(&data_str).unwrap_or(json!({"parse_error": data_str}));
            println!("{}: {}", date, serde_json::to_string_pretty(&data).unwrap_or_default());
        }
        Ok(())
    })
}
