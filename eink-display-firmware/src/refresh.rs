use crate::http_client::PartialWindow;

pub enum Refresh {
    Clear,
    Image(Option<String>),
    Partial {
        hash: Option<String>,
        window: PartialWindow,
    },
}
