use chrono::{Datelike, Timelike, Utc};
use clap::{Arg, Command};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command as StdCommand;

// Assuming PPC and other modules are structured like this.
// This might need adjustment based on the actual definitions in other files.
use crate::habitica::Habitica; // Assuming this exists

pub struct C2vi {
    pub db: Connection,
}

impl C2vi {
    pub fn new(local_storage_path: &str) -> Result<Self> {
        let db_path = PathBuf::from(local_storage_path).join("data.db");
        let conn = Connection::open(db_path)?;
        Ok(C2vi { db: conn })
    }
}

pub fn extend_cli_c2vi(mut cli: Command) -> Command {
    cli.subcommand(
        Command::new("bed")
            .about("Command to run when c2vi goes to bed")
            .action(async || {
                // Placeholder for actual PPC context
                // await save_habitica_dailies(ppc);
                // await save_habitica_logs(ppc);
            }),
    )
    .subcommand(
        Command::new("clearTodos")
            .about("Clear all todos in Habitica")
            .alias("clt")
            .action(async || {
                // Placeholder
            }),
    )
    .subcommand(
        Command::new("listTodos")
            .about("List all todos in Habitica")
            .alias("dut")
            .action(async || {
                // Placeholder
            }),
    )
    .subcommand(
        Command::new("buyHealthPotion")
            .about("Buy a health potion in Habitica")
            .alias("buyh")
            .action(async || {
                // Placeholder
            }),
    )
    .subcommand(
        Command::new("addActionItems")
            .about("Add action items from a temp file")
            .alias("aal")
            .action(async || {
                // add_action_items();
            }),
    )
    .subcommand(
        Command::new("skip")
            .about("Skip a Habitica todo")
            .alias("sk")
            .arg(Arg::new("num").default_value("1"))
            .action(async || {
                // let num = matches.value_of("num").unwrap().parse::<i32>().unwrap();
                // skip_habitica_todo(ppc, num);
            }),
    )
    .subcommand(
        Command::new("dumpLog")
            .about("Dump the complete Habitica log")
            .action(async || {
                // dump_habitica_logs(ppc);
            }),
    )
    .subcommand(
        Command::new("listDailies")
            .about("List all dailies from the local db")
            .action(async || {
                // Placeholder
            }),
    )
    .subcommand(
        Command::new("printTask")
            .about("Print a specific todo from Habitica")
            .arg(Arg::new("id").required(true))
            .action(async || {
                // let id = matches.value_of("id").unwrap();
                // Placeholder
            }),
    )
}

fn format_date(date: chrono::DateTime<Utc>) -> String {
    date.format("%Y-%m-%d").to_string()
}

#[derive(Serialize, Deserialize)]
struct HabiticaDaily {
    id: String,
    text: String,
    notes: String,
    priority: Option<String>,
    frequency: String,
}

#[derive(Serialize, Deserialize)]
struct HabiticaLog {
    date: String,
    data: String, // JSON string
}

fn setup_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habitica_dailies (
            id TEXT PRIMARY KEY,
            text TEXT NOT NULL,
            notes TEXT,
            priority TEXT,
            frequency TEXT
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habitica_log (
            date TEXT PRIMARY KEY,
            data TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

// In-progress translation. Many functions will need the PPC context passed in.

/*
async fn save_habitica_dailies(ppc: &PPC) -> Result<()> {
    println!("######### cli bed cmd #########");
    setup_tables(&ppc.c2vi.db)?;

    // Assuming ppc.habitica.get_tasks returns a Result
    let dailies = ppc.habitica.get_tasks("dailys").await?;
    write_dailies_to_database(&dailies, &ppc.c2vi.db).await?;
    Ok(())
}

async fn write_dailies_to_database(dailies: &[HabiticaTask], db: &Connection) -> Result<()> {
    println!("Syncing {} dailies to database...", dailies.len());
    let tx = db.transaction()?;
    for daily in dailies {
        tx.execute(
            "INSERT INTO habitica_dailies (id, text, notes, priority, frequency)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                text=excluded.text,
                notes=excluded.notes,
                priority=excluded.priority,
                frequency=excluded.frequency",
            params![
                daily.id,
                daily.text,
                daily.notes.as_deref().unwrap_or(""),
                daily.priority.map(|p| p.to_string()),
                daily.frequency.as_deref().unwrap_or("daily"),
            ],
        )?;
    }
    tx.commit()
}

async fn add_action_items(ppc: &PPC) -> Result<()> {
    let initial_content = "
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
bed 22:10
";
    let tmp_file = env::temp_dir().join(format!("habitica-tasks-{}.md", Utc::now().timestamp()));
    fs::write(&tmp_file, initial_content.trim())?;

    StdCommand::new("nvim").arg(&tmp_file).status()?;

    let list_str = fs::read_to_string(&tmp_file)?;
    fs::remove_file(&tmp_file)?;

    let tasks: Vec<&str> = list_str.lines().map(|t| t.trim()).filter(|t| !t.is_empty()).collect();
    let mut task_ids = vec![];

    for todo_text in tasks {
        // Assuming api_request exists and works like this
        let data = ppc.habitica.api_request("POST", "tasks/user", Some(serde_json::json!({ "text": todo_text, "type": "todo" }))).await?;
        let task_id = data["id"].as_str().unwrap().to_string();
        println!("Added to-do: '{}' with id '{}'", todo_text, task_id);
        task_ids.push(task_id);
    }

    for task_id in task_ids {
        ppc.habitica.api_request("POST", &format!("tasks/{}/move/to/-1", task_id), None).await?;
    }

    Ok(())
}
*/
