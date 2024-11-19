use std::path::PathBuf;
use image_compressor::compressor::Compressor;
use image_compressor::Factor;
use std::fs;

pub fn compress_image(source:PathBuf, dest: PathBuf) {
    let mut comp = Compressor::new(source.clone(), dest);
    comp.set_factor(Factor::new(80., 0.8));
    if let Err(err) = comp.compress_to_jpg() {
        // todo: trace error
        eprintln!("Failed to compress image {:?}", err);
    } else {
        fs::remove_file(source.clone()).unwrap_or_else(|err| {
            // todo: trace error
            eprintln!("Failed to remove source file after compression. source: {:?} {:?}", source.to_str(), err);
        });
    }
}
