use ytmusic::YoutubeMusicClient;

#[test]
pub fn get_library_albums_after_fetching_visitor_id() {
    futures::executor::block_on(async {
        let mut client = YoutubeMusicClient::new(env!("COOKIES"), None).unwrap();
        client.fetch_visitor_id().await.unwrap();

        let result = client.get_library_albums(None).await;

        assert!(result.is_ok());
    });
}
