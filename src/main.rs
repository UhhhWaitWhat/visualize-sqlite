use std::fmt::Display;

use eyre::{eyre, Context, Result};
use rusqlite::{params, Connection, OpenFlags};

#[derive(Debug)]
struct Column {
    name: String,
    typ: String,
    nullable: bool,
    default: Option<String>,
    primary: bool,
}

#[derive(Debug)]
struct Table {
    name: String,
    columns: Vec<Column>,
    foreign_keys: Vec<ForeignKey>,
}

#[derive(Debug)]
struct ForeignKey {
    target_table: String,
    target_column: String,
    source_table: String,
    source_column: String,
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} [shape=plaintext label=< <table border='0' cellborder='1' cellspacing='0' cellpadding='5'>
                <tr><td border='0'></td><td colspan='2'><b>{}</b></td></tr>",
            self.name, self.name
        )?;

        for column in &self.columns {
            writeln!(
                f,
                "<tr><td {} width='16'></td><td>{}</td><td port='{}'>{}</td></tr>",
                if column.primary {
                    "bgcolor='#2aa198'"
                } else if column.nullable {
                    "bgcolor='#6c71c4'"
                } else {
                    ""
                },
                column.name,
                column.name,
                column.typ,
            )?;
        }

        writeln!(f, "</table> >]")?;

        for key in &self.foreign_keys {
            key.fmt(f)?;
        }

        Ok(())
    }
}

impl Display for ForeignKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}:{} -> {}",
            self.source_table, self.source_column, self.target_table
        )
    }
}

fn open_database() -> Result<Connection> {
    let mut args = std::env::args();
    args.next();

    match args.next() {
        None => Err(eyre!(
            "Please pass an sqlite database file as the first argument"
        )),
        Some(file) => Connection::open_with_flags(file, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .wrap_err("failed to open database"),
    }
}

fn get_tables(db: &Connection) -> Result<Vec<Table>> {
    let mut statement = db.prepare(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT IN ('sqlite_sequence')",
    )?;

    let tables = statement
        .query_and_then([], |row| {
            Ok(Table {
                name: row.get(0)?,
                columns: get_columns(db, row.get(0)?)?,
                foreign_keys: get_keys(db, row.get(0)?)?,
            })
        })?
        .collect();

    tables
}

fn get_columns(db: &Connection, table: String) -> Result<Vec<Column>> {
    let mut statement = db.prepare("SELECT * FROM pragma_table_info(?)")?;

    let columns = statement
        .query_and_then(params![table], |row| {
            Ok(Column {
                name: row.get("name")?,
                typ: row.get("type")?,
                nullable: !row.get("notnull")?,
                primary: row.get("pk")?,
                default: row.get("dflt_value")?,
            })
        })?
        .collect();

    columns
}

fn get_keys(db: &Connection, table: String) -> Result<Vec<ForeignKey>> {
    let mut statement = db.prepare("SELECT * FROM pragma_foreign_key_list(?)")?;

    let keys = statement
        .query_and_then(params![table], |row| {
            Ok(ForeignKey {
                target_table: row.get("table")?,
                target_column: row.get("to")?,
                source_table: table.clone(),
                source_column: row.get("from")?,
            })
        })?
        .collect();

    keys
}

fn main() -> Result<()> {
    let db = open_database()?;

    println!("digraph {{");
    println!("rankdir=LR;");

    for table in get_tables(&db).wrap_err("failed to get tables")? {
        print!("{}", table);
    }
    println!("}}");

    Ok(())
}
