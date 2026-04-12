fn main() {
    let mut s1: u32 = 65520;
    let mut s2: u32 = 65520; // Initial s2
    for _ in 0..5552 {
        s1 += 255;
        s2 += s1;
    }
    println!("s1: {}, s2: {}", s1, s2);
}
