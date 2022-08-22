mod game;
mod renderer;
fn main() {
    pollster::block_on(run());
}

async fn run(){
    env_logger::init();
    let mut game = game::GameLauncher::new().await;
    game.launch();
}
