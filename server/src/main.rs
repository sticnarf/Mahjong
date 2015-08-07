extern crate rand;
extern crate time;

use std::process;
use std::process::*;
use std::io::*;
use std::thread;
use std::env;
use rand::*;
use std::sync::*;
use std::sync::mpsc::*;
use std::collections::*;
use std::cmp::*;
use time::*;

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
    hands: Vec<String>,
    chows: Vec<String>,
    pungs: Vec<String>,
    kongs: Vec<String>,
    ckongs: Vec<String>
}

struct Game {
    rng: StdRng,
    paths: [String; 4],
    stage: String,
    inputs: Vec<ChildStdin>,
    join_counter: HashSet<usize>,
    action_id: usize,
    tiles: Vec<Tiles>,
    left: Vec<String>,
    last_time: PreciseTime,
    last_tile: String,
    score: [i64; 4],
    messages: HashMap<usize, Message>
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
            tiles: Vec::new(),
            left: left,
            last_time: PreciseTime::now(),
            score: [0; 4],
            last_tile: String::new(),
            messages: HashMap::new()
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
        // BUG: 如果某AI在此时疯狂发指令，会导致后发指令的其它AI的指令超时而舍弃
        if self.stage == "outwait" {
            if self.last_time.to(PreciseTime::now()).num_milliseconds() < 550 {
                self.messages.insert(msg.id, msg);
                return;
            } else {
                self.outwait();
                return;
            }
        }
        let v: Vec<&str> = msg.message.split('_').collect();
        match v[0] {
            "join" => self.join(msg.id),
            "out" => self.out(msg.id, v[1].to_string()),
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
            self.tiles.push(Tiles {
                hands: Vec::new(),
                chows: Vec::new(),
                pungs: Vec::new(),
                kongs: Vec::new(),
                ckongs: Vec::new()
            });
            for _ in 0..13 {
                let tile = self.left.pop().unwrap();
                output.push_str(" ");
                output.push_str(&tile);
                self.tiles[i].hands.push(tile);
            }
            output.push_str("\n");
            self.inputs[i].write(output.as_bytes()).ok();
            self.inputs[i].flush().ok();
        }
        self.pick();
    }

    fn pick(&mut self) {
        if self.left.len() == 0 {
            self.draw();
        }
        let tile = self.left.pop().unwrap();
        self.tiles[self.action_id].hands.push(tile.clone());
        self.inputs[self.action_id].write(format!("pick {}\n", tile).to_string().as_bytes()).ok();
        self.inputs[self.action_id].flush().ok();
        for i in 0..4 {
            if i == self.action_id { continue; }
            self
            .inputs[i].write(format!("mpick {}\n", self.action_id).to_string().as_bytes()).ok();
            self.inputs[i].flush().ok();
        }
        self.stage = "out".to_string();
        self.last_tile = tile;
        self.last_time = PreciseTime::now();
    }

    fn out(&mut self, id: usize, tile: String) {
        if self.stage != "out" || id != self.action_id {
            return;
        }
        match self.tiles[id].hands.iter().position(|x| *x == tile) {
            Some(index) => {
                let duration = self.last_time.to(PreciseTime::now()).num_milliseconds();
                if duration >= 1050 {
                    self.score[id] -= (duration - 950) / 100;
                }
                self.tiles[id].hands.remove(index);
                for i in 0..4 {
                    if i == self.action_id { continue; }
                    self.inputs[i]
                        .write(format!("mout {} {}\n", self.action_id, tile)
                                   .to_string()
                                   .as_bytes())
                        .ok();
                    self.inputs[i].flush().ok();
                }
                self.last_tile = tile;
                self.stage = "outwait".to_string();
                self.last_time = PreciseTime::now();
            },
            None => {
                //TODO
            }
        }
    }

    fn outwait(&mut self) {
        let mut messages = Vec::new();
        for _msg in self.messages.values() {
            messages.push(_msg.clone());
        }
        messages.sort_by(|a, b| {
            let o1 = match a.message.split('_').next().unwrap() {
                "hu" => (a.id + 4 - self.action_id) % 4,
                "gang" => 16,
                "peng" => 64,
                "chi" => 254,
                _ => 255
            };
            let o2 = match b.message.split('_').next().unwrap() {
                "hu" => (b.id + 4 - self.action_id) % 4,
                "gang" => 16,
                "peng" => 64,
                "chi" => 254,
                _ => 255
            };
            o1.cmp(&o2)
        });
        let mut chi = false;
        for msg in messages {
            if msg.id == self.action_id { continue; }
            let v: Vec<&str> = msg.message.split('_').collect();
            match v[0] {
                "hu" => {
                    if self.hu(msg.id) {
                        return;
                    }
                },
                "gang" => {
                    if self.gang(msg.id, v[1]) {
                        break;
                    }
                },
                "peng" => {
                    if self.peng(msg.id, v[1]) {
                        break;
                    }
                },
                "chi" => {
                    if self.chi(msg.id, v[1]) {
                        chi = true;
                        break;
                    }
                },
                _ => ()
            }
        }
        if !chi {
            let post = post_pos(self.action_id);
            match self.messages.get(&post) {
                Some(msg) => {
                    if msg.message.split('_').next().unwrap() == "chi" {
                        self.inputs[post].write("mfail\n".to_string().as_bytes()).ok();
                        self.inputs[post].flush().ok();
                    }
                },
                None => ()
            }
        }
    }

    fn hu(&mut self, id: usize) -> bool {
        //TODO
        return false;
    }

    fn gang(&mut self, id: usize, tile: &str) -> bool {
        if self.tiles[id].hands.iter().filter(|&x| *x == tile).count() == 3 {
            self.tiles[id].hands.retain(|x| x != tile);
            self.tiles[id].kongs.push(tile.to_string());
            return true;
        }
        return false;
    }

    fn peng(&mut self, id: usize, tile: &str) -> bool {
        if self.tiles[id].hands.iter().filter(|&x| *x == tile).count() >= 2 {
            for _ in 0..2 {
                let index = self.tiles[id].hands.iter().position(|x| *x == tile).unwrap();
                self.tiles[id].hands.remove(index);
            }
            return true;
        }
        return false;
    }

    fn chi(&mut self, id: usize, tile: &str) -> bool {
        let mut set = HashSet::new();
        set.insert(tile.to_string());
        let second = match post(tile.to_string()) {
            Some(x) => x,
            None => return false
        };
        set.insert(second.clone());
        set.insert(match post(second) {
            Some(x) => x,
            None => return false
        });
        set.remove(&self.last_tile);
        if self.tiles[id].hands.iter().filter(|&x| set.contains(x)).count() >= 2{
            for i in set {
                let index = self.tiles[id].hands.iter().position(|x| *x == i).unwrap();
                self.tiles[id].hands.remove(index);
            }
            return true;
        }
        return false;
    }

    fn draw(&mut self) {
        //TODO
    }
}

#[derive(Clone)]
struct Message {
    id: usize,
    message: String
}

fn post(tile: String) -> Option<String> {
    let chars: Vec<char> = tile.chars().collect();
    if tile.len() == 2 {
        let kind = chars[0];
        if kind == 'M' || kind == 'T' || kind == 'S' {
            let num = match chars[1].to_digit(10) {
                Some(x) => x,
                None => {
                    return None;
                }
            };
            if num < 9 && num > 0 {
                return Some(format!("{}{}", kind, num + 1));
            }
        }
    }
    return None;
}

fn post_pos(pos: usize) -> usize {
    return (pos + 1) % 4;
}