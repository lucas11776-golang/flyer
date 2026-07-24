


pub fn merge<T>(mut i: Vec<T>, mut j: Vec<T>) -> Vec<T> {
    i.append(&mut j);
    return i;
}