use client_builder::build_client;

mod client_builder;

#[test]
pub fn search_query() {
    futures::executor::block_on(async {
        let client = build_client();

        let result = client.search("edm playlist").await.unwrap();

        assert!(result.len() > 10)
    });
}
