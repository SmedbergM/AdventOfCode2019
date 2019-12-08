use std::fmt;


struct Layer {
    width: usize,
    pixels: Vec<u8>
}

impl Layer {
    fn from_bytes(cs: &[u8], width: usize) -> Layer {
        let mut pixels = Vec::new();
        pixels.extend_from_slice(cs);
        Layer { width, pixels }
    }

    fn count(&self, v: char) -> usize {
        self.pixels.iter().fold(0, |acc, x| {
            acc + ((*x == (v as u8)) as usize)
        })
    }

    fn get(&self, x: usize, y: usize) -> char {
        self.pixels[y * self.width + x] as char
    }
}

struct Image {
    height: usize,
    width: usize,
    layers: Vec<Layer>
}

impl Image {
    fn from_str(line: &str, height: usize, width: usize) -> Image {
        let layer_length = height*width;
        let layers = (0..(line.len())).step_by(layer_length).map(|offset| {
            Layer::from_bytes(&line.as_bytes()[offset..(offset + layer_length)], width)
        }).collect();

        Image { height, width, layers }
    }

    fn checksum(&self) -> usize {
        let (opt_min_0_layer, _) = self.layers.iter().fold((None, usize::max_value()), |(acc, min_zeros), layer| {
            let chk = layer.count('0');
            if chk < min_zeros {
                (Some(layer), chk)
            } else {
                (acc, min_zeros)
            }
        });
        let min_0_layer = opt_min_0_layer.unwrap();

        min_0_layer.count('1') * min_0_layer.count('2')
    }
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut display = String::with_capacity(self.height * (self.width + 1));
        let mut reversed = String::with_capacity(self.height * (self.width + 1));
        for y in 0..self.height {
            for x in 0..self.width {
                for layer in &self.layers {
                    match layer.get(x,y) {
                        '2' => (),
                        '1' => {
                            display.push('*');
                            reversed.push(' ');
                            break
                        },
                        '0' => {
                            display.push(' ');
                            reversed.push('*');
                            break
                        },
                        z => {
                            display.push(z as char);
                            reversed.push(z as char);
                            break
                        }
                    }
                }
            };
            display.push('\n');
            reversed.push('\n');
        }
        write!(f, "Image:\n{}\nReversed:\n{}", display, reversed)
    }
}

fn main() {
    let puzzle = util::read_single_line_from_stdin().unwrap();
    let width = 25;
    let height = 6;
    let image = Image::from_str(&puzzle, height, width);
    let chk = image.checksum();
    println!("Image checksum: {}", &chk);

    println!("{}", image)
}
