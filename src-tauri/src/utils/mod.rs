pub mod protobuf;
pub mod http;
pub mod window_state;
pub mod settings;
#[cfg(target_os = "windows")]
pub mod keyring_win;

pub fn get_oauth_client_id() -> String {
    const ID_BYTES: &[u8] = &[107, 106, 109, 107, 106, 106, 108, 106, 108, 106, 111, 99, 107, 119, 46, 55, 50, 41, 41, 51, 52, 104, 50, 104, 107, 54, 57, 40, 63, 104, 105, 111, 44, 46, 53, 54, 53, 48, 50, 110, 61, 110, 106, 105, 63, 42, 116, 59, 42, 42, 41, 116, 61, 53, 53, 61, 54, 63, 47, 41, 63, 40, 57, 53, 52, 46, 63, 52, 46, 116, 57, 53, 55];
    let decoded: Vec<u8> = ID_BYTES.iter().map(|b| b ^ 0x5A).collect();
    String::from_utf8(decoded).unwrap_or_default()
}

pub fn get_oauth_client_secret() -> String {
    const SEC_BYTES: &[u8] = &[29, 21, 25, 9, 10, 2, 119, 17, 111, 98, 28, 13, 8, 110, 98, 108, 22, 62, 22, 16, 107, 55, 22, 24, 98, 41, 2, 25, 110, 32, 108, 43, 30, 27, 60];
    let decoded: Vec<u8> = SEC_BYTES.iter().map(|b| b ^ 0x5A).collect();
    String::from_utf8(decoded).unwrap_or_default()
}
