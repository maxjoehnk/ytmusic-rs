use client_builder::build_client;

mod client_builder;

const SAMPLE_PLAYLIST: &str = "RDCLAK5uy_kpxnNxJpPZjLKbL9WgvrPuErWkUxMP6x4"; // Eminem - Revival

#[test]
pub fn get_playlist() {
    futures::executor::block_on(async {
        let client = build_client();

        let result = client.get_playlist(SAMPLE_PLAYLIST).await.unwrap();

        assert!(result.is_some());
        let playlist = result.unwrap();
        println!("{:#?}", playlist);
        assert!(playlist.tracks.len() > 100)
    });
}
