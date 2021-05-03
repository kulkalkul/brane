
pub fn concat_bytes<T: AsRef<[u8]>>(vec: Vec<T>) -> Vec<u8> {
    let len = vec.iter().map(|x| x.as_ref().len()).sum();
    let mut combined = Vec::with_capacity(len);

    for value in vec.into_iter() {
        combined.extend_from_slice(value.as_ref());
    }

    combined
}
