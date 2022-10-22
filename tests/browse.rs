use client_builder::build_client;

mod client_builder;

const SAMPLE_ALBUM: &str = "MPREb_4pL8gzRtw1p"; // Eminem - Revival
const SAMPLE_VIDEO: &str = "tGWs0xKwhag"; // Oasis - Wonderwall (Remastered)
const SAMPLE_ARTIST: &str = "MPLAUCmMUZbaYdNH0bEd1PAlAqsA";

#[test]
pub fn get_album() {
    futures::executor::block_on(async {
        let client = build_client();

        let result = client.get_album(SAMPLE_ALBUM).await.unwrap();

        assert!(result.is_some());
        let album = result.unwrap();
        println!("{:#?}", album);
        assert_eq!(album.tracks.len(), 19)
    });
}

#[test]
pub fn get_artist() {
    futures::executor::block_on(async {
        let client = build_client();

        let result = client.get_artist(SAMPLE_ARTIST).await.unwrap();

        assert!(result.is_some());
        let artist = result.unwrap();
        println!("{:#?}", artist);
    });
}

#[test]
pub fn get_song() {
    futures::executor::block_on(async {
        let client = build_client();

        let result = client.get_song(SAMPLE_VIDEO).await.unwrap();

        assert!(result.is_some());
        let song = result.unwrap();
        println!("{:#?}", song);
    });
}
