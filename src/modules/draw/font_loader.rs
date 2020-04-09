use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct FontLoader {
    load_requests: Receiver<(String, PathBuf)>,
    loaded_fonts: Sender<(String, Vec<u8>)>,
}

impl FontLoader {
    pub fn build() -> (Sender<(String, PathBuf)>, Receiver<(String, Vec<u8>)>, Self) {
        let (load_requests_sender, load_requests_receiver) = channel();
        let (loaded_fonts_sender, loaded_fonts_receiver) = channel();
        (
            load_requests_sender,
            loaded_fonts_receiver,
            FontLoader {
                load_requests: load_requests_receiver,
                loaded_fonts: loaded_fonts_sender,
            },
        )
    }

    pub fn run(self) {
        loop {
            for (name, path) in self.load_requests.recv() {
                println!("Loading font {}", name);
                let buffer = std::fs::read(path);
                if let Ok(buffer) = buffer {
                    println!("Buffer size: {}", buffer.len());
                    let result = self.loaded_fonts.send((name, buffer));
                    if let Err(_) = result {
                        println!("Failed to send a loaded font to the draw module");
                    } else {
                        println!("Sent font bytes to draw module");
                    }
                } else {
                    println!("Failed to load a font");
                }
            }
        }
    }
}
