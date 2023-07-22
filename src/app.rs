use bot::BotCreator;
use eframe::{egui, CreationContext};
use egui_extras::RetainedImage;

pub struct ChessBoard<'a> {
    pieces: &'a PieceTexture,
    state: &'a chess::Board,
    result: Option<chess::GameResult>,
}

impl egui::Widget for ChessBoard<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        if let Some(result) = self.result {
            ui.label(format!("{:?}", result))
        } else {
            let space_to_fill = ui.available_size().min_elem();
            let (id, rect) = ui.allocate_space(egui::Vec2::splat(space_to_fill));

            let square_size = rect.width().min(rect.height()) / 8.0;

            let mut start_black = true;
            for r in 0..8 {
                let mut draw_black = start_black;
                for c in 0..8 {
                    let square = egui::Rect::from_min_size(
                        egui::Pos2::new(c as f32 * square_size, r as f32 * square_size),
                        egui::Vec2::splat(square_size),
                    );

                    let color = if draw_black {
                        egui::Color32::BROWN
                    } else {
                        egui::Color32::LIGHT_BLUE
                    };

                    ui.painter()
                        .rect_filled(square, egui::Rounding::none(), color);

                    let piece_square = chess::Square::make_square(
                        chess::Rank::from_index(r),
                        chess::File::from_index(c),
                    );
                    let piece = self.state.piece_on(piece_square);
                    let piece_color = self.state.color_on(piece_square);
                    if let (Some(piece), Some(color)) = (piece, piece_color) {
                        let (texture, uv) =
                            self.pieces
                                .get(ui.ctx(), piece, color == chess::Color::Black);
                        ui.painter()
                            .image(texture, square, uv, egui::Color32::WHITE);
                    }

                    draw_black = !draw_black;
                }
                start_black = !start_black;
            }

            ui.interact(rect, id, egui::Sense::click())
        }
    }
}

pub struct PieceTexture {
    sprite_texture: [RetainedImage; 12],
}

impl PieceTexture {
    pub fn get(
        &self,
        ctx: &egui::Context,
        piece: chess::Piece,
        black: bool,
    ) -> (egui::TextureId, egui::Rect) {
        let sprite_index = match piece {
            chess::Piece::Pawn => 3,
            chess::Piece::Knight => 2,
            chess::Piece::Bishop => 0,
            chess::Piece::Rook => 5,
            chess::Piece::Queen => 4,
            chess::Piece::King => 1,
        };

        let sprite_index = if black {
            sprite_index
        } else {
            sprite_index + 6
        };

        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        let id = self.sprite_texture[sprite_index].texture_id(ctx);

        (id, uv)
    }
}

pub struct BotComms {
    handle: std::thread::JoinHandle<()>,
    tx: std::sync::mpsc::Sender<(std::time::Duration, std::time::Duration, chess::Board)>,
    rx: std::sync::mpsc::Receiver<chess::ChessMove>,
}

impl BotComms {
    fn create<B: BotCreator + 'static>(ctx: egui::Context) -> BotComms {
        let (tx, app_rx) = std::sync::mpsc::channel();
        let (app_tx, rx) = std::sync::mpsc::channel();

        let bot = B::create();
        let handle = std::thread::spawn(move || {
            let mut bot = bot;
            while let Ok((art, brt, state)) = rx.recv() {
                tx.send(bot.get_next_move(art, brt, state));
                ctx.request_repaint();
            }
        });

        BotComms {
            handle,
            tx: app_tx,
            rx: app_rx,
        }
    }
}

pub enum MoveState {
    ToMove,
    Waiting,
    GameComplete,
}

pub struct ChessBotApp {
    pieces: PieceTexture,

    game_state: chess::Game,
    move_state: MoveState,

    move_start: std::time::Instant,
    white_time: std::time::Duration,
    black_time: std::time::Duration,

    white_bot: BotComms,
    black_bot: BotComms,
}

impl ChessBotApp {
    pub fn new(cc: &CreationContext) -> Self {
        let piece_sprites = [
            egui_extras::RetainedImage::from_svg_bytes(
                "black-bishop",
                include_bytes!("../resources/black/bishop.svg"),
            )
            .unwrap(),
            egui_extras::RetainedImage::from_svg_bytes(
                "black-king",
                include_bytes!("../resources/black/king.svg"),
            )
            .unwrap(),
            egui_extras::RetainedImage::from_svg_bytes(
                "black-knight",
                include_bytes!("../resources/black/knight.svg"),
            )
            .unwrap(),
            egui_extras::RetainedImage::from_svg_bytes(
                "black-pawn",
                include_bytes!("../resources/black/pawn.svg"),
            )
            .unwrap(),
            egui_extras::RetainedImage::from_svg_bytes(
                "black-queen",
                include_bytes!("../resources/black/queen.svg"),
            )
            .unwrap(),
            egui_extras::RetainedImage::from_svg_bytes(
                "black-rook",
                include_bytes!("../resources/black/rook.svg"),
            )
            .unwrap(),
            egui_extras::RetainedImage::from_svg_bytes(
                "white-bishop",
                include_bytes!("../resources/white/bishop.svg"),
            )
            .unwrap(),
            egui_extras::RetainedImage::from_svg_bytes(
                "white-king",
                include_bytes!("../resources/white/king.svg"),
            )
            .unwrap(),
            egui_extras::RetainedImage::from_svg_bytes(
                "white-knight",
                include_bytes!("../resources/white/knight.svg"),
            )
            .unwrap(),
            egui_extras::RetainedImage::from_svg_bytes(
                "white-pawn",
                include_bytes!("../resources/white/pawn.svg"),
            )
            .unwrap(),
            egui_extras::RetainedImage::from_svg_bytes(
                "white-queen",
                include_bytes!("../resources/white/queen.svg"),
            )
            .unwrap(),
            egui_extras::RetainedImage::from_svg_bytes(
                "white-rook",
                include_bytes!("../resources/white/rook.svg"),
            )
            .unwrap(),
        ];

        Self {
            pieces: PieceTexture {
                sprite_texture: piece_sprites,
            },

            game_state: chess::Game::new(),
            move_state: MoveState::ToMove,

            move_start: std::time::Instant::now(),
            white_time: std::time::Duration::from_secs(60),
            black_time: std::time::Duration::from_secs(60),

            white_bot: BotComms::create::<bot::Random>(cc.egui_ctx.clone()),
            black_bot: BotComms::create::<bot::Random>(cc.egui_ctx.clone()),
        }
    }

    fn draw_board(&self, ui: &mut egui::Ui) {
        ui.add(ChessBoard {
            pieces: &self.pieces,
            result: self.game_state.result(),
            state: &self.game_state.current_position(),
        });
    }
}

impl eframe::App for ChessBotApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.game_state.result().is_some() {
                self.move_state = MoveState::GameComplete;
            }

            match &self.move_state {
                MoveState::ToMove => {
                    match self.game_state.side_to_move() {
                        chess::Color::White => {
                            self.white_bot.tx.send((
                                self.white_time.clone(),
                                self.black_time.clone(),
                                self.game_state.current_position(),
                            ));
                        }
                        chess::Color::Black => {
                            self.black_bot.tx.send((
                                self.black_time.clone(),
                                self.white_time.clone(),
                                self.game_state.current_position(),
                            ));
                        }
                    }

                    self.move_state = MoveState::Waiting;
                    self.move_start = std::time::Instant::now();
                }
                MoveState::Waiting => match self.game_state.side_to_move() {
                    chess::Color::White => {
                        if let Ok(mv) = self.white_bot.rx.try_recv() {
                            if !self.game_state.make_move(mv) {
                                self.game_state.resign(chess::Color::White);
                            }
                            self.white_time =
                                self.white_time.saturating_sub(self.move_start.elapsed());
                            self.move_state = MoveState::ToMove;
                            ctx.request_repaint();
                        }

                        if self.game_state.can_declare_draw() {
                            self.game_state.declare_draw();
                        } else if self
                            .white_time
                            .saturating_sub(self.move_start.elapsed())
                            .is_zero()
                        {
                            self.game_state.resign(chess::Color::White);
                        }
                    }
                    chess::Color::Black => {
                        if let Ok(mv) = self.black_bot.rx.try_recv() {
                            if !self.game_state.make_move(mv) {
                                self.game_state.resign(chess::Color::Black);
                            }
                            self.black_time =
                                self.black_time.saturating_sub(self.move_start.elapsed());
                            self.move_state = MoveState::ToMove;
                            ctx.request_repaint();
                        }

                        if self.game_state.can_declare_draw() {
                            self.game_state.declare_draw();
                        } else if (self.black_time.saturating_sub(self.move_start.elapsed()))
                            .is_zero()
                        {
                            self.game_state.resign(chess::Color::Black);
                        }
                    }
                },
                MoveState::GameComplete => {
                    if let Some(result) = self.game_state.result() {
                        ui.label(format!("Main Loop: {result:?}"));
                    }
                }
            }

            self.draw_board(ui);
        });
    }
}
