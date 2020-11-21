use raio::client::error::ClientError;
use raio::client::{Client, ClientConfig};
use raio::client::auth::Basic;
use raio::messaging::commit_prepare::CommitPrepare;
use raio::messaging::query::Query;

#[async_std::test]
pub async fn transaction_simple() -> Result<(), ClientError> {
    let client =
        Client::create(
            "localhost:7687",
            Basic::new("neo4j",
                       "mastertest"),
            ClientConfig::default("raio-rs-test",
                                  "0.2.0"));

    let mut transaction = client.begin(CommitPrepare::new()).await?;

    let mut query_1 = Query::new("RETURN $x + 42 as x");
    query_1.param("x", 3);

    let mut query_2 = Query::new("RETURN $y as y");
    query_2.param("y", true);

    let res_1 = transaction.run(&query_1).await?;
    assert_eq!(
        res_1.first().expect("At least one result in _1").get_field_typed("x"),
        Some(&45));

    let res_2 = transaction.run(&query_2).await?;
    assert_eq!(
        res_2.first().expect("At least one result in _2").get_field_typed("y"),
        Some(&true));

    transaction.commit().await?;

    Ok(())
}