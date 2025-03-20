pub mod merkle_lib;

pub fn decode_ethereum_leaf(leaf_node: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut index = 0;
    let list_prefix = leaf_node[index];
    index += 1;
    let list_length = if list_prefix <= 0xf7 {
        (list_prefix - 0xc0) as usize
    } else {
        let len_of_length = (list_prefix - 0xf7) as usize;
        let mut length = 0usize;
        for _ in 0..len_of_length {
            length = (length << 8) | (leaf_node[index] as usize);
            index += 1;
        }
        length
    };
    let list_end = index + list_length;
    let key_prefix = leaf_node[index];
    index += 1;
    let odd_length = (key_prefix & 0x10) != 0;
    let mut nibbles = Vec::new();

    if odd_length {
        nibbles.push(key_prefix & 0x0F);
    }
    while index < list_end && leaf_node[index] != 0xa0 {
        let byte = leaf_node[index];
        nibbles.push(byte >> 4);
        nibbles.push(byte & 0x0F);
        index += 1;
    }
    let mut key = Vec::new();
    for chunk in nibbles.chunks(2) {
        if chunk.len() == 2 {
            key.push((chunk[0] << 4) | chunk[1]);
        } else {
            key.push(chunk[0] << 4);
        }
    }
    let value_prefix = leaf_node[index];
    index += 1;
    let value_length = if value_prefix <= 0xb7 {
        (value_prefix - 0x80) as usize
    } else {
        let len_of_length = (value_prefix - 0xb7) as usize;
        let mut length = 0usize;
        for _ in 0..len_of_length {
            length = (length << 8) | (leaf_node[index] as usize);
            index += 1;
        }
        length
    };
    let value = leaf_node[index..index + value_length].to_vec();
    // note that the key here is the hex encoded nibble key
    // i think we should use the full key for now
    (key, value)
}
