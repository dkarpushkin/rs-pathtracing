
fn main() {
    let mut buffer = vec![1, 2, 3, 4, 5, 6];

    dbg!(&buffer);

    std::thread::scope(|s| {
        for a in buffer.iter_mut() {
            s.spawn(|| {
                *a *= 2;
            });
        }
    });

    dbg!(&buffer);
}
