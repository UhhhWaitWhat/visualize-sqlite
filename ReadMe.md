# Visualize an sqlite database

Create simple visualizations of sqlite databases in GraphViz `dot` format.

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
    let dot_input = Schema::load(&db).unwrap();

    println!("{}", dot_input);
}
```

## Sample Output

![Sample Output](./example.png)