_Disclaimer: This project and its author is not directly associated with
Meilisearch and its team, although I am a fan._

# Description

A simple implementation using the core engine of _Meilisearch_, `milli`, to
create an embedded search engine.

Uses the power of `milli` to create a basic, embedded search database. The only
requirement is that your document implements the `Serialize` and `Deserialize`
traits.

_Note: Settings all settings are currently hard-coded, such as the results
be 10._

# Usage

Add this to your `Cargo.toml` manifest file's list of dependencies.

`simple-milli = { git = "https://github.com/AceofSpades5757/simple-milli", tag = "v0.1.0", version = "0.1.0" }`

This is a basic example.

```rust
use serde::{Deserialize, Serialize};
use simple_milli::Database;

#[derive(Serialize, Deserialize)]
struct Document {
    id: usize,
    name: String,
}

fn main() {
    let documents: Vec<Document> = vec![
        Document {
            id: 100,
            name: "Document 1".to_string(),
        },
        Document {
            id: 101,
            name: "Document 2".to_string(),
        },
        Document {
            id: 102,
            name: "Document 3".to_string(),
        },
    ];

    let mut database = Database::new();
    database.add_documents(documents).unwrap();

    let results: Vec<Document> = database.search("Doc").unwrap();
    for doc in results {
        println!("Result Name: {}", doc.name);
    }
}
```

# References

- [Meilisearch search engine](https://www.meilisearch.com/)
- [GitHub - milli](https://github.com/meilisearch/milli)

_As `milli`, and some of its dependencies are not yet available on crates.io,
this crate cannot be published their either._
