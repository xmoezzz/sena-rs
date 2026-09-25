use crate::MpegFrame;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::MpegFrame;

    #[repr(C)]
    struct Plm {
        _private: [u8; 0],
    }

    #[repr(C)]
    struct PlmPlane {
        width: u32,
        height: u32,
        data: *mut u8,
    }

    #[repr(C)]
    struct PlmFrame {
        time: f64,
        width: u32,
        height: u32,
        y: PlmPlane,
        cr: PlmPlane,
        cb: PlmPlane,
    }

    unsafe extern "C" {
        fn plm_create_with_memory(bytes: *mut u8, length: usize, free_when_done: i32) -> *mut Plm;
        fn plm_destroy(plm: *mut Plm);
        fn plm_set_audio_enabled(plm: *mut Plm, enabled: i32);
        fn plm_set_video_enabled(plm: *mut Plm, enabled: i32);
        fn plm_set_loop(plm: *mut Plm, looped: i32);
        fn plm_rewind(plm: *mut Plm);
        fn plm_decode_video(plm: *mut Plm) -> *mut PlmFrame;
        fn plm_seek_frame(plm: *mut Plm, time: f64, seek_exact: i32) -> *mut PlmFrame;
        fn plm_frame_to_rgba(frame: *mut PlmFrame, dest: *mut u8, stride: i32);
    }

    pub struct MpegDecoder {
        inner: *mut Plm,
        // pl_mpeg borrows this allocation until plm_destroy.
        #[allow(dead_code)]
        bytes: Vec<u8>,
    }

    impl MpegDecoder {
        pub fn open(mut bytes: Vec<u8>) -> Result<Self, String> {
            if !crate::is_mpeg_packet(&bytes) {
                return Err("buffer is not an MPEG program or elementary stream".to_owned());
            }
            let inner = unsafe { plm_create_with_memory(bytes.as_mut_ptr(), bytes.len(), 0) };
            if inner.is_null() {
                return Err("MPEG decoder rejected the stream".to_owned());
            }
            unsafe {
                plm_set_audio_enabled(inner, 0);
                plm_set_video_enabled(inner, 1);
                plm_set_loop(inner, 0);
            }
            Ok(Self { inner, bytes })
        }

        pub fn next_frame(&mut self) -> Result<Option<MpegFrame>, String> {
            let frame = unsafe { plm_decode_video(self.inner) };
            if frame.is_null() {
                return Ok(None);
            }
            Ok(Some(unsafe { rgba_frame(frame) }))
        }

        pub fn rewind(&mut self) {
            unsafe { plm_rewind(self.inner) };
        }

        pub fn seek_ms(&mut self, ms: u32) -> Result<Option<MpegFrame>, String> {
            let frame = unsafe { plm_seek_frame(self.inner, f64::from(ms) / 1000.0, 0) };
            if frame.is_null() {
                return Ok(None);
            }
            Ok(Some(unsafe { rgba_frame(frame) }))
        }
    }

    impl Drop for MpegDecoder {
        fn drop(&mut self) {
            if !self.inner.is_null() {
                unsafe { plm_destroy(self.inner) };
                self.inner = std::ptr::null_mut();
            }
        }
    }

    unsafe fn rgba_frame(frame: *mut PlmFrame) -> MpegFrame {
        let width = (*frame).width.max(1);
        let height = (*frame).height.max(1);
        let mut rgba = vec![0u8; width as usize * height as usize * 4];
        plm_frame_to_rgba(frame, rgba.as_mut_ptr(), (width * 4) as i32);
        for pixel in rgba.chunks_exact_mut(4) {
            pixel[3] = 255;
        }
        let pts_ms = if (*frame).time.is_finite() && (*frame).time > 0.0 {
            ((*frame).time * 1000.0) as u32
        } else {
            0
        };
        MpegFrame {
            pts_ms,
            width,
            height,
            rgba,
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod native {
    use super::MpegFrame;

    pub struct MpegDecoder;

    impl MpegDecoder {
        pub fn open(_bytes: Vec<u8>) -> Result<Self, String> {
            Err("MPEG playback is not compiled for wasm".to_owned())
        }

        pub fn next_frame(&mut self) -> Result<Option<MpegFrame>, String> {
            Err("MPEG playback is not compiled for wasm".to_owned())
        }

        pub fn rewind(&mut self) {}

        pub fn seek_ms(&mut self, _ms: u32) -> Result<Option<MpegFrame>, String> {
            Err("MPEG playback is not compiled for wasm".to_owned())
        }
    }
}

pub use native::MpegDecoder;
