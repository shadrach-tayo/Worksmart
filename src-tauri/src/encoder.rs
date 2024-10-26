// use ffmpeg;

// pub fn uyvy422_frame(bytes: &[u8], width: u32, height: u32) -> ffmpeg::frame::Video {
//     let mut frame = ffmpeg::frame::Video::new(ffmpeg::format::Pixel::YUYV422, width, height);

//     frame.data_mut(0).copy_from_slice(bytes);

//     frame
// }
