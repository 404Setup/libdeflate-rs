fn main() {
    let mut s1: u32 = 65520;
    let mut s2: u32 = 65520;
    for _ in 0..5553 { // One more byte
        s1 += 255;
        s2 += s1;
    }
    println!("s1: {}, s2: {}", s1, s2);
}
