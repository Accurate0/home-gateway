use super::UnraidImportance;

#[derive(Debug, Clone, PartialEq)]
pub struct UnraidNotification {
    pub id: String,
    pub title: String,
    pub subject: String,
    pub description: String,
    pub importance: UnraidImportance,
    pub link: Option<String>,
}
