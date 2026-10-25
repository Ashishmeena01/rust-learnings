use macrolib1::{log_call};

#[log_call]
fn dm(){
    println!("hello from the demo function");
}

fn main() {
    dm();
    println!("Hello, world!");
}
