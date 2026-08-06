use serde::Serialize;

/// FCM HTTP v1 send payload: `{ "message": { "token", "notification": { ... } } }`.
#[derive(Serialize)]
pub struct FcmSendRequest {
    pub message: FcmMessage,
}

#[derive(Serialize)]
pub struct FcmMessage {
    pub token: String,
    pub notification: FcmNotification,
}

#[derive(Serialize)]
pub struct FcmNotification {
    pub title: String,
    pub body: String,
}
