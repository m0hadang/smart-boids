use std::collections::{HashMap, HashSet};

use bonsai_bt::{ActionArgs, BT, Event, State, UpdateArgs, Success, Action, Failure, Sequence, Behavior};
use ggez::{conf, Context, ContextBuilder, event, GameResult, graphics, input, timer};
use ggez::mint::Point2;
use ggez::winit::event::VirtualKeyCode;

use crate::boid::{Boid, BoidAction};

mod boid;

const WINDOW_HEIGHT: f32 = 720.0;
const WINDOW_WIDTH: f32 = WINDOW_HEIGHT * (16.0 / 9.0);
const OBJECT_COUNT: usize = 100;
pub const OBJECT_SIZE: f32 = 32.0; // Pixels

#[derive(Clone, PartialEq)]
enum PlayState {
    Play,
    Setup,
    Pause,
}

#[derive(Clone, Debug)]
enum GameOpState {
    InputKey,
    UpdateGameData,
}

struct GameState {
    state: PlayState,
    dt: std::time::Duration,
    boids: Vec<Boid>,
    points: Vec<glam::Vec2>,
    bt: BT<BoidAction, String, f32>,
    game_op_bt: State<GameOpState>,
}

impl GameState {
    pub fn new(
        _ctx: &mut Context,
        bt: BT<BoidAction, String, f32>,
    ) -> GameState {
        GameState {
            state: PlayState::Setup,
            dt: Default::default(),
            boids: std::default::Default::default(),
            points: vec![
                glam::vec2(0.0, -OBJECT_SIZE / 2.0),
                glam::vec2(OBJECT_SIZE / 4.0, OBJECT_SIZE / 2.0),
                glam::vec2(0.0, OBJECT_SIZE / 3.0),
                glam::vec2(-OBJECT_SIZE / 4.0, OBJECT_SIZE / 2.0),
            ],
            bt,
            game_op_bt: Self::create_bt(),
        }
    }
    fn create_bt() -> State<GameOpState> {
        let state = Sequence(vec![
            Action(GameOpState::InputKey),
            Action(GameOpState::UpdateGameData)
        ]);
        State::new(state)
    }
    fn game_op_tick(&mut self,
                    dt: f32,
                    pressed_keys: &HashSet<VirtualKeyCode>,
                    cursor: Point2<f32>) {
        let e: Event = UpdateArgs { dt: dt.into() }.into();
        let mut game_op_bt = self.game_op_bt.clone();
        game_op_bt.tick(&e, &mut |args: ActionArgs<Event, GameOpState>|
            match args.action {
                GameOpState::InputKey => {
                    if pressed_keys.is_empty() {
                    } else {
                        // -> setup
                        if pressed_keys.contains(&event::KeyCode::R) {
                            self.state = PlayState::Setup;
                            self.boids.drain(..);
                        } else {
                            match self.state {
                                PlayState::Setup => {
                                    // -> play
                                    if pressed_keys.contains(&event::KeyCode::Space) {
                                        self.boids = Boid::create_boids(
                                            &self.bt,
                                            OBJECT_COUNT,
                                            WINDOW_WIDTH,
                                            WINDOW_HEIGHT);
                                        self.state = PlayState::Play;
                                    }
                                }
                                PlayState::Pause => {
                                    // -> play
                                    if pressed_keys.contains(&event::KeyCode::Space) {
                                        self.state = PlayState::Play;
                                    }
                                }
                                PlayState::Play => {
                                    // -> pause
                                    if pressed_keys.contains(&event::KeyCode::P) {
                                        self.state = PlayState::Pause;
                                    }
                                }
                            };
                        }
                    }

                    if self.state == PlayState::Play {
                        (Success, args.dt)
                    } else {
                        (Failure, args.dt)
                    }
                }
                GameOpState::UpdateGameData => {
                    let tick = (self.dt.subsec_millis() as f32) / 1000.0;
                    for i in 0..(self.boids).len() {
                        let boids_vec = self.boids.to_vec();
                        let boid = &mut self.boids[i];
                        Boid::game_tick(
                            self.dt.as_secs_f32(),
                            cursor,
                            boid,
                            boids_vec,
                        );

                        //Convert new velocity to postion change
                        boid.x += (boid.dx * tick);
                        boid.y += (boid.dy * tick);

                        self.boids[i] = boid.clone();
                    }
                    (Success, args.dt)
                }
            },
        );
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.dt = timer::delta(ctx);
        let pressed_keys =
            input::keyboard::pressed_keys(ctx);
        let cursor: Point2<f32> =
            input::mouse::position(ctx);
        self.game_op_tick(
            self.dt.as_secs_f32(),
            pressed_keys,
            cursor);
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
                    (WINDOW_WIDTH - menu_text.width(ctx) as f32) / 2.0,
                    (WINDOW_HEIGHT - menu_text.height(ctx) as f32) / 2.0,
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

fn main() {
    let (mut ctx, events_loop) = ContextBuilder::new("Boids", "Daniel Eisen")
        .window_mode(conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .window_setup(conf::WindowSetup::default().samples(conf::NumSamples::Eight))
        .build()
        .expect("Failed to create context");

    let boid_bt = Boid::create_bt();
    let mut blackboard: HashMap<String, f32> = HashMap::new();
    blackboard.insert("win_width".to_string(), WINDOW_WIDTH);
    blackboard.insert("win_height".to_string(), WINDOW_HEIGHT);
    let boid_bt: BT<BoidAction, String, f32> = BT::new(boid_bt, blackboard);

    let game_state =
        GameState::new(&mut ctx, boid_bt);
    event::run(ctx, events_loop, game_state);
}

