pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter()
         .map(|b| format!("{b:02X}"))
         .collect::<Vec<String>>()
         .join(" ") 
}
