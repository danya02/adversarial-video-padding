#![feature(generic_const_exprs)]
use std::{
    convert::Infallible,
    io::{BufWriter, Write},
    process::{ChildStdin, Stdio},
    time::Instant,
};

use boids::Boid2D;
use cgmath::Vector2;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{Dimensions, DrawTarget, OriginDimensions, RgbColor},
    primitives::Line,
    Pixel,
};

use embedded_graphics::{
    mock_display::MockDisplay,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{
        Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
};
use rand::Rng;

fn main() -> Result<(), Infallible> {
    let mut process = std::process::Command::new("ffmpeg")
        .arg("-y")
        .arg("-pixel_format")
        .arg("rgb24")
        .arg("-f")
        .arg("rawvideo")
        .arg("-pix_fmt")
        .arg("rgb24")
        .arg("-video_size")
        .arg(&format!("{WIDTH}x{HEIGHT}"))
        .arg("-i")
        .arg("-")
        .arg("output.mp4")
        .stdin(Stdio::piped())
        //.stderr(Stdio::null())
        .spawn()
        .unwrap();
    let sub = process.stdin.take().unwrap();
    let mut sub = std::io::BufWriter::new(sub);
    let start = Instant::now();

    let mut display = FfmpegDisplay::new(sub);

    let mut rng = rand::thread_rng();

    let mut flock = boids::flock::Flock {
        boids: vec![],
        goal_alignment: 50.0,
        goal_cohesion: 50.0,
        goal_separation: 25.0,
        target: None,
    };

    for _ in 0..250 {
        flock.boids.push(Boid2D::new(Vector2 {
            x: rng.gen_range(0..WIDTH) as f32,
            y: rng.gen_range(0..HEIGHT) as f32,
        }))
    }

    let mut a = Point::new(0, 0);
    let mut b = Point::new(WIDTH as i32, 0);
    let mut c = Point::new(0, HEIGHT as i32);

    let boid_style = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb888::new(0, 255, 0))
        .stroke_width(1)
        .fill_color(Rgb888::new(0, 255, 0))
        .build();

    let boid_velocity = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb888::new(255, 0, 0))
        .stroke_width(2)
        .build();

    for _ in 0..1000 {
        flock.update();
        // Teleport any boid that is out of bounds
        for boid in &mut flock.boids {
            if boid.position.x < 0.0 {
                boid.position.x = WIDTH as f32;
            } else if boid.position.x > WIDTH as f32 {
                boid.position.x = 0.0;
            }
            if boid.position.y < 0.0 {
                boid.position.y = HEIGHT as f32;
            } else if boid.position.y > HEIGHT as f32 {
                boid.position.y = 0.0;
            }
        }

        for boid in flock.boids.iter() {
            let pt = Point::new(boid.position.x as i32, boid.position.y as i32);
            let scale = 0.1;
            let vel = pt
                + Point::new(
                    (boid.velocity.x / scale) as i32,
                    (boid.velocity.y / scale) as i32,
                );
            Rectangle::with_center(pt, Size::new(10, 10))
                .into_styled(boid_style)
                .draw(&mut display)?;
            Line::new(pt, vel)
                .into_styled(boid_velocity)
                .draw(&mut display)
                .unwrap();
        }

        display.write();
        a.x += rng.gen_range(-10..=10);
        a.y += rng.gen_range(-10..=10);
        b.x += rng.gen_range(-10..=10);
        b.y += rng.gen_range(-10..=10);
        c.x += rng.gen_range(-10..=10);
        c.y += rng.gen_range(-10..=10);
    }

    let d = Instant::now().duration_since(start);
    eprintln!("{d:?}");

    Ok(())
}

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

pub struct FfmpegDisplay {
    stdin: BufWriter<ChildStdin>,
    framebuffer: [u8; WIDTH as usize * HEIGHT as usize * 3],
}

impl FfmpegDisplay {
    pub fn new(w: BufWriter<ChildStdin>) -> Self {
        Self {
            stdin: w,
            framebuffer: [0; WIDTH as usize * HEIGHT as usize * 3],
        }
    }

    pub fn write(&mut self) {
        self.stdin.write_all(&self.framebuffer).unwrap();
        self.stdin.flush().unwrap();
        self.framebuffer.fill(0);
    }
}

impl OriginDimensions for FfmpegDisplay {
    fn size(&self) -> embedded_graphics::prelude::Size {
        embedded_graphics::prelude::Size {
            width: WIDTH,
            height: HEIGHT,
        }
    }
}

impl DrawTarget for FfmpegDisplay {
    type Color = Rgb888;

    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        let h = HEIGHT as usize - 1;
        let w = WIDTH as usize - 1;
        for Pixel(coord, color) in pixels.into_iter() {
            let x = if (0..w).contains(&(coord.x as usize)) {
                coord.x as usize
            } else {
                continue;
            };
            let y = if (0..h).contains(&(coord.y as usize)) {
                coord.y as usize
            } else {
                continue;
            };

            let index = ((y * WIDTH as usize) + x) * 3;
            self.framebuffer[index] = color.r();
            self.framebuffer[index + 1] = color.g();
            self.framebuffer[index + 2] = color.b();
        }

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let mut idx = 0;
        let colors = [color.r(), color.g(), color.b()];
        for pix in self.framebuffer.iter_mut() {
            *pix = colors[idx];
            idx += 1;
            if idx == 3 {
                idx = 0;
            }
        }
        Ok(())
    }
}
