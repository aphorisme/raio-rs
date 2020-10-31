use raio::client::{Client, ClientConfig};
use raio::client::auth::Basic;
use raio::client::error::ClientError;
use raio::client::query::Query;

#[async_std::test]
pub async fn auto_commit_simple() -> Result<(), ClientError> {
    let auth = Basic::new("neo4j", "mastertest");
    let mut client =
        Client::create("localhost:7687", auth, ClientConfig::default("clienttest", "0.2.0"))?;
    let mut query = Query::new("RETURN $x as x, $y as y, $b as b");
    query.param("x", 1);
    query.param("y", Some(String::from("Hello")));
    query.param("b", true);

    let records =
        client
            .run(query)
            .commit()
            .await?;

    let first = records.records().first().expect("Expected one result");
    assert_eq!(first.get_field_typed("x"), Some(&1));
    assert_eq!(first.get_field_typed("y"), Some(&String::from("Hello")));
    assert_eq!(first.get_field_typed("b"), Some(&true));

    Ok(())
}