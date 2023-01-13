use smoller_str::*;

#[smoller_str(deref = false)]
#[derive(EnumStr, Clone, Copy)]
pub enum Foo {
    #[value("first_one")]
    FirstOne,
    #[value("The - Second")]
    TheSecond,
}

fn main() {}
