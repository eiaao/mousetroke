//! CommonCrypto 随 libSystem 提供；不启动外部摘要命令，也不增加加密依赖。
use super::Error;
use std::ffi::c_void;

#[repr(C)]
struct Context {
    count: [u32; 2],
    hash: [u32; 8],
    buffer: [u32; 16],
}

#[link(name = "System")]
extern "C" {
    fn CC_SHA256_Init(context: *mut Context) -> i32;
    fn CC_SHA256_Update(context: *mut Context, data: *const c_void, length: u32) -> i32;
    fn CC_SHA256_Final(digest: *mut u8, context: *mut Context) -> i32;
}

pub(super) fn sha256(data: &[u8]) -> Result<String, Error> {
    let mut context = Context {
        count: [0; 2],
        hash: [0; 8],
        buffer: [0; 16],
    };
    let mut digest = [0u8; 32];
    // 安全：结构与 CommonDigest.h 的 CC_SHA256_CTX 一致；所有指针在调用期间有效。
    unsafe {
        if CC_SHA256_Init(&mut context) != 1 {
            return Err(Error::Checksum);
        }
        for chunk in data.chunks(1024 * 1024) {
            if CC_SHA256_Update(&mut context, chunk.as_ptr().cast(), chunk.len() as u32) != 1 {
                return Err(Error::Checksum);
            }
        }
        if CC_SHA256_Final(digest.as_mut_ptr(), &mut context) != 1 {
            return Err(Error::Checksum);
        }
    }
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(test)]
mod tests {
    #[test]
    fn common_crypto_known_vectors() {
        assert_eq!(
            super::sha256(b"abc").unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            super::sha256(b"").unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            super::sha256(&vec![b'a'; 1_000_000]).unwrap(),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }
}
