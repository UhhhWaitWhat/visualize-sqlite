#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use eyre::{Context, Result};
use std::fmt::Display;

mod raw {
    use diesel::sql_types::*;

    #[derive(QueryableByName)]
    pub struct Table {
        #[sql_type = "Text"]
        pub name: String,
    }

    #[derive(QueryableByName)]
    pub struct Column {
        #[sql_type = "Text"]
        pub name: String,
        #[sql_type = "Text"]
        #[column_name = "type"]
        pub typ: String,
        #[sql_type = "Bool"]
        pub notnull: bool,
        #[sql_type = "Bool"]
        pub pk: bool,
        #[sql_type = "Nullable<Text>"]
        pub dflt_value: Option<String>,
    }

    #[derive(QueryableByName)]
    pub struct ForeignKey {
        #[sql_type = "Text"]
        pub table: String,
        #[sql_type = "Nullable<Text>"]
        pub to: Option<String>,
        #[sql_type = "Text"]
        pub from: String,
    }
}

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
    target_column: Option<String>,
    source_table: String,
    source_column: String,
}

#[derive(Debug)]
pub struct Schema(Vec<Table>);

impl Schema {
    fn get_tables(db: &SqliteConnection) -> Result<Vec<Table>> {
        let tables: Vec<raw::Table> = diesel::sql_query(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT IN ('sqlite_sequence')",
        )
        .load(db)?;

        tables
            .into_iter()
            .map(|table| {
                Ok(Table {
                    foreign_keys: Self::get_keys(db, &table.name)
                        .wrap_err_with(|| format!("failed to get keys for {}", &table.name))?,
                    columns: Self::get_columns(db, &table.name)
                        .wrap_err_with(|| format!("failed to get columns for {}", &table.name))?,
                    name: table.name,
                })
            })
            .collect()
    }

    fn get_columns(db: &SqliteConnection, table: &str) -> Result<Vec<Column>> {
        let columns: Vec<raw::Column> =
            diesel::sql_query(format!("SELECT * FROM pragma_table_info('{}')", table)).load(db)?;

        Ok(columns
            .into_iter()
            .map(|column| Column {
                name: column.name,
                typ: column.typ,
                nullable: !column.notnull,
                primary: column.pk,
                default: column.dflt_value,
            })
            .collect())
    }

    fn get_keys(db: &SqliteConnection, table: &str) -> Result<Vec<ForeignKey>> {
        let keys: Vec<raw::ForeignKey> = diesel::sql_query(format!(
            "SELECT * FROM pragma_foreign_key_list('{}')",
            table
        ))
        .load(db)?;

        Ok(keys
            .into_iter()
            .map(|key| ForeignKey {
                target_table: key.table,
                target_column: key.to,
                source_table: table.to_owned(),
                source_column: key.from,
            })
            .collect())
    }

    pub fn load(db: &SqliteConnection) -> eyre::Result<Self> {
        Ok(Self(Self::get_tables(db)?))
    }
}

impl Display for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "digraph {{")?;
        writeln!(f, "rankdir=LR;")?;

        for table in &self.0 {
            table.fmt(f)?;
        }

        writeln!(f, "}}")
    }
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
