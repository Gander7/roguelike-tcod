use tcod::colors::*;
use tcod::console::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

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

    let con = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);
    let mut tcod = Tcod { root, con };
    tcod::system::set_fps(LIMIT_FPS);

    let mut player_x = SCREEN_WIDTH/2;
    let mut player_y = SCREEN_HEIGHT/2;

    while !tcod.root.window_closed() {
        tcod.con.set_default_foreground(WHITE);
        tcod.con.clear();
        tcod.con.put_char(player_x, player_y, '@', BackgroundFlag::None);
        blit(
            &tcod.con,
            (0,0),
            (SCREEN_WIDTH, SCREEN_HEIGHT),
            &mut tcod.root,
            (0,0),
            1.0,
            1.0,
        );
        tcod.root.flush();
        tcod.root.wait_for_keypress(true);

        let exit = handle_keys(&mut tcod, &mut player_x, &mut player_y);
        if exit {
            break;
        }
    }
}

fn handle_keys(tcod: &mut Tcod, player_x: &mut i32, player_y: &mut i32) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = tcod.root.wait_for_keypress(true); 
    match key {
        // Movement Keys
        Key { code: Up, .. } => *player_y -= 1,
        Key { code: Down, .. } => *player_y += 1,
        Key { code: Left, .. } => *player_x -= 1,
        Key { code: Right, .. } => * player_x += 1,

        Key { code: Enter, alt: true, .. } =>  { 
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        },
        Key { code: Escape, .. } => return true,

        _ => {}
    }

    false
}
