use client_builder::build_client;

mod client_builder;

#[test]
pub fn get_library_playlists() {
    futures::executor::block_on(async {
        let client = build_client();

        let result = client.get_library_playlists(None).await.unwrap();

        println!("{:?}", result);
        assert!(result.len() > 10)
    });
}

#[test]
pub fn get_library_albums() {
    futures::executor::block_on(async {
        let client = build_client();

        let result = client.get_library_albums(None).await.unwrap();

        println!("{:?}", result);
        assert!(result.len() > 10)
    });
}

#[test]
pub fn get_library_artists() {
    futures::executor::block_on(async {
        let client = build_client();

        let result = client.get_library_artists(None).await.unwrap();

        println!("{:?}", result);
        assert!(result.len() > 10)
    });
}

#[test]
pub fn get_library_songs() {
    futures::executor::block_on(async {
        let client = build_client();

        let result = client.get_library_songs(None).await.unwrap();

        println!("{:?}", result);
        assert!(result.len() > 10)
    });
}
