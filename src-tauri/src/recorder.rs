// use gst::prelude::*;

pub enum RecordCommand {
    Start,
    Pause,
    Resume,
    Stop,
}

pub type RecordChannel = tauri::async_runtime::Sender<RecordCommand>;

// pub fn gstreamer_loop(
//     pipeline: gst::Pipeline,
//     on_message: impl Fn(&gst::Message) -> bool,
// ) -> crate::Result<()> {
//     println!("[gstream_loop]: {:?}", &pipeline);
//     pipeline.set_state(gst::State::Playing)?;

//     let bus = pipeline.bus().unwrap();

//     for message in bus.iter_timed(gst::ClockTime::NONE) {
//         let view = message.view();
//         if let gst::MessageView::Eos(_) = view {
//             println!("[gstreamer_loop]: Eos");
//             break;
//         }

//         if let gst::MessageView::Error(err) = view {
//             println!("[gstreamer_loop]: Error: {:?}", err);
//             return Err(format!("{:?}", err).into());
//         }

//         let should_break = on_message(&message);
//         println!("[gstreamer_loop]: Should break: {should_break}");
//         if should_break {
//             break;
//         }
//     }

//     pipeline.set_state(gst::State::Null)?;

//     println!("[gstream_loop]: Leave {:?}", &pipeline);
//     Ok(())
// }
