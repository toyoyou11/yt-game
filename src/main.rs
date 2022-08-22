mod game;
mod renderer;
fn main() {
    env_logger::init();
    let mut game = pollster::block_on(game::Game::new());
    game.start();
}
