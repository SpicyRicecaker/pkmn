use engine::frontend::color::Color;
use engine::{context::Context, Runnable};

fn main() {
    println!("Hello, world!");

    let game = Game {
        playerx: 0.0,
        playery: 0.0,
    };

    let (event_loop, ctx) = engine::ContextBuilder::new().with_title("Booboo").build();

    let texture1 = std::fs::read("game/res/floor.png").unwrap();

    engine::main::run(event_loop, ctx, game);
}

struct Game {
    playerx: f32,
    playery: f32,
}

impl Runnable for Game {
    fn tick(&mut self, _ctx: &mut Context) {
        self.playerx += 1.0;
        self.playery += 1.0;
    }
    fn render(&self, ctx: &mut Context) {
        ctx.graphics
            .clear_background(Color::from_hex("#000000").unwrap());
        ctx.graphics.draw_square(
            self.playerx,
            self.playery,
            1.0,
            Color::from_hex("#FFFFFF").unwrap(),
        );
    }
}
