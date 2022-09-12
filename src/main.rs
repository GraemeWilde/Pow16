use num_bigint::{ToBigUint, BigUint};

fn main() {

    let bui16: BigUint = 16.to_biguint().unwrap();
    let mut num = 16_u32.pow(4).to_biguint().unwrap();
    let mut pow = 4;

    loop {
        num = num * &bui16;
        pow += 1;

        let num_str = num.to_string();

        println!("Checking num: 16^{}", pow);

        if !num_str.contains(is1248) {
            println!("Num is 16^{} = {}", pow, num);
            break;
        }
    }
}

fn is1248(c: char) -> bool {
    match c {
        '1' | '2' | '4' | '8' => true,
        _ => false
    }
}
