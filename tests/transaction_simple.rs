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
            ClientConfig::default("raio-rs/integrationtest",
                                  "0.2.0"));

    let mut transaction = client.begin(CommitPrepare::new()).await?;
    let mut query_01 = Query::new("RETURN $x as x");
    query_01.param("x", 42);

    let mut query = transaction.run(query_01).await?;
    let results = query.pull().await?;
    transaction.commit().await?;

    Ok(())
}