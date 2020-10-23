use raio::ll::connection::Connection;
use raio::ll::connection;
use raio::ll::version::Version;
use raio::client::request::{Hello, GoodBye, Run, Pull};
use raio::client::response::Response;
use packs::ExtractRef;

#[async_std::test]
/// 1. Opens a bolt connection,
/// 2. establishes a handshake and checks for returned version to be 4.1.
/// 3. Sends a HELLO, authenticates therefore, expects a SUCCESS response.
/// 4. Runs a "RETURN 1 as x"
/// 5. checks for "x" as being a field in the SUCCESS message,
/// 6. PULLs all
/// 7. receives RECORD 
/// 8. receives SUCCESS with has_more = false
/// 9. closes the connection with a GOODBYE afterwards.
pub async fn open_connection_hello() -> Result<(), connection::Error> {
    let mut connection = Connection::connect("localhost:7687").await?;
    let version = connection
        .handshake(
            &[
                Version::new(4, 1),
                Version::new(4, 0),
                Version::empty(),
                Version::empty()])
        .await?;

    assert_eq!(Version::new(4, 1), version);

    // now send HELLO
    let written =
        connection
            .send(Hello::new("integrationtest_raio", "0.2.0", "basic", "neo4j", "mastertest"))
            .await?;
    assert!(written > 0);

    // ... and expect a success:
    let response = connection.recv_response().await?;
    assert!(response.is_success(), "expected Success, but response was: {:?}", response);

    // Send a query:
    let mut run  = Run::new("RETURN $x as x");
    run.param("x", 42);

    let written = connection.send(run).await?;
    assert!(written > 0);

    let response = connection.recv_response().await?;
    match response {
        Response::Success(suc) => {
            assert!(
                suc.fields().unwrap().contains(&&String::from("x")), 
                "Expected a SUCCESS with field 'x'");
        },
        _ => panic!("Expected SUCCESS bot got {:?}", response),
    }

    let written = connection.send(Pull::all_from_last()).await?;
    assert!(written > 0);

    let response = connection.recv_response().await?;
    match response {
        Response::Record(r) =>
            assert_eq!(i64::extract_ref(r.data.first().unwrap()), Some(&42)),
        _ => panic!("Expected RECORD but got {:?}", response),
    }

    let response = connection.recv_response().await?;
    match response {
        Response::Success(suc) => {
            assert!(!suc.has_more());
        },
        _ => panic!("Expected SUCCESS but got {:?}", response),
    }

    // close friendly with GOODBYE:
    let written = connection.send(GoodBye {}).await?;
    assert!(written > 0);

    Ok(())
}
