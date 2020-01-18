mod shunting_yard;

use shunting_yard::{evaluator, Ctx};

fn main() {
    let ctx = Ctx::default();
    let res = evaluator::eval_str("max(1, 2)");
    match res {
        Ok(res) => println!("res = {}", res),
        Err(e) => println!("error: {:?}", e),
    }
}
