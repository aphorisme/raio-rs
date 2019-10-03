use raio::net::sync::*;
use raio::packing::Run;

#[test]
pub fn simple_open() {
    let client = Client::open("localhost:7687", "raio_service", "12345");
    assert!(client.is_ok());
}

#[test]
pub fn simple_cypher() {
    let mut client = Client::open("localhost:7687", "raio_service", "12345").expect("cannot open");

    let query = Run::statement("RETURN 1 as num");
    let res = client.run(&query).expect("Cannot run statement");

    println!("result: {:?}", res);

    assert_eq!(res.get_record(0).unwrap().get_value("num"), Some(&1));
}
