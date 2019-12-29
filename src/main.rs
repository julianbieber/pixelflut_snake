use std::io::prelude::*;
use std::net::{TcpStream, UdpSocket};
use std::error::Error;
use structopt::StructOpt;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::time::{Duration, Instant};
use std::sync::{Mutex, Arc};
use std::io::BufReader;
use std::thread;

#[derive(Debug, StructOpt)]
struct Config {
    #[structopt(short)]
    address: String
}

struct Pixel{
    x: i32,
    y: i32,
    color: String,
}

struct Snake {
    direction: (i32, i32),
    points: Vec<(i32, i32)>,
    size: i32,
    color: String,
}

impl Snake {
    fn pixels(&self) -> Vec<Pixel> {
        let mut result = Vec::new();
        for (p_x, p_y) in self.points.iter() {
            for x in *p_x - self.size/2 .. p_x + self.size/2 {
                for y in *p_y - self.size/2 .. p_y + self.size/2 {
                    result.push(Pixel {
                        x,
                        y,
                        color: self.color.clone()
                    });
                }
            }
        }
        result
    }

    fn right(&mut self) {
        self.direction = (1, 0);
    }

    fn left(&mut self){
        self.direction = (-1, 0);
    }

    fn up(&mut self) {
        self.direction = (0, -1);
    }

    fn down(&mut self) {
        self.direction = (0, 1);
    }

    fn run(&mut self) {
        let mut snake_head = {
            let l = self.points.last().unwrap_or(&(0,0));
            (l.0, l.1)
        };
        snake_head.0 += self.direction.0;
        snake_head.1 += self.direction.1;

        self.points.push(snake_head);
        self.points.remove(0);
    }

    fn add_tail(&mut self) {
        let mut tail = {
            let mut l = self.points[0];
            l.0 -= self.direction.0;
            l.1 -= self.direction.1;
            vec![l]
        };
        for (x, y) in self.points.iter() {
            tail.push((*x, *y));
        }

        self.points = tail;
    }
}

impl Display for Pixel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(format!("PX {} {} {}\n", self.x, self.y, self.color).as_str())
    }
}

fn write_pixels(stream: &mut TcpStream, pixels: &Vec<Pixel>) -> Result<(), Box<dyn Error>> {
    let message: String = pixels.iter().map(|pixel| format!("{}", pixel)).collect::<Vec<String>>().join("");
    stream.write_all(message.as_bytes())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let config: Config = Config::from_args();
    let snake = Arc::new(Mutex::new(Snake {
        direction: (0, 0),
        points: vec![(50, 50), (100, 100)],
        color: "ffffff".to_string(),
        size: 10
    }));
    let mut draw_threads = Vec::new();

    for _ in 0 .. 10 {
        let address = config.address.clone();
        let snake_clone = snake.clone();
        draw_threads.push(thread::spawn(move || {
            draw(address.as_str(), snake_clone)
        }))
    }

    let server_snake = snake.clone();
    let server_thread = thread::spawn(move || {
        handle_inputs(server_snake)
    });


    for t in draw_threads.into_iter() {
        t.join();
    }

    Ok(())
}

fn snake_movement(address: &str, snake_mutex: Arc<Mutex<Snake>>) {
    let mut stream = TcpStream::connect(address).unwrap();
}

fn get_pixel(stream: &mut TcpStream, x: i32, y: i32) -> Result<Pixel, Box<dyn Error>> {

    Ok(Pixel {
        x,
        y,
        color: "".to_string()
    })
}

fn draw(address: &str, snake_mutex: Arc<Mutex<Snake>>) {
    let mut stream = TcpStream::connect(address).unwrap();
    let mut pixles =
        snake_mutex.lock().map(|snake| {
            (snake).pixels()
        }).unwrap();

    let ten_ms = Duration::from_millis(10);
    let start = Instant::now();
    loop {
        write_pixels(&mut stream, &pixles);
        if start.elapsed() > ten_ms {
           pixles = snake_mutex.lock().map(|snake| {
             (snake).pixels()
           }).unwrap()
        }
    }
}

fn handle_inputs(snake_mutex: Arc<Mutex<Snake>>) {
    let socket = UdpSocket::bind("127.0.0.1:34254").unwrap();
    let mut byte = [0];
    loop {
        socket.recv(&mut byte).map(|received_size| {
            if received_size > 0 {
                let character = byte[0] as char;
                snake_mutex.lock().map(|mut snake| {
                    match character {
                        'w' => snake.up(),
                        's' => snake.down(),
                        'a' => snake.left(),
                        'd' => snake.right(),
                        _ => ()
                    }
                });

            }
        });
    }
}
