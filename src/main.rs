mod boid;

use std::collections::HashMap;

use bonsai_bt::{Action, BT};
use ggez::{conf, Context, ContextBuilder, event, GameResult, graphics, input, timer};

use boid::game_tick;

use crate::boid::{Boid, BOID_SIZE, BoidAction, NUM_BOIDS};

//window stuff
const HEIGHT: f32 = 720.0;
const WIDTH: f32 = HEIGHT * (16.0 / 9.0);


enum PlayState {
    Setup,
    Play,
    Pause,
}

struct GameState {
    state: PlayState,
    dt: std::time::Duration,
    boids: Vec<Boid>,
    points: Vec<glam::Vec2>,
    bt: BT<BoidAction, String, f32>,
}

impl GameState {
    pub fn new(_ctx: &mut Context, bt: BT<BoidAction, String, f32>) -> GameState {
        GameState {
            state: PlayState::Setup,
            dt: std::time::Duration::new(0, 0),
            boids: Vec::with_capacity(NUM_BOIDS),
            points: vec![
                glam::vec2(0.0, -BOID_SIZE / 2.0),
                glam::vec2(BOID_SIZE / 4.0, BOID_SIZE / 2.0),
                glam::vec2(0.0, BOID_SIZE / 3.0),
                glam::vec2(-BOID_SIZE / 4.0, BOID_SIZE / 2.0),
            ],
            bt,
        }
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.dt = timer::delta(ctx);
        let tick = (self.dt.subsec_millis() as f32) / 1000.0;
        let pressed_keys = input::keyboard::pressed_keys(ctx);

        match self.state {
            PlayState::Setup => {
                self.boids.drain(..);
                if pressed_keys.contains(&event::KeyCode::Space) {
                    self.boids = Boid::create_boids(&self.bt, WIDTH, HEIGHT);
                    self.state = PlayState::Play;
                }
            }

            PlayState::Pause => {
                let pressed_keys = input::keyboard::pressed_keys(ctx);
                if pressed_keys.contains(&event::KeyCode::Space) {
                    self.state = PlayState::Play;
                } else if pressed_keys.contains(&event::KeyCode::R) {
                    self.state = PlayState::Setup;
                }
            }
            PlayState::Play => {
                if pressed_keys.contains(&event::KeyCode::P) {
                    self.state = PlayState::Pause;
                } else if pressed_keys.contains(&event::KeyCode::R) {
                    self.state = PlayState::Setup;
                }
                for i in 0..(self.boids).len() {
                    let boids_vec = self.boids.to_vec();
                    let b = &mut self.boids[i];
                    game_tick(
                        self.dt.as_secs_f32(),
                        input::mouse::position(ctx),
                        b,
                        boids_vec,
                    );

                    //Convert new velocity to postion change
                    b.x += b.dx * tick;
                    b.y += b.dy * tick;

                    self.boids[i] = b.clone();
                }
            }
        };

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.15, 0.2, 0.22, 1.0].into());
        // MENU: display controls
        match self.state {
            PlayState::Setup => {
                let menu_text = graphics::Text::new(graphics::TextFragment {
                    text: "play : <space>\npause : <p>\nreset : <r>".to_string(),
                    color: Some(graphics::Color::WHITE),
                    font: Some(graphics::Font::default()),
                    scale: Some(graphics::PxScale::from(100.0)),
                });

                let text_pos = glam::vec2(
                    (WIDTH - menu_text.width(ctx) as f32) / 2.0,
                    (HEIGHT - menu_text.height(ctx) as f32) / 2.0,
                );

                graphics::draw(
                    ctx,
                    &menu_text,
                    graphics::DrawParam::default().dest(text_pos),
                )?;
            }

            _ => {
                let mb = &mut graphics::MeshBuilder::new();
                for boid in &self.boids {
                    let rot = glam::Mat2::from_angle(boid.dx.atan2(-boid.dy));
                    let pos = glam::vec2(boid.x, boid.y);
                    mb.polygon(
                        graphics::DrawMode::fill(),
                        &[
                            (rot * self.points[0]) + pos,
                            (rot * self.points[1]) + pos,
                            (rot * self.points[2]) + pos,
                            (rot * self.points[3]) + pos,
                        ],
                        boid.color.into(),
                    )?;
                }
                /*Highlight cursor..*/
                mb.circle(
                    graphics::DrawMode::fill(),
                    input::mouse::position(ctx),
                    10.0,
                    0.1,
                    [1.0, 1.0, 1.0, 0.5].into(),
                )?;
                let line = &[
                    glam::vec2(0.0, 0.0),
                    glam::vec2(50.0, 5.0),
                    glam::vec2(42.0, 10.0),
                    glam::vec2(150.0, 100.0),
                ];
                mb.polyline(
                    graphics::DrawMode::stroke(2.0),
                    line,
                    [1.0, 1.0, 1.0, 1.0].into(),
                )?;
                let m = mb.build(ctx)?;
                graphics::draw(ctx, &m, graphics::DrawParam::new())?;
            }
        };
        graphics::present(ctx)
    }
}

fn create_bt() -> BT<BoidAction, String, f32> {
    let avoid_others = Action(BoidAction::AvoidOthers);
    let fly_towards_center = Action(BoidAction::FlyTowardsCenter);
    let limit_speed = Action(BoidAction::LimitSpeed);
    let match_velocity = Action(BoidAction::MatchVelocity);
    let keep_within_bounds = Action(BoidAction::KeepWithinBounds);

    // Run both behaviors in parallell, WhenAll will always return (Running, 0.0) because
    // both behaviors would have to return (Success, dt) to the WhenAll condition to succeed.
    let avoid_and_fly = bonsai_bt::WhenAll(vec![fly_towards_center, avoid_others]);
    let behavior = bonsai_bt::While(
        Box::new(avoid_and_fly),
        // vec![Succees, Success, Running] -> sequence is always returning running
        vec![match_velocity, limit_speed, keep_within_bounds],
    );

    // add some values to blackboard
    let mut blackboard: HashMap<String, f32> = HashMap::new();
    blackboard.insert("win_width".to_string(), WIDTH);
    blackboard.insert("win_height".to_string(), HEIGHT);

    // create bt
    BT::new(behavior, blackboard)
}

fn main() {
    let (mut ctx, events_loop) = ContextBuilder::new("Boids", "Daniel Eisen")
        .window_mode(conf::WindowMode::default().dimensions(WIDTH, HEIGHT))
        .window_setup(conf::WindowSetup::default().samples(conf::NumSamples::Eight))
        .build()
        .expect("Failed to create context");
    let bt = create_bt();
    let game_state = GameState::new(&mut ctx, bt);
    event::run(ctx, events_loop, game_state);
}
