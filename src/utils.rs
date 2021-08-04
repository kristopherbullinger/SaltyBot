pub fn is_profane(msg: &str) -> bool {
	let lowerd = msg.to_ascii_lowercase();
	lowerd.contains("nigga")
}
