use std::sync::{Mutex, Arc};
use std::thread;


struct A {
    a: i32,
    b: String,
}

fn main() {
    let counter = Arc::new(A { a: 1, b: String::new() });

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            a(&*counter);
        });
    }

    {
        let data = Arc::new(42);

        // 从Arc中获取值的引用
        let value_ref = &*data;

        // 打印值
        println!("Value: {}", value_ref);
    }
}


fn a(a: &A) {}