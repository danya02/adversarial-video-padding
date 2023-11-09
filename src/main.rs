use std::{io::Write, path::PathBuf, process::Stdio, time::Instant};

use ndarray::Array3;

use video_rs::{Encoder, EncoderSettings, Locator, Time};

fn main() {
    let mut process = std::process::Command::new("ffmpeg")
        .arg("-f")
        .arg("rawvideo")
        .arg("-pixel_format")
        .arg("rgb24")
        .arg("-video_size")
        .arg("800x600")
        .arg("-i")
        .arg("-")
        .arg("output.mp4")
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    let sub = process.stdin.take().unwrap();
    let mut sub = std::io::BufWriter::new(sub);
    let a = Instant::now();

    for _ in 0..2000 {
        let mut screen = Screen::new();
        for row in screen.0.iter_mut() {
            for pix in row.0.iter_mut() {
                pix.0[0] = 128;
                pix.0[0] = 0;
                pix.0[0] = 255;
            }
        }
        let screen_buf: [u8; 800 * 600 * 3] = unsafe { std::mem::transmute(screen) };
        sub.write(&screen_buf).unwrap();
        //sub.flush().unwrap();
    }
    sub.flush().unwrap();
    drop(sub);

    let d = Instant::now().duration_since(a);
    eprintln!("{d:?}");
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Screen([Row; 600]);

impl Screen {
    pub fn new() -> Self {
        Self([Row::new(); 600])
    }
}

#[repr(C)]
#[derive(Clone, Copy)]

struct Row([Pixel; 800]);

impl Row {
    pub fn new() -> Self {
        Self([Pixel::new(); 800])
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Pixel([u8; 3]);

impl Pixel {
    pub fn new() -> Self {
        Self([0; 3])
    }
}
