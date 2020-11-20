use raio::connectivity::connection::{Connection, ConnectionConfig};
use raio::connectivity::connection;
use raio::connectivity::version::Version;
use raio::messaging::response::Response;
use packs::{ExtractRef, Value};
use raio::messaging::request::{Run, Pull, GoodBye};

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
    let run  =
        Run::new(
            String::from("RETURN $x as x, $y as y, $b as b"),
            vec!(
                (String::from("x"), Value::Integer(42)),
                (String::from("y"), Value::Integer(-34882)),
                (String::from("b"), Value::Boolean(true))
            ).into_iter().collect());

    let written = connection.send(&run).await?;
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
        },
        _ => panic!("Expected SUCCESS but got {:?}", response),
    }

    let written = connection.send(&Pull::all_from_last()).await?;
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
    let written = connection.send(&GoodBye {}).await?;
    assert!(written > 0);

    Ok(())
}
