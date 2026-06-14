#[derive(Debug, Clone, PartialEq, Default)]
pub enum FocusedPane {
    Sidebar,
    #[default]
    Url,
    SendButton,
    Params,
    Body,
    Response,
}
