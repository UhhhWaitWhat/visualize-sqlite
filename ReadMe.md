# Visualize an sqlite database

Create simple visualizations of sqlite databases in GraphViz `dot` format.

This version currently works with the latest prerelease version of diesel (2.0.0-rc.0).
Use version 1.x of this crate if you need to work with version 1.x of diesel.

**CLI**

```bash
visualize-sqlite your_sqlite_database.db | dot -Tpng -Gfontname='Fira Mono' -Gfontcolor='#586e75' -Gbgcolor='#fdf6e3' -Nfontname='Fira Mono' -Nfontcolor='#586e75' -Efontname='Fira Mono' > output.png
```

**API**

```rust
use diesel::SqliteConnection;
use visualize_sqlite::Schema;

fn main() {
    let db = SqliteConnection::establish("your_sqlite_database.db").unwrap();
    let dot_input = Schema::load(&mut db).unwrap();

    println!("{}", dot_input);
}
```

## Sample Output

![Sample Output](./example.png)