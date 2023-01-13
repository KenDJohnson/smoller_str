use smoller_str_macro::{smoller_str, EnumStr};

#[smoller_str]
#[derive(EnumStr, Clone, Copy)]
pub enum Foo {
    #[value("first_one")]
    FirstOne,
    #[value("The - Second")]
    TheSecond,
}

fn main() {}
