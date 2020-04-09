use nalgebra::Vector2;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct SpritesheetLoader {
    load_requests: Receiver<(String, PathBuf)>,
    loaded_textures: Sender<(String, Vector2<u32>, Vec<u8>)>,
}

impl SpritesheetLoader {
    pub fn build() -> (
        Sender<(String, PathBuf)>,
        Receiver<(String, Vector2<u32>, Vec<u8>)>,
        Self,
    ) {
        let (load_requests_sender, load_requests_receiver) = channel();
        let (loaded_textures_sender, loaded_textures_receiver) = channel();
        (
            load_requests_sender,
            loaded_textures_receiver,
            SpritesheetLoader {
                load_requests: load_requests_receiver,
                loaded_textures: loaded_textures_sender,
            },
        )
    }

    pub fn run(self) {
        loop {
            for (name, path) in self.load_requests.recv() {
                println!("Loading sprite {}", name);
                let buffer: Option<(Vector2<u32>, Vec<u8>)> = try {
                    let buffer = image::load(
                        BufReader::new(File::open(path).ok()?),
                        image::ImageFormat::Png,
                    )
                    .ok()?
                    .flipv()
                    .to_rgba();
                    let (width, height) = buffer.dimensions();
                    ([width, height].into(), buffer.into_raw())
                };
                if let Some((dimensions, buffer)) = buffer {
                    let result = self.loaded_textures.send((name, dimensions, buffer));
                    if let Err(_) = result {
                        println!("Failed to send a loaded spritesheet to the draw module");
                    }
                } else {
                    println!("Failed to load an image for a spritesheet");
                }
            }
        }
    }
}
