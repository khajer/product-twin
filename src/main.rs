use neo4rs::{Graph};
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), neo4rs::Error>{
    dotenv().ok();
    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password123".to_string());

    test_connection(&uri, &user, &password).await?;
    Ok(())
}

async fn test_connection(uri: &str, user: &str, password: &str) -> Result<(), neo4rs::Error>{
    let graph = Graph::new(uri, user, password).await?;

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
