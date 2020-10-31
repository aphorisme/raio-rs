use raio::ll::connection::{Connection, ConnectionConfig};
use raio::ll::connection;
use raio::ll::version::Version;
use raio::client::request::{GoodBye, Run, Pull};
use raio::client::response::Response;
use packs::ExtractRef;

#[async_std::test]
/// 1. Opens a bolt connection,
/// 2. establishes a handshake and checks for returned version to be 4.1.
/// 3. authenticates,
/// 4. Runs a "RETURN 1 as x"
/// 5. checks for "x" as being a field in the SUCCESS message,
/// 6. PULLs all
/// 7. receives RECORD
/// 8. receives SUCCESS with has_more = false
/// 9. closes the connection with a GOODBYE afterwards.
pub async fn open_connection_query() -> Result<(), connection::ConnectionError> {
    let mut connection = Connection::connect("localhost:7687", ConnectionConfig::default()).await?;
    let version = connection
        .handshake(
            &[
                Version::new(4, 1),
                Version::new(4, 0),
                Version::empty(),
                Version::empty()])
        .await?;

    assert_eq!(Version::new(4, 1), version);

    // now authenticate:
    connection.auth_hello("integrationtest_raio", "0.2.0", "basic", "neo4j", "mastertest").await?;

    // Send a query:
    let mut run  = Run::new("RETURN $x as x, $y as y, $b as b");
    run.param("x", 42);
    run.param("y", -34882);
    run.param("b", true);

    let written = connection.send(run).await?;
    assert!(written > 0);

    let response = connection.recv::<Response>().await?;
    match response {
        Response::Success(suc) => {
            let fields = suc.fields().unwrap();
            assert!(
                fields.contains(&&String::from("x")),
                "Expected a SUCCESS with field 'x'");
            assert!(
                fields.contains(&&String::from("y")),
                "Expected a SUCCESS with field 'y'");
            assert!(
                fields.contains(&&String::from("b")),
                "Expected a SUCCESS with field 'b'");
            assert!(suc.has_more());
        },
        _ => panic!("Expected SUCCESS but got {:?}", response),
    }

    let written = connection.send(Pull::all_from_last()).await?;
    assert!(written > 0);

    let response = connection.recv::<Response>().await?;
    match response {
        Response::Record(r) => {
            // 3 fields:
            assert_eq!(r.data.len(), 3);
            assert_eq!(i64::extract_ref(&r.data[0]), Some(&42));
            assert_eq!(i64::extract_ref(&r.data[1]), Some(&-34882));
            assert_eq!(bool::extract_ref(&r.data[2]), Some(&true));
        }
        _ => panic!("Expected RECORD but got {:?}", response),
    }

    let response = connection.recv::<Response>().await?;
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
