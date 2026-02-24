import { createClient } from "@libsql/client";
import { drizzle } from "drizzle-orm/libsql";
import { sqliteTable, text, integer } from "drizzle-orm/sqlite-core";
import { eq } from "drizzle-orm";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { spawnSync } from "node:child_process";

export async function init(ppc: PPC) {
  ppc.c2vi = new C2vi(ppc);

  ppc.cli
    .command("bed", "command to run when c2vi goes to bed")
    .action(async () => {
      await save_habitica_dailies(ppc);
      await save_habitica_logs(ppc);
    });

  ppc.cli
    .command("clearTodos", "command to run when c2vi goes to bed")
    .alias("clt")
    .action(async () => {
      const todos = await ppc.habitica.get_tasks();
      for (const todo of todos) {
        await ppc.habitica.delete_task(todo.id);
      }
    });

  ppc.cli
    .command("listTodos", "command to run when c2vi goes to bed")
    .alias("dut")
    .action(async () => {
      const todos = await ppc.habitica.get_tasks();
      for (const todo of todos) {
        console.log(todo.text);
      }
    });

  ppc.cli
    .command("buyHealthPotion", "command to run when c2vi goes to bed")
    .alias("buyh")
    .action(async () => {
      await ppc.habitica.api_request("POST", "/user/buy-health-potion");
    });

  ppc.cli
    .command("addActionItems", "command to run when c2vi goes to bed")
    .alias("aal")
    .action(async () => {
      await add_action_items(ppc);
    });

  ppc.cli
    .command("test", "a command to run some test code")
    .alias("t")
    .action(async () => {});

  ppc.cli
    .command("skip [num]", "command to skip a habitica todo")
    .alias("sk")
    .action((num = 1) => {
      skip_habitica_todo(ppc, parseInt(num));
    });

  ppc.cli.command("dumpLog", "dump the complete habitica log").action(() => {
    dump_habitica_logs(ppc);
  });

  ppc.cli
    .command("listDailies", "list all dailies")
    .action(async (folder, options) => {
      const allDailies = await ppc.c2vi.db.select().from(dailiesTable).all();
      for (const task of allDailies) {
        console.log(task.id + ": " + task.text);
      }
    });

  ppc.cli
    .command("printTask <id>", "print a specific todo")
    .action(async (id) => {
      const data = await ppc.habitica.api_request("GET", `/tasks/${id}`, {});
      console.log(data.history);
    });
}

class C2vi {
  [key: string]: any;

  constructor(ppc: PPC) {
    this.ppc = ppc;
    this.db = drizzle(
      createClient({
        url: "file://" + ppc.config.local_storage_path + "/data.db",
      }),
    );
  }
}

function formatDate(d) {
  const pad = (n) => n.toString().padStart(2, "0");

  // Custom YYYY-MM-DD format
  const formatted = `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
  return formatted;
}

// ============= SCHEMA DEFINITION =============
const dailiesTable = sqliteTable("habitica_dailies", {
  id: text("id").primaryKey(),
  text: text("text").notNull(),
  notes: text("notes"),
  priority: text("priority"), // 0.1, 1, 1.5, 2
  frequency: text("frequency"), // 'daily', 'weekly', etc.
});
const habiticaLogTable = sqliteTable("habitica_log", {
  date: text("date").primaryKey(),
  data: text("data").notNull(),
});

async function save_habitica_dailies(ppc) {
  console.log("######### cli bed cmd #########");
  await ppc.c2vi.db.run(
    "CREATE TABLE IF NOT EXISTS habitica_dailies (id TEXT PRIMARY KEY, text TEXT NOT NULL, notes TEXT, priority TEXT, frequency TEXT)",
  );
  await ppc.c2vi.db.run(
    "CREATE TABLE IF NOT EXISTS habitica_log (date TEXT PRIMARY KEY, data TEXT NOT NULL)",
  );

  // ============= DATABASE INITIALIZATION =============
  if (!ppc.config.habitica.user_id || !ppc.config.habitica.api_token) {
    console.error(
      "Error: HABITICA_USER_ID and HABITICA_API_TOKEN env vars required",
    );
    console.error(
      "Get these from Habitica Settings > API and configure them in the ppc.config",
    );
    process.exit(1);
  }

  const dailies = await ppc.habitica.get_tasks("dailys");

  await writeDailiesToDatabase(dailies, ppc);
}

async function dump_habitica_logs(ppc) {
  const entries = await ppc.c2vi.db.select().from(habiticaLogTable).all();
  for (const entry of entries) {
    console.log(entry.date + ": ", JSON.parse(entry.data));
  }
}

async function skip_habitica_todo(ppc, num) {
  await ppc.c2vi.db.run(
    "CREATE TABLE IF NOT EXISTS habitica_dailies (id TEXT PRIMARY KEY, text TEXT NOT NULL, notes TEXT, priority TEXT, frequency TEXT)",
  );
  await ppc.c2vi.db.run(
    "CREATE TABLE IF NOT EXISTS habitica_log (date TEXT PRIMARY KEY, data TEXT NOT NULL)",
  );

  const todos = await ppc.habitica.get_tasks("todos");

  const date = new Date();
  const save_for_yesterday = date.getHours() < 12;
  if (save_for_yesterday) {
    console.log(
      "Saving logs for yesterday!!!! because it is " +
        date.getHours() +
        "hours of the day",
    );
    date.setDate(date.getDate() - 1);
  }

  const result = await ppc.c2vi.db
    .select()
    .from(habiticaLogTable)
    .where(eq(habiticaLogTable.date, formatDate(date)));

  const data = result[0]
    ? JSON.parse(result[0].data)
    : {
        dailies_done: [],
        dailies_skipped: [],
        todos_done: [],
        todos_skipped: [],
      };

  for (let i = 0; i < num; i++) {
    const todo = todos[i];
    if (!todo) continue;
    await ppc.habitica.delete_task(todo.id);
    await incrementHabit(ppc);

    data.todos_skipped.push({
      id: todo.id,
      text: todo.text,
      notes: todo.notes || "",
      checklist: todo.checklist || [],
      tags: todo.tags || [],
    });
  }

  // saving to db
  await ppc.c2vi.db
    .insert(habiticaLogTable)
    .values({ date: formatDate(date), data: JSON.stringify(data) })
    .onConflictDoUpdate({
      target: habiticaLogTable.date,
      set: {
        data: JSON.stringify(data),
      },
    });
}

async function save_habitica_logs(ppc) {
  const date = new Date();
  const save_for_yesterday = date.getHours() < 12;
  if (save_for_yesterday) {
    console.log(
      "Saving logs for yesterday!!!! because it is " +
        date.getHours() +
        "hours of the day",
    );
    date.setDate(date.getDate() - 1);
  }

  const result = await ppc.c2vi.db
    .select()
    .from(habiticaLogTable)
    .where(eq(habiticaLogTable.date, formatDate(date)));

  const data = result[0]
    ? JSON.parse(result[0].data)
    : {
        dailies_done: [],
        dailies_skipped: [],
        todos_done: [],
        todos_skipped: [],
      };

  // dailies
  const dailies = await ppc.habitica.get_tasks("dailys");
  for (const daily of dailies) {
    let completed = daily.completed;
    let isDue = daily.isDue;
    if (save_for_yesterday) {
      const yesterday = daily.history.reduce((prev, current) => {
        return prev.date > current.date ? prev : current;
      });
      completed = yesterday.completed;
      isDue = yesterday.isDue;
    }
    if (!isDue) {
      continue;
    }
    if (completed) {
      data.dailies_done.push(daily.id);
      console.log("daily-done: " + daily.text);
    } else {
      data.dailies_skipped.push(daily.id);
      console.log("daily-skipped: " + daily.text);
    }
  }

  // todos done
  const todos = await ppc.habitica.get_tasks("completedTodos");
  for (const todo of todos) {
    const dateCompleted = new Date(todo.dateCompleted);

    // check if this todo is already saved in yesterday's log entry
    if (save_for_yesterday) {
      // get yesterday log entry
      const result = await ppc.c2vi.db
        .select()
        .from(habiticaLogTable)
        .where(eq(habiticaLogTable.date, formatDate(date)));

      const data = result[0]
        ? JSON.parse(result[0].data)
        : {
            dailies_done: [],
            dailies_skipped: [],
            todos_done: [],
            todos_skipped: [],
          };

      // check if this todo is already saved in yesterday's log entry
      const todoIndex = data.todos_done.findIndex((t) => t.id === todo.id);
      if (todoIndex !== -1) {
        continue; // skipp saving this todo
      }
    }

    if (dateCompleted.toDateString() == date.toDateString()) {
      console.log("todo-done:", todo.text);
      data.todos_done.push({
        id: todo.id,
        text: todo.text,
        notes: todo.notes || "",
        checklist: todo.checklist || [],
        tags: todo.tags || [],
      });
    }

    // we save_for_yesterday but the todo was completed today... also save that for this day
    if (
      save_for_yesterday &&
      dateCompleted.toDateString() == new Date().toDateString()
    ) {
      console.log("todo-done:", todo.text);
      data.todos_done.push({
        id: todo.id,
        text: todo.text,
        notes: todo.notes || "",
        checklist: todo.checklist || [],
        tags: todo.tags || [],
      });
    }
  }

  // saving to db
  await ppc.c2vi.db
    .insert(habiticaLogTable)
    .values({ date: formatDate(date), data: JSON.stringify(data) })
    .onConflictDoUpdate({
      target: habiticaLogTable.date,
      set: {
        data: JSON.stringify(data),
      },
    });
}

async function writeDailiesToDatabase(dailies, ppc) {
  console.log(`Syncing ${dailies.length} dailies to database...`);

  const records = dailies.map((daily) => ({
    id: daily.id,
    text: daily.text,
    notes: daily.notes || "",
    priority: daily.priority?.toString() || null,
    frequency: daily.frequency || "daily",
  }));

  async function upsertMany(records) {
    for (const record of records) {
      await ppc.c2vi.db
        .insert(dailiesTable)
        .values(record)
        .onConflictDoUpdate({
          target: dailiesTable.id,
          set: {
            text: record.text,
            notes: record.notes,
            priority: record.priority,
            frequency: record.frequency,
          },
        });
    }
  }

  await upsertMany(records);
}

async function incrementHabit(ppc) {
  const habits = await ppc.habitica.get_tasks("habits");
  let id = "";
  for (const habit of habits) {
    if ((habit.text = "skipped a planned task")) {
      id = habit.id;
    }
  }
  await ppc.habitica.api_request("POST", `tasks/${id}/score/down`);
}

async function add_action_items(ppc: PPC) {
  let task_ids = [];

  const tmpFile = path.join(os.tmpdir(), `habitica-tasks-${Date.now()}.md`);
  const initialContent = `
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
`;

  fs.writeFileSync(tmpFile, initialContent.trim());

  spawnSync("nvim", [tmpFile], { stdio: "inherit" });

  // read file
  const listStr = fs.readFileSync(tmpFile, "utf8");

  // delete file
  fs.unlinkSync(tmpFile);

  // Simple parser: filter out empty lines
  const tasks = listStr
    .split("\n")
    .map((t) => t.trim())
    .filter((t) => t.length > 0);

  // 5. Add tasks to Habitica
  for (const todoText of tasks) {
    const data = await ppc.habitica.api_request(
      "POST",
      "tasks/user",
      {},
      { text: todoText, type: "todo" },
    );
    task_ids.push(data.id);
    console.log(`Added to-do: '${todoText}' with id '${data.id}'`);
  }

  // 6. Move tasks to bottom
  for (const taskId of task_ids) {
    await ppc.habitica.api_request("POST", `tasks/${taskId}/move/to/-1`);
  }
}
