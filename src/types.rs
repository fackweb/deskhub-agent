pub static DESK_SEVICE_NAME: &str = "DeskHubService";

#[derive(Debug, Clone)]
pub enum AlertType {
    Error,
    Info,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub message: String,
    pub alert_type: AlertType,
}
