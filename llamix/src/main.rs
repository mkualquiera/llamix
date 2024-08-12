fn main() {
    // Clear screen by printing a lot of newlines
    for _ in 0..100 {
        println!();
    }
    println!("Hello from llamix!");
    // read input to prevent the program from closing immediately
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        // echo the input
        print!("{}", input);
    }
}
