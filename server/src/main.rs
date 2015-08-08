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
    let mut board: HashMap<String, i64> = HashMap::new();
    for i in 0..4 {
        board.insert(args[i].to_string(), 0);
    }
    println!("Game start!");
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
        println!("Generated new random seeds");
        for position in group {
            println!("The positions are {:?}", position);
            let rng = StdRng::from_seed(&seed);
            let paths = [
                args[position[0]].clone(), args[position[1]].clone(), args[position[2]].clone(),
                args[position[3]].clone()
            ];
            let mut game = Game::new(paths, rng);
            game.run();
            println!("This hand's score: {:?}", game.score);
            let _board = board.clone();
            for i in 0..4 {
                let score = _board.get(&args[position[i]].to_string()).unwrap();
                board.insert(args[position[i]].to_string(), score + game.score[i]);
            }
        }
    }
    println!("Final score: {:?}", board);
}

fn gen_seed() -> [usize; 8] {
    let mut seed = [0; 8];
    for i in 0..8 {
        seed[i] = usize::rand(&mut rand::thread_rng());
    }
    seed
}

#[derive(Clone)]
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
    messages: HashMap<usize, Message>,
    base: i64,
}

static mut flags: [bool; 4] = [true, true, true, true];

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
        unsafe {
            flags = [true, true, true, true];
        }
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
            messages: HashMap::new(),
            base: 4
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
                unsafe {
                    println!("AI{} started", i);
                    while flags[i] {
                        let mut result = String::new();
                        output.read_line(&mut result).ok();
                        tx.send(Message { id: i, message: result }).ok();
                        thread::sleep_ms(10);
                    }
                    println!("AI{} shut", i);
                }
            });
        }
        loop {
            let msg = match rx.recv() {
                Ok(msg) => msg,
                Err(_) => return
            };
            self.process(msg);
        }
    }

    fn process(&mut self, msg: Message) {
        let valid = msg.message.len() > 0;
        if valid {
            println!("Received message: {:?}", msg);
        }
        // BUG: 如果某AI在此时疯狂发指令，会导致后发指令的其它AI的指令超时而舍弃
        if self.stage == "outwait" {
            if self.last_time.to(PreciseTime::now()).num_milliseconds() < 550 {
                if valid {
                    self.messages.insert(msg.id, msg);
                    println!("Added to queue!");
                }
                return;
            } else {
                self.outwait();
                return;
            }
        }
        if self.stage == "qgwait" {
            if self.last_time.to(PreciseTime::now()).num_milliseconds() < 550 {
                if valid {
                    self.messages.insert(msg.id, msg);
                    println!("Added to queue!");
                }
                return;
            } else {
                self.qgwait();
                return;
            }
        }
        if !valid {
            return;
        }
        let v: Vec<&str> = msg.message.split('_').collect();
        match v[0].trim() {
            "join" => self.join(msg.id),
            "out" => self.out(msg.id, v[1].trim().to_string()),
            "agang" => self.agang(msg.id, v[1].trim().to_string()),
            "jgang" => self.jgang(msg.id, v[1].trim().to_string()),
            "hu" => self.tsumo(msg.id),
            _ => ()
        }
    }

    fn tsumo(&mut self, id: usize) {
        if self.stage != "out" || id != self.action_id {
            println!("{} sent invalid hu", id);
            return;
        }
        let time = PreciseTime::now();
        match cal_fan(self.tiles[id].clone()) {
            Some(x) => {
                let duration = self.last_time.to(time).num_milliseconds();
                if duration >= 1050 {
                    let penalty = (duration - 950) / 100;
                    self.score[id] -= penalty;
                    println!("{} was fined {} due to timeout", id, penalty);
                }
                self.score[id] += 3 * (x + self.base);
                for i in 0..4 {
                    if i != id {
                        self.score[i] -= x + self.base;
                    }
                }
            },
            None => {
                //TODO
            }
        }
    }

    fn qgwait(&mut self) {
        let mut messages = Vec::new();
        for _msg in self.messages.values() {
            messages.push(_msg.clone());
        }
        messages.sort_by(|a, b| ((a.id + 4 - self.action_id) % 4)
                                    .cmp(&((b.id + 4 - self.action_id) % 4)));
        for msg in messages {
            if msg.id == self.action_id { continue; }
            match msg.message.trim() {
                "qgang" => {
                    if self.hu(msg.id) {
                        println!("{} robbed the kong", msg.id);
                        return;
                    }
                },
                _ => ()
            }
        }
        self.pick();
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
        let mut sort = "fail";
        for msg in messages {
            if msg.id == self.action_id { continue; }
            let v: Vec<&str> = msg.message.split('_').collect();
            match v[0].trim() {
                "hu" => {
                    if self.hu(msg.id) {
                        return;
                    }
                },
                "gang" => {
                    if self.gang(msg.id, v[1].trim()) {
                        for i in 0..4 {
                            self.inputs[i]
                                .write(format!("mgang {} {}\n", msg.id, v[1].trim())
                                           .to_string()
                                           .as_bytes())
                                .ok();
                            self.inputs[i].flush().ok();
                        }
                        self.action_id = msg.id;
                        sort = "gang";
                        break;
                    }
                },
                "peng" => {
                    if self.peng(msg.id, v[1].trim()) {
                        for i in 0..4 {
                            self.inputs[i]
                                .write(format!("mpeng {} {}\n", msg.id, v[1].trim())
                                           .to_string()
                                           .as_bytes())
                                .ok();
                            self.inputs[i].flush().ok();
                        }
                        self.action_id = msg.id;
                        sort = "peng";
                        break;
                    }
                },
                "chi" => {
                    if self.chi(msg.id, v[1].trim()) {
                        for i in 0..4 {
                            self.inputs[i]
                                .write(format!("mchi {} {}\n", msg.id, v[1].trim())
                                           .to_string()
                                           .as_bytes())
                                .ok();
                            self.inputs[i].flush().ok();
                        }
                        self.action_id = msg.id;
                        sort = "chi";
                        break;
                    }
                },
                _ => ()
            }
        }
        if sort != "chi" {
            let post = post_pos(self.action_id);
            match self.messages.get(&post) {
                Some(msg) => {
                    if msg.message.split('_').next().unwrap() == "chi" {
                        println!("{} failed to chi", post);
                        self.inputs[post].write("mfail\n".to_string().as_bytes()).ok();
                        self.inputs[post].flush().ok();
                    }
                },
                None => ()
            }
        }
        if sort == "gang" {
            self.pick();
        } else if sort == "fail" {
            self.action_id = post_pos(self.action_id);
            self.pick();
        } else {
            self.stage = "out".to_string();
        }
    }

    fn agang(&mut self, id: usize, tile: String) {
        if self.stage != "out" || id != self.action_id {
            println!("{} sent invalid agang", id);
            return;
        }
        if self.tiles[id].hands.iter().filter(|&x| *x == tile).count() == 4 {
            let duration = self.last_time.to(PreciseTime::now()).num_milliseconds();
            if duration >= 1050 {
                let penalty = (duration - 950) / 100;
                self.score[id] -= penalty;
                println!("{} was fined {} due to timeout", id, penalty);
            }
            self.tiles[id].hands.retain(|x| x != &tile);
            self.tiles[id].ckongs.push(tile.to_string());
            println!("{} gang {} concealedly", id, tile);
            for i in 0..4 {
                self.inputs[i].write(format!("magang {}\n", id).to_string().as_bytes()).ok();
                self.inputs[i].flush().ok();
            }
            self.pick();
        }
    }

    fn jgang(&mut self, id: usize, tile: String) {
        if self.stage != "out" || id != self.action_id {
            return;
        }
        if self.tiles[id].hands.contains(&tile) && self.tiles[id].pungs.contains(&tile) {
            let duration = self.last_time.to(PreciseTime::now()).num_milliseconds();
            if duration >= 1050 {
                let penalty = (duration - 950) / 100;
                self.score[id] -= penalty;
                println!("{} was fined {} due to timeout", id, penalty);
            }
            let index = self.tiles[id].hands.iter().position(|x| *x == tile).unwrap();
            self.tiles[id].hands.remove(index);
            let index = self.tiles[id].pungs.iter().position(|x| *x == tile).unwrap();
            self.tiles[id].pungs.remove(index);
            self.tiles[id].kongs.push(tile.clone());
            println!("{} gang {} by adding", id, tile);
            for i in 0..4 {
                self
                .inputs[i].write(format!("mjgang {} {}\n", id, tile).to_string().as_bytes()).ok();
                self.inputs[i].flush().ok();
            }
            self.stage = "qgwait".to_string();
            self.last_tile = tile;
            self.messages.clear();
            println!("Waiting for action");
            self.last_time = PreciseTime::now();
        }
    }

    fn join(&mut self, id: usize) {
        if self.stage != "join" {
            println!("{} sent invalid join message", id);
            return;
        }
        self.join_counter.insert(id);
        println!("{} joined", id);
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
        println!("{} acts first", self.action_id);
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
            print!("{}'s first 13 tiles are:", i);
            for _ in 0..13 {
                let tile = self.left.pop().unwrap();
                output.push_str(" ");
                output.push_str(&tile);
                print!("{} ", tile);
                self.tiles[i].hands.push(tile);
            }
            println!("");
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
        println!("{} picked {}", self.action_id, tile);
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
        println!("Waiting for action");
        self.last_time = PreciseTime::now();
    }

    fn out(&mut self, id: usize, tile: String) {
        if self.stage != "out" || id != self.action_id {
            println!("{} sent invalid out", id);
            return;
        }
        match self.tiles[id].hands.iter().position(|x| *x == tile) {
            Some(index) => {
                println!("{} discarded {}", id, tile);
                let duration = self.last_time.to(PreciseTime::now()).num_milliseconds();
                if duration >= 1050 {
                    let penalty = (duration - 950) / 100;
                    self.score[id] -= penalty;
                    println!("{} was fined {} due to timeout", id, penalty);
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
                self.messages.clear();
                self.last_time = PreciseTime::now();
            },
            None => {
                println!("{} sent invalid out", id);
                //TODO
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
            println!("{} gang {}", id, tile);
            return true;
        }
        println!("{} sent invalid gang", id);
        return false;
    }

    fn peng(&mut self, id: usize, tile: &str) -> bool {
        if self.tiles[id].hands.iter().filter(|&x| *x == tile).count() >= 2 {
            for _ in 0..2 {
                let index = self.tiles[id].hands.iter().position(|x| *x == tile).unwrap();
                self.tiles[id].hands.remove(index);
            }
            println!("{} peng {}", id, tile);
            return true;
        }
        println!("{} sent invalid peng", id);
        return false;
    }

    fn chi(&mut self, id: usize, tile: &str) -> bool {
        if id != post_pos(self.action_id) {
            return false;
        }
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
        if self.tiles[id].hands.iter().filter(|&x| set.contains(x)).count() >= 2 {
            for i in set {
                let index = self.tiles[id].hands.iter().position(|x| *x == i).unwrap();
                self.tiles[id].hands.remove(index);
            }
            println!("{} chi {}", id, tile);
            return true;
        }
        return false;
    }

    fn draw(&mut self) {
        unsafe {
            println!("Draw game!");
            for i in 0..4 {
                flags[i] = false;
            }
        }
    }
}

#[derive(Clone, Debug)]
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

fn cal_fan(tiles: Tiles) -> Option<i64> {
    //TODO
    return None;
}