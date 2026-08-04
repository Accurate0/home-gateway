use crate::http_client::PartialWindow;

pub enum Refresh {
    None,
    Clear,
    Image(Option<String>),
    Partial {
        hash: Option<String>,
        window: PartialWindow,
    },
}
