use green_thread::green;

fn main() {
    green::spawn_from_main(green::producer, 2 * 1024 * 1024);
}
