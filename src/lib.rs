#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use eyre::{Context, Result};
use std::fmt::Display;

mod raw {
    use diesel::sql_types::*;

    #[derive(QueryableByName)]
    pub struct Table {
        #[diesel(sql_type = Text)]
        pub name: String,
    }

    #[derive(QueryableByName)]
    pub struct Column {
        #[diesel(sql_type = Text)]
        pub name: String,
        #[diesel(sql_type = Text, column_name = "type")]
        pub typ: String,
        #[diesel(sql_type = Bool)]
        pub notnull: bool,
        #[diesel(sql_type = Bool)]
        pub pk: bool,
        #[diesel(sql_type = Nullable<Text>)]
        pub dflt_value: Option<String>,
    }

    #[derive(QueryableByName)]
    pub struct ForeignKey {
        #[diesel(sql_type = Text)]
        pub table: String,
        #[diesel(sql_type = Nullable<Text>)]
        pub to: Option<String>,
        #[diesel(sql_type = Text)]
        pub from: String,
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub typ: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub primary: bool,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub foreign_keys: Vec<ForeignKey>,
}

#[derive(Debug, Clone)]
pub struct ForeignKey {
    pub target_table: String,
    pub target_column: Option<String>,
    pub source_table: String,
    pub source_column: String,
}

#[derive(Debug, Clone)]
pub struct Schema(pub Vec<Table>);

impl Schema {
    fn get_tables(db: &mut SqliteConnection) -> Result<Vec<Table>> {
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

    fn get_columns(db: &mut SqliteConnection, table: &str) -> Result<Vec<Column>> {
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

    fn get_keys(db: &mut SqliteConnection, table: &str) -> Result<Vec<ForeignKey>> {
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

    pub fn load(db: &mut SqliteConnection) -> eyre::Result<Self> {
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
