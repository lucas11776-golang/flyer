
// Comment
fn add(a: *mut i32, b: i32) -> i32 {
    unsafe {
        *a += 10;
        return b + 10 + *a;
    }
}

struct Person<'a> {
    email: &'a str,
    password: &'a str,
}

// Comment
fn _main() {
    let mut i: i32 = 10;
    let raw_mut: *mut i32 = &mut i as *mut i32;
    let total: i32 = add(raw_mut, 10);
    
    println!("{0} {1} ", i, total);
    
    let person = Person {
        email: "jeo@doe.com",
        password:  "test@123",
    };
    
    println!("email: {0} password: {1} ", person.email, person.password);
}

// Printing take time - (touching IO)
// ab -c 100 -n 1000 http://127.0.0.1:9999/