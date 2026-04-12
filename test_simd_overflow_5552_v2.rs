fn main() {
    let mut s1: u32 = 65520; // Max s1 before modulo
    let mut s2: u32 = 0;
    for _ in 0..5552 {
        s1 += 255;
        s2 += s1;
    }
    println!("s1: {}, s2: {}", s1, s2);
}
