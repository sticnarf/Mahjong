extern crate rand;

use std::process;
use std::process::*;
use std::io::*;
use std::thread;
use std::env;
use rand::*;
use std::sync::*;
use std::sync::mpsc::*;
use std::collections::*;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 5 {
        println!("{}", args[0]);
        println!("Usage: ./server <AI1> <AI2> <AI3> <AI4>");
        process::exit(1);
    }
    let mut board: HashMap<&str, i32> = HashMap::new();
    for i in 0..4 {
        board.insert(&args[i], 0);
    }
    let positions = vec![
        [[1, 2, 3, 4], [2, 3, 4, 1], [3, 4, 1, 2], [4, 1, 2, 3]],
        [[1, 3, 2, 4], [3, 2, 4, 1], [2, 4, 1, 3], [4, 1, 3, 2]],
        [[1, 3, 4, 2], [3, 4, 2, 1], [4, 2, 1, 3], [2, 1, 3, 4]],
        [[1, 4, 2, 3], [4, 2, 3, 1], [2, 3, 1, 4], [3, 1, 4, 2]],
        [[1, 4, 3, 2], [4, 3, 2, 1], [3, 2, 1, 4], [2, 1, 4, 3]],
        [[1, 2, 4, 3], [2, 4, 3, 1], [4, 3, 1, 2], [3, 1, 2, 4]],
    ];
    for group in &positions {
        let seed = gen_seed();
        for position in group {
            let rng = StdRng::from_seed(&seed);
            let paths = [
                args[position[0]].clone(), args[position[1]].clone(), args[position[2]].clone(),
                args[position[3]].clone()
            ];
            let mut game = Game::new(paths, rng);
            game.run();
        }
    }
}

fn gen_seed() -> [usize; 8] {
    let mut seed = [0; 8];
    for i in 0..8 {
        seed[i] = usize::rand(&mut rand::thread_rng());
    }
    seed
}

struct Tiles {
    v: Vec<String>
}

struct Game {
    rng: StdRng,
    paths: [String; 4],
    stage: String,
    inputs: Vec<ChildStdin>,
    join_counter: HashSet<usize>,
    action_id: usize,
    inhand: Vec<Tiles>,
    left: Vec<String>
}

impl Game {
    fn new(paths: [String; 4], rng: StdRng) -> Game {
        let tiles = [
            "1M", "2M", "3M", "4M", "5M", "6M", "7M", "8M", "9M", "1S", "2S", "3S", "4S", "5S",
            "6S", "7S", "8S", "9S", "1T", "2T", "3T", "4T", "5T", "6T", "7T", "8T", "9T", "E", "S",
            "W", "N", "Z", "F", "B", "1M", "2M", "3M", "4M", "5M", "6M", "7M", "8M", "9M", "1S",
            "2S", "3S", "4S", "5S", "6S", "7S", "8S", "9S", "1T", "2T", "3T", "4T", "5T", "6T",
            "7T", "8T", "9T", "E", "S", "W", "N", "Z", "F", "B", "1M", "2M", "3M", "4M", "5M",
            "6M", "7M", "8M", "9M", "1S", "2S", "3S", "4S", "5S", "6S", "7S", "8S", "9S", "1T",
            "2T", "3T", "4T", "5T", "6T", "7T", "8T", "9T", "E", "S", "W", "N", "Z", "F", "B",
            "1M", "2M", "3M", "4M", "5M", "6M", "7M", "8M", "9M", "1S", "2S", "3S", "4S", "5S",
            "6S", "7S", "8S", "9S", "1T", "2T", "3T", "4T", "5T", "6T", "7T", "8T", "9T", "E", "S",
            "W", "N", "Z", "F", "B"
        ];
        let mut left: Vec<_> = tiles.iter().map(|x| x.to_string()).collect();
        rng.clone().shuffle(&mut left);
        Game {
            rng: rng,
            paths: paths,
            stage: "join".to_string(),
            inputs: Vec::new(),
            join_counter: HashSet::new(),
            action_id: 0,
            inhand: Vec::new(),
            left: left
        }
    }

    fn run(&mut self) {
        let (tx, rx) = mpsc::channel();
        for i in 0..4 {
            let command = Command::new(&self.paths[i])
                              .stdin(Stdio::piped())
                              .stdout(Stdio::piped())
                              .spawn()
                              .unwrap();
            self.inputs.push(command.stdin.unwrap());
            let tx = tx.clone();
            let mut output = BufReader::new(command.stdout.unwrap());
            thread::spawn(move || {
                loop {
                    let mut result = String::new();
                    let size = output.read_line(&mut result).ok().unwrap();
                    if size > 0 {
                        tx.send(Message {
                            id: i,
                            message: result
                        }).ok();
                    }
                    thread::sleep_ms(10);
                }
            });
        }
        loop {
            let msg = rx.recv().ok().unwrap();
            self.process(msg);
        }
    }

    fn process(&mut self, msg: Message) {
        let v: Vec<&str> = msg.message.split('_').collect();
        match v[0] {
            "join" => self.join(msg.id),
            _ => ()
        }
    }

    fn join(&mut self, id: usize) {
        if self.stage != "join" {
            return;
        }
        self.join_counter.insert(id);
        if self.join_counter.len() == 4 {
            for i in 0..4 {
                self.inputs[i].write(format!("id {}\n", i).as_bytes()).ok();
                self.inputs[i].flush().ok();
            }
            self.start();
        }
    }

    fn start(&mut self) {
        self.action_id = self.rng.gen_range(0, 4);
        for i in 0..4 {
            self.inputs[i].write(format!("first {}\n", i).as_bytes()).ok();
            self.inputs[i].flush().ok();
        }
        self.init();
    }

    fn init(&mut self) {
        for i in 0..4 {
            let mut output = "init".to_string();
            self.inhand.push(Tiles {
                v: Vec::new()
            });
            for _ in 0..13 {
                let tile = self.left.pop().unwrap();
                output.push_str(" ");
                output.push_str(&tile);
                self.inhand[i].v.push(tile);
            }
            output.push_str("\n");
            self.inputs[i].write(output.as_bytes()).ok();
            self.inputs[i].flush().ok();
        }
        self.pick();
    }

    fn pick(&mut self) {
        if self.left.len() == 0{
            self.draw();
        }
        let tile = self.left.pop().unwrap();
        self.inputs[self.action_id].write(format!("pick {}\n", tile).to_string().as_bytes()).ok();
        self.inputs[self.action_id].flush().ok();
        for i in 0..4 {
            if i != self.action_id {
                self.inputs[i].write(format!("mpick {}\n",self.action_id).to_string().as_bytes()).ok();
                self.inputs[i].flush().ok();
            }
        }
        self.stage = "out".to_string();
    }

    fn draw(&mut self){
        //TODO
    }
}

struct Message {
    id: usize,
    message: String
}