#[derive(Debug, Clone)]
pub enum LocaleError {
    InvalidTagStructure(String),
}