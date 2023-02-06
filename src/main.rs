use std::collections::HashMap;

use piston_window::Button;
use piston_window::ButtonArgs;
use piston_window::ButtonState;
use piston_window::Event;
use piston_window::DrawState;
use piston_window::Input;
use piston_window::Key;
use piston_window::Motion;
use piston_window::MouseButton;
use piston_window::PistonWindow;
use piston_window::WindowSettings;
use voronoice::{Point, VoronoiBuilder, Voronoi, BoundingBox};

use image::ImageBuffer;
use image::{Rgb, Rgba};
use rand::{Rng, random};

struct VoronoiImage<T>
where T: Into<Rgba<u8>> + Clone {
    diagram: Voronoi,
    colors: Vec<T>,
    c: Option<VoronoiCache>
}

struct VoronoiCache {
    v: HashMap<(u32, u32), usize>
}

impl<T> VoronoiImage<T>
where T: Into<Rgba<u8>> + Clone,
Vec<T>: FromIterator<Rgba<u8>> {
    fn new(voronoi_builder: VoronoiBuilder, colors: Vec<T>) ->  Result<Self, &'static str> {
        println!("Got here!");
        let mut r = Err("Unknown error");
        if let Some(diagram) = voronoi_builder.build() { 
            println!("Got here too!");
            if diagram.sites().len() == colors.len() {
                r = Ok(Self { diagram, colors, c: None });
            } else {
                r = Err("The number of colors does not match the number of sites.");
            }
        }
        println!("Got there!");
        r
    }

    fn random(number_of_points: usize, lloyd_relaxation_iterations: usize, width: u32, height: u32) -> Result<Self, &'static str> {
        Self::new(VoronoiBuilder::default()
            .set_sites(random_points(width, height, number_of_points))
            .set_clip_behavior(voronoice::ClipBehavior::None)
            .set_bounding_box(BoundingBox::new(Point { x: width as f64 / 2.0, y: height as f64 / 2.0 }, width as f64, height as f64))
            .set_lloyd_relaxation_iterations(lloyd_relaxation_iterations),
            (0..number_of_points).map(|_| Rgba([20, 80, 240, 255])).collect()
        )
        
    }

    fn draw(&mut self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut image_buffer = ImageBuffer::new(self.width(), self.height());
        if let Some(cch) = &self.c {
            for x in 0..self.width() {
                for y in 0..self.height() {
                    image_buffer.put_pixel(x, y, self.colors[*cch.v.get(&(x, y)).unwrap()].clone().into());
                }
            }
        } else {
            let mut cache = VoronoiCache { v: HashMap::new() };
            for x in 0..self.width() {
                for y in 0..self.height() {
                    let col = self.get_pixel(x, y).into();
                    cache.v.insert((x, y), *&col);
                    image_buffer.put_pixel(x, y, self.colors[col].clone().into());
                }
            }
            self.c = Some(cache);
        }
        

        image_buffer
    }


    fn width(&self) -> u32 {
        self.diagram.bounding_box().width().round() as u32
    }

    fn height (&self) -> u32 {
        self.diagram.bounding_box().height().round() as u32
    }

    fn get_pixel(&self, x: impl Into<f64>, y: impl Into<f64>) -> usize {
        self.diagram.cell(0).iter_path(Point { x: x.into(), y: y.into()}).last().unwrap()
    }
}

fn random_points(width: u32, height: u32, number_of_points: usize) -> Vec<Point> {
    let mut rng = rand::thread_rng();
    let x_range = rand::distributions::Uniform::new(0, width);
    let y_range = rand::distributions::Uniform::new(0, height);
    let mut points = vec![];

    for _ in 0..number_of_points {
        let mut p = Point {x: rng.sample(x_range) as f64, y: rng.sample(y_range) as f64};
        while points.contains(&p) {
            p = Point {x: rng.sample(x_range) as f64, y: rng.sample(y_range) as f64};
        }
        points.push(p);
    }

    points
}

fn main() {
    let result = std::panic::catch_unwind(|| {
        test();
        print!("{}[2J", 27 as char);
    });
    test()
}

fn test() {

    let width = 800;
    let height = 600;
    let number_of_points = 3000;
    let lloyd_relaxation_iterations = 15; 
    let mut v = VoronoiImage::random(number_of_points, lloyd_relaxation_iterations, width, height).unwrap();
    let t = chrono::Local::now();
    let img = v.draw();
    println!("Draw duration (without cache): {}ms", (chrono::Local::now() - t).num_milliseconds());

    let mut window: PistonWindow = WindowSettings::new("TEST", [width, height])
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    let mut ctx = window.create_texture_context();
    let image = piston_window::Image::new();
    let mut mouse_pos = [0., 0.];
    let mut texture = piston_window::Texture::from_image(&mut ctx, &img, &piston_window::TextureSettings::new()).unwrap();
    let mut last = chrono::Local::now();
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g, _| {
            image.draw(&texture, &DrawState::default(), c.transform, g);
        });
        if let Event::Input(inp, _) = e {
            match inp {
                Input::Move(mot) => {
                    if let Motion::MouseCursor(cur) = mot {
                        mouse_pos = cur;
                    }
                },
                Input::Button(but) => {
                    match but.button { Button::Mouse(mouse_but) =>  {
                        if let MouseButton::Left = mouse_but { if let ButtonState::Release = but.state { if (chrono::Local::now() - last).num_milliseconds() > 300 {
                            last = chrono::Local::now();
                            println!("Position of the mouse: {:?}", mouse_pos);
                            let id = v.diagram.cell(0).iter_path(Point { x: mouse_pos[0], y: mouse_pos[1]}).last().unwrap();
                            if v.colors[id] == Rgba([20, 80, 240, 255]) {
                                v.colors[id] = Rgba([20, 240, 80, 255]);
                            } else {
                                v.colors[id] = Rgba([20, 80, 240, 255]);
                            }
                            let t = chrono::Local::now();
                            let img = v.draw();
                            println!("Draw duration (with cache): {}ms", (chrono::Local::now() - t).num_milliseconds());
                            texture = piston_window::Texture::from_image(&mut ctx, &img, &piston_window::TextureSettings::new()).unwrap();
                        }}}
                    }, 
                    Button::Keyboard(k) => {
                        if let Key::Return = k {if let ButtonState::Release = but.state {
                            v.draw().save("result/CURRENT.png").unwrap();
                        }}
                    }
                    _ => {}}
                },
                _ => {}
            }
        }
    }
}

