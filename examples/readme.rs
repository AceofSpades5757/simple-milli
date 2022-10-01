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
