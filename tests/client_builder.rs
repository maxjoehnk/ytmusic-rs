use ytmusic::YoutubeMusicClient;

pub fn build_client() -> YoutubeMusicClient {
    let mut client = YoutubeMusicClient::new(env!("COOKIES"), None).unwrap();
    client.set_visitor_id(env!("VISITOR_ID").to_string());

    client
}
