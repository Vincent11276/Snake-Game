

use sfml::{
    graphics::*, system::*, window::*, SfBox,
};

use rand::Rng;

const KEY_MOVE_UP       : Key = Key::UP;
const KEY_MOVE_DOWN     : Key = Key::DOWN;
const KEY_MOVE_LEFT     : Key = Key::LEFT;
const KEY_MOVE_RIGHT    : Key = Key::RIGHT;

const KEY_START_GAME    : Key = Key::NUM1;
const KEY_PAUSE_GAME    : Key = Key::NUM2;
const KEY_CONTINUE_GAME : Key = Key::NUM3;
const KEY_END_GAME      : Key = Key::NUM4;
const KEY_RESTART_GAME  : Key = Key::NUM5;

const COLOR_SNAKE       : Color = Color::GREEN;
const COLOR_FOOD        : Color = Color::RED;
const COLOR_WORLD       : Color = Color::WHITE;

const SNAKE_DIRECTION   : Direction = Direction::Right;
const SNAKE_MOVE_TIME   : Time = Time::milliseconds(50);
const SNAKE_LENGTH      : i32 = 10;

#[derive(Copy, Clone, Debug)]
#[derive(PartialEq)]
enum Direction {
    Up, Down, Right, Left
}

#[derive(Copy, Clone, Debug)]
enum GameState {
    StateMenu,
    StatePlaying,
    StatePaused,
    StateGameEnded,
    StateGameOver
}

struct SnakeGame {
    // system data
    world                   : Vec<Vec<char>>,
    playing                 : bool,
    game_state              : GameState,
    elapsed_time            : Time,
    last_frame_time         : Time,
    food_location           : Vector2i,
    
    // snake data
    snake_head              : Vector2i,
    snake_body              : Vec<Vector2i>,
    snake_direction    : Direction,
    snake_init_length       : i32,
    snake_move_time         : Time,

    // game data
    current_score           : i32,
    best_score              : i32,

    // graphics
    d_world_vertices        : Vec<Vertex>,
    tile_size               : Vector2f,
}

struct Helpers { }

impl Helpers {
    fn get_absolute_direction(direction: Direction) -> Vector2i {
        match direction {
            Direction::Up => Vector2i::new(0, -1),
            Direction::Down => Vector2i::new(0, 1),
            Direction::Right => Vector2i::new(1, 0),
            Direction::Left => Vector2i::new(-1, 0),
        }
    }

    fn reverse_direction(direction: Direction) -> Direction {
        match direction {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }

    fn set_tex_coords_by_rect(rect: FloatRect, vertex_slice: &mut [Vertex]) {
        vertex_slice[0].tex_coords = Vector2f::new(rect.left, rect.top);
        vertex_slice[1].tex_coords = Vector2f::new(rect.left + rect.width, rect.top);
        vertex_slice[2].tex_coords = Vector2f::new(rect.left + rect.width, rect.top + rect.height);
        vertex_slice[3].tex_coords = Vector2f::new(rect.left, rect.top + rect.height);
    }

    
}

struct TileMap { 
    tile_size : Vector2f,
    d_world_vertices: Vec<Vertex>,
    world_data : Vec<Vec<i32>>,
    tilemap_tex: SfBox<Texture>,
}

impl<'s> TileMap {
    fn new(world_size: Vector2i, tile_size: Vector2f) -> Self {
        TileMap {
            tile_size,
            d_world_vertices: vec![],
            world_data: vec![vec![world_size.x as i32]; world_size.y as usize],
            tilemap_tex: Texture::new(0, 0).unwrap(),
        }
    }

    fn load_from_file(&mut self, path: &str) {
        let tilemap_tex = Texture::from_file(path);
    }

    fn tile_number_to_coords(&mut self, tile_number: i32) -> Vector2i {
        let world_size = self.get_world_size();
        
        Vector2i::new(tile_number / world_size.x, tile_number / world_size.x - 1)
    }

    fn coords_to_tile_number(&mut self, coords: Vector2i) -> i32 {
        let world_size = self.get_world_size();

        return world_size.x * coords.y + coords.x;
    }

    fn get_texture_rect_by_coords(&mut self, coords: Vector2i) -> FloatRect {
        FloatRect::new(coords.x as f32, coords.y as f32, self.tile_size.x, self.tile_size.x)
    }
    fn get_world_size(&mut self) -> Vector2i {
        Vector2i::new(self.world_data[0].len() as i32, self.world_data.len() as i32)
    }

    fn resize_world(&mut self, width: i32, height: i32) {
        self.world_data.resize(height as usize, vec![-1, width.into()]);
    }

    fn try_set_block(&mut self, x: i32, y: i32, value: i32) {
        if self.is_inbounds(x, y) {       
            let starting_index = self.coords_to_tile_number((x, y).into()) * 4;
            
            let mut quad = &mut self.d_world_vertices[starting_index as usize..starting_index as usize + 4];

            //let rect = self.get_texture_rect_by_coords(Vector2i::new(x, y));
            let rect = FloatRect::new(x as f32, y as f32, self.tile_size.x, self.tile_size.x);

            quad[0].tex_coords = Vector2f::new(rect.left, rect.top);
            quad[1].tex_coords = Vector2f::new(rect.left + rect.width, rect.top);
            quad[2].tex_coords = Vector2f::new(rect.left + rect.width, rect.top + rect.height);
            quad[3].tex_coords = Vector2f::new(rect.left, rect.top + rect.height);

            self.world_data[y as usize][x as usize] = value;
        }
    }

    fn is_inbounds(&mut self, x: i32, y: i32) -> bool {
        let world_size = self.get_world_size();

        x >= 0 && y >= 0 && x < world_size.x as i32 && y < world_size.y
    }
}

impl<'s> Drawable for TileMap {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        render_target: &mut dyn RenderTarget,
        states: &RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        states.set_texture(Some(&*self.tilemap_tex));

        render_target.draw_primitives(&self.d_world_vertices, PrimitiveType::QUADS, &states);   
    }
}

impl<'s> SnakeGame {
    fn new(world_width: usize, world_height: usize) -> Self {
        SnakeGame { 
            // system data
            world                   : vec![vec!['x'; world_width]; world_height],
            playing                 : false,
            game_state              : GameState::StateMenu,
            elapsed_time            : Time::ZERO,
            last_frame_time         : Time::ZERO,
            food_location           : (0, 0).into(),
            
            // snake data
            snake_head              : (0, 0).into(),
            snake_body              : vec![],
            snake_direction         : SNAKE_DIRECTION,
            snake_init_length       : SNAKE_LENGTH,
            snake_move_time         : SNAKE_MOVE_TIME,
            
        
            // game data
            current_score           : 0,
            best_score              : 0,

            // graphics
            d_world_vertices        : vec![],
            tile_size               : (16.0, 16.0).into()
        }
    }

    fn init_snake(&mut self) {
        let half_of_world = self.get_world_size() / 2;

        self.snake_head = half_of_world;

        for i in 1..self.snake_init_length {            
            self.snake_body.push(half_of_world as Vector2i + Vector2i::new(-i, 0));
        }

        println!("New snake generated!\n    Head: {:?}\n    Body: {:?}", self.snake_head, self.snake_body);
    }

    fn init_world_vertices(&mut self) {
        let world_size = self.get_world_size();

        self.d_world_vertices.resize(((world_size.x * world_size.y) * 4) as usize, Vertex::with_pos_color((0.0, 0.0).into(), Color::WHITE));
        
        for y in 0..world_size.y {
            for x in 0..world_size.x {         
                self.set_world_block(x, y, COLOR_WORLD);       
                let tile_number = (x + y * world_size.x) * 4;

                let quad = &mut self.d_world_vertices[tile_number as usize.. tile_number as usize + 4];

                quad[0].position = (x as f32 * self.tile_size.x, y as f32 * self.tile_size.y).into();
                quad[1].position = ((x as f32 + 1.0) * self.tile_size.x, y as f32 * self.tile_size.y).into();
                quad[2].position = ((x as f32 + 1.0) * self.tile_size.x, (y as f32 + 1.0) * self.tile_size.y).into();
                quad[3].position = (x as f32 * self.tile_size.x, (y as f32 + 1.0) * self.tile_size.y).into();
            }
        }
        println!("New world vertices generated!");
    }

    fn restart_game(&mut self) {
        println!("Game has been restarted!");
        self.start_game();
    }

    fn start_game(&mut self) {
        let world_size = self.get_world_size();

        self.world = vec![vec!['x'; world_size.x as usize]; world_size.y as usize];
        self.snake_direction = Direction::Right;
        self.elapsed_time = Time::ZERO;
        self.last_frame_time = Time::ZERO;
        self.current_score = 0;
        self.playing = true;

        self.init_snake();
        self.init_world_vertices();
        self.spawn_random_food();

        // Draw snake head
        self.set_world_block(self.snake_head.x, self.snake_head.y, Color::GREEN);
        
        // Draw snake body
        for body in self.snake_body.clone() {
            self.set_world_block(body.x, body.y, Color::GREEN);
        }

        println!("Game has been started!");
    }

    fn set_world_block(&mut self, x: i32, y: i32, color: Color) {
        let tile_number = (x + y * self.get_world_size().x) * 4;

        let quad = &mut self.d_world_vertices[tile_number as usize..tile_number as usize + 4];

        quad[0].color = color;
        quad[1].color = color;
        quad[2].color = color;
        quad[3].color = color;
    }

    fn clear_world(&mut self) {
        for vertex in &mut self.d_world_vertices {
            vertex.color = COLOR_WORLD;
        }
    }

    fn pause_game(&mut self) {
        println!("Game has been paused!");

        self.game_state = GameState::StatePaused;
    }

    fn continue_game(&mut self) {
        println!("Game has been continued!");

        self.game_state = GameState::StatePlaying;
    }

    fn end_game(&mut self) {
        println!("Game has been ended!");

        if self.current_score > self.best_score {
            println!("New best score with a points of {}!", self.current_score);
            self.best_score = self.current_score;
        }
        self.game_state = GameState::StateGameEnded;

    }

    fn get_world_size(&mut self) -> Vector2i {
        return Vector2i::new(self.world[0].len() as i32, self.world.len() as i32);
    }

    fn try_set_direction(&mut self, desired_direction: Direction) {
        if !(self.snake_direction == Helpers::reverse_direction(desired_direction)) {
            self.snake_direction = desired_direction;
        }
    }

    fn get_next_head_location(&mut self) -> Vector2i {
        let world_size = self.get_world_size();
        
        let mut target_head_location 
            = self.snake_head + Helpers::get_absolute_direction(self.snake_direction);

        //  prevent head from moving going out of world
        if target_head_location.x > world_size.x - 1 {
            target_head_location.x = 0;
        }
        else if target_head_location.y > world_size.y - 1 {
            target_head_location.y = 0;
        }
        else if target_head_location.x < 0 {
            target_head_location.x = world_size.x - 1;
        }
        else if target_head_location.y < 0 {
            target_head_location.y = world_size.y - 1;
        }
        return target_head_location;
    }

    fn move_snake(&mut self) {
        let next_head_location = self.get_next_head_location();

        let tail_location = *self.snake_body.first().unwrap();
        
        // check if head is not colliding with its body
        if !self.snake_body.contains(&next_head_location) {
            // copy the location of the succeeding body 
            for i in (1..self.snake_body.len()).rev() {
                self.snake_body[i] = self.snake_body[i - 1];
            }
            *self.snake_body.first_mut().unwrap() = self.snake_head;
            self.snake_head = next_head_location;

            // check if snake ate the food
            if next_head_location == self.food_location {
                self.snake_body.push(tail_location);
                self.current_score += 1;
                self.spawn_random_food();

                println!("You ate an apple!");
            }
        }
        else {
            self.game_state = GameState::StateGameOver;

            println!("Game Over! Your score is {}", self.current_score);
        }
    }

    fn update_world(&mut self) {
        self.clear_world();

        // draw snake head
        self.set_world_block(self.snake_head.x, self.snake_head.y, COLOR_SNAKE);

        // draw snake body
        for body in self.snake_body.clone() {
            self.set_world_block(body.x, body.y, COLOR_SNAKE);
        }

        // draw food
        self.set_world_block(self.food_location.x, self.food_location.y, COLOR_FOOD);
    }
    
    fn spawn_random_food(&mut self) {
        let world_size = self.get_world_size();

        let pos_x = rand::thread_rng().gen_range(0..world_size.x);
        let pos_y = rand::thread_rng().gen_range(0..world_size.y);

        let tile_number = (pos_x + pos_y * world_size.x) * 4;
        
        let quad = &mut self.d_world_vertices[tile_number as usize.. tile_number as usize + 4];

        quad[0].color = Color::RED;
        quad[1].color = Color::RED;
        quad[2].color = Color::RED;
        quad[3].color = Color::RED;

        // store the food location
        self.food_location = (pos_x, pos_y).into(); 
    }

    fn process_event(&mut self, event: &Event) {
        match event {
            Event::KeyPressed { code, .. } => {
                match self.game_state {
                    GameState::StateMenu => {
                        match *code {
                            KEY_START_GAME => {
                                self.game_state = GameState::StatePlaying;
                                self.start_game();
                            }
                            _ => ()
                        }
                    }
                    GameState::StatePlaying => {
                        match *code {
                            // game actions
                            KEY_PAUSE_GAME   => self.pause_game(),
                            KEY_END_GAME     => self.end_game(),
                            KEY_RESTART_GAME => self.restart_game(),

                            // player movements
                            KEY_MOVE_UP      => self.try_set_direction(Direction::Up),
                            KEY_MOVE_LEFT    => self.try_set_direction(Direction::Left),
                            KEY_MOVE_DOWN    => self.try_set_direction(Direction::Down),
                            KEY_MOVE_RIGHT   => self.try_set_direction(Direction::Right),

                            _ => ()
                        }
                    }
                    GameState::StatePaused => {
                        match *code {
                            KEY_CONTINUE_GAME => self.continue_game(),
                            KEY_RESTART_GAME  => self.restart_game(),
                            KEY_END_GAME      => self.end_game(),
                            _ => ()
                        }
                    }

                    GameState::StateGameOver => {
                        match *code {
                            KEY_START_GAME => self.start_game(),
                            KEY_CONTINUE_GAME => self.continue_game(),
                            KEY_RESTART_GAME  => self.restart_game(),
                            _ => ()
                        }
                    }
                    _ => ()
                }
            }
            _ => ()
        }
    }

    fn update(&mut self, delta: Time) {
        match self.game_state {
            GameState::StatePlaying => {
                self.elapsed_time += delta;
    
                if self.elapsed_time >= self.last_frame_time + self.snake_move_time {
                    self.last_frame_time = self.elapsed_time;
                    self.move_snake();
                    self.update_world();
                }
            }
            _ => ()
        }

    }
}

impl<'s> Drawable for SnakeGame {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        render_target: &mut dyn RenderTarget,
        states: &RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        match self.game_state {
            GameState::StatePlaying => {
                render_target.draw_primitives(&self.d_world_vertices, PrimitiveType::QUADS, &states);
            }
            _ => ()
        }
    }
}

fn main() {
    let mut window = RenderWindow::new(
        (800, 600), "Snake Game",
        Style::DEFAULT, 
        &ContextSettings::default()
    );
    window.set_framerate_limit(60);

    let mut tilemap = TileMap::new(Vector2i::new(10, 15), Vector2f::new(32.0, 32.0));
    tilemap.load_from_file("C:\\Users\\VitalityEdge42\\Documents\\GitHub\\Snake-Game\\src\\snake-graphics.png");

    tilemap.try_set_block(2, 4, 2);

    
    let mut snake_game = SnakeGame::new(32, 32);


    let mut clock = Clock::start();

    while window.is_open() {
        let delta_time = clock.restart();

        while let Some(event) = window.poll_event() {
            snake_game.process_event(&event);

            match event {
                Event::Closed => window.close(),
                _ => ()
            }
        }
        snake_game.update(delta_time);

        window.clear(Color::BLACK);
        window.draw(&snake_game);
        window.display();
    }
}