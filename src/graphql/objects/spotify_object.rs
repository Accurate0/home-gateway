use async_graphql::SimpleObject;

#[derive(SimpleObject)]
#[graphql(name = "SpotifyArtist", unresolvable = "name")]
pub struct SpotifyArtist {
    pub name: String,
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyItem", unresolvable = "uri")]
pub struct SpotifyItem {
    pub uri: String,
}
