use neo4rs::{Graph};

#[tokio::main]
async fn main() -> Result<(), neo4rs::Error>{
    test_connection().await?;
    Ok(())
}

async fn test_connection() -> Result<(), neo4rs::Error>{
    // create connection
    let graph = Graph::new("bolt://localhost:7687", "neo4j", "password123").await?;

    // test query
    let mut result = graph.execute(
            neo4rs::query("MATCH (p:Person) RETURN p.name AS name, p.age AS age")
        ).await?;

        while let Some(row) = result.next().await? {
            let name: String = row.get("name").expect("name");
            let age: i64 = row.get("age").expect("age");
            println!("{} is {}", name, age);
        }
    Ok(())
}
