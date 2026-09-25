//! MPEG-1 program-stream decoder.
//!
//! The decode implementation is Dominic Szablewski's pl_mpeg (MIT), vendored
//! under `vendor/pl_mpeg.h`.

mod backend;

pub use backend::MpegDecoder;

#[derive(Clone, Debug)]
pub struct MpegFrame {
    pub pts_ms: u32,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn is_mpeg_packet(bytes: &[u8]) -> bool {
    bytes.len() >= 4
        && bytes[0] == 0
        && bytes[1] == 0
        && bytes[2] == 1
        && matches!(bytes[3], 0xB3 | 0xBA)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_a_buffer_that_is_not_mpeg() {
        let err = MpegDecoder::open(b"not a movie".to_vec())
            .err()
            .expect("non-mpeg buffer should fail");
        assert!(err.contains("mpeg") || err.contains("MPEG") || err.contains("video"));
    }

    #[test]
    fn decodes_one_frame_from_sena_mpeg_fixture() {
        let Some(path) = std::env::var_os("SENA_MPEG_FIXTURE") else {
            return;
        };
        let bytes = std::fs::read(path).expect("read SENA_MPEG_FIXTURE");
        let mut decoder = MpegDecoder::open(bytes).expect("open mpeg");
        let frame = decoder.next_frame().expect("decode").expect("frame");
        assert!(frame.width >= 16);
        assert!(frame.height >= 16);
        assert_eq!(
            frame.rgba.len(),
            frame.width as usize * frame.height as usize * 4
        );
        assert!(frame.rgba.chunks_exact(4).any(|px| px[3] == 255));
    }
}
