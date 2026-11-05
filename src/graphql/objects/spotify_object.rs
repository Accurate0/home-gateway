use async_graphql::SimpleObject;

#[derive(SimpleObject)]
#[graphql(name = "Artist", unresolvable = "name")]
pub struct SpotifyArtist {
    pub name: String,
}

#[derive(SimpleObject)]
#[graphql(name = "Item", unresolvable = "uri")]
pub struct SpotifyItem {
    pub uri: String,
}
