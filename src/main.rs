use std::{io::{ Write, Result as IOResult }, fs::File, error::Error, slice::from_raw_parts};

#[derive(Default, Clone, Copy, Debug)]
struct Pixel(u8, u8, u8);

impl Pixel {
    pub const WHITE: Pixel = Pixel::gray(255);
    pub const BLACK: Pixel = Pixel::gray(0);

    const fn gray(v: u8) -> Self {
        Pixel( v, v, v )
    }

    fn as_slice(&self) -> &[u8] {
        unsafe {
            let this: *const u8 = &self.0;
            from_raw_parts(this, 3)
        }
    }
}

struct Canvas {
    width: usize,
    height: usize,
    data: Vec<Pixel>
}

impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        Canvas {
            width,
            height,
            data: vec![ Pixel::default(); width * height ]
        }
    }

    fn flat_pixels(&self) -> &[u8] {
        unsafe {
            let data: *const u8 = self.data.get(0).unwrap().as_slice().as_ptr();

            from_raw_parts(data, self.data.len() * 3)
        }
    }

    fn write_to(&self, file: &mut File) -> IOResult<()> {
        writeln!(file, "P6\n{} {}\n255", self.width, self.height)?;

        file.write_all( self.flat_pixels() )?;

        Ok(())
    }

    fn set(&mut self, x: usize, y: usize, p: Pixel) {
        self.data[y * self.width + x] = p;
    }

    fn get(&self, x: usize, y: usize) -> &Pixel {
        &self.data[y * self.width + x]
    }

    fn draw_circle(&mut self, x: f32, y: f32, radius: f32) {
        let uradius = radius.ceil() as usize;
        let ux = x as usize;
        let uy = y as usize;

        for cell_y in uy.saturating_sub(uradius)..=uy+uradius {
            for cell_x in ux.saturating_sub(uradius)..=ux+uradius {
                let fcell_x = cell_x as f32;
                let fcell_y = cell_y as f32;

                if (fcell_x - x)*(fcell_x - x) + (fcell_y - y)*(fcell_y - y) <= radius*radius {
                    self.set(cell_x, cell_y, Pixel::WHITE)
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {

    let mut canvas = Canvas::new(1920, 1080);

    canvas.draw_circle(100.0, 100.0, 101.0);


    let mut file = File::create("output.ppm")?;

    canvas.write_to(&mut file)?;

    Ok(())
}
