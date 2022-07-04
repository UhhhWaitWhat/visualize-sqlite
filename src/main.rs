use diesel::prelude::*;
use eyre::{eyre, Context, Result};

use visualize_sqlite::Schema;

fn open_database() -> Result<SqliteConnection> {
    let mut args = std::env::args();
    args.next();

    match args.next() {
        None => Err(eyre!(
            "Please pass an sqlite database file as the first argument"
        )),
        Some(file) => {
            std::fs::metadata(&file).wrap_err("failed to open database")?;
            SqliteConnection::establish(&file).wrap_err("failed to open database")
        }
    }
}

fn main() -> Result<()> {
    println!("{}", Schema::load(&mut open_database()?)?);

    Ok(())
}
