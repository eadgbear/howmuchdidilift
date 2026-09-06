mod convert;
pub mod login;
mod measures;
pub mod register;

pub use convert::Convert;
pub use login::Login;
pub use measures::MeasureList;
pub use register::Register;

#[derive(Debug, Clone, Copy, Default)]
pub enum Page {
    #[default]
    Convert,
    Login,
    Register,
    MeasureList,
}

impl Page {
    pub fn path(&self) -> &'static str {
        match self {
            Page::Convert => "/",
            Page::Login => "/login",
            Page::Register => "/register",
            Page::MeasureList => "/measures",
        }
    }
}
