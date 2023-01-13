use smoller_str_macro::EnumStr;

#[derive(EnumStr, Clone, Copy)]
pub enum Foo {
    #[value("first_one")]
    FirstOne,
    #[value("The - Second")]
    TheSecond,
}

fn main() {}
