use tcod::colors::*;
use tcod::console::*;
use rand::Rng;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;
const COLOR_DARK_GROUND: Color = Color { r:50, g:50, b:150 };
const COLOR_DARK_WALL: Color = Color { r:0, g:0, b:100 };

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

const LIMIT_FPS: i32 = 20;

struct Tcod {
    root: Root,
    con: Offscreen,
}

fn main() {
    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust/libtcod tutorial")
        .init();

    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
    let mut tcod = Tcod { root, con };
    tcod::system::set_fps(LIMIT_FPS);

    let player = Object::new(0, 0, '@', WHITE);
    let npc = Object::new(SCREEN_WIDTH/2-5, SCREEN_HEIGHT/2, '@', YELLOW);
    let mut objects = [player, npc];

    let game = Game { map: generate_map(&mut objects[0]), };
    while !tcod.root.window_closed() {
        tcod.con.clear();

        render_all(&mut tcod, &game, &objects);
        tcod.root.flush();
        tcod.root.wait_for_keypress(true);
        
        let player = &mut objects[0];
        let exit = handle_keys(&mut tcod, &game, player);
        if exit {
            break;
        }
    }
}

fn handle_keys(tcod: &mut Tcod, game: &Game, player: &mut Object) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = tcod.root.wait_for_keypress(true); 
    match key {
        // Movement Keys
        Key { code: Up, .. } => player.move_by(0,-1, game),
        Key { code: Down, .. } => player.move_by(0,1, game),
        Key { code: Left, .. } => player.move_by(-1,0, game),
        Key { code: Right, .. } => player.move_by(1,0, game),

        Key { code: Enter, alt: true, .. } =>  { 
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        },
        Key { code: Escape, .. } => return true,

        _ => {}
    }

    false
}

fn generate_map(player: &mut Object) -> Map {
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);

        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = Rect::new(x, y, w, h);
        let failed = rooms.iter().any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            create_room(new_room, &mut map);

            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                player.x = new_x;
                player.y = new_y;
            } else {
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                if rand::random() {
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }

            rooms.push(new_room);
        }
    }

    map
}

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]) {

        for object in objects {
            object.draw(&mut tcod.con);
        }

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let wall = game.map[x as usize][y as usize].block_sight;
                if wall {
                    tcod.con.set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
                } else {
                    tcod.con.set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
                }
            }
        }

        blit(
            &tcod.con,
            (0,0),
            (MAP_WIDTH, MAP_HEIGHT),
            &mut tcod.root,
            (0,0),
            1.0,
            1.0,
        );
}

#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}
impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object { x, y, char, color }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
        if !game.map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {
            self.x += dx;
            self.y += dy;
        }
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}

#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}
impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
        }
    }
}

type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
}

// A rectangular area of the map
#[derive(Debug, Copy, Clone)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}
impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x+w,
            y2: y+h,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool {
        (self.x1 <= other.x2)
            && (self.x2 >= other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }
}

fn create_room(room: Rect, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    } 
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    use std::cmp;

    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    use std::cmp;

    for y in cmp::min(y1, y2)..(cmp::max(y1,y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}
