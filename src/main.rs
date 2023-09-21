use macroquad::prelude::*;

trait Collidable {
    fn is_colliding(&self, other: &dyn Collidable) -> bool;
    fn get_position(&self) -> Vec2;
    fn get_radius(&self) -> f32;
}

impl dyn Collidable {
    fn circle_circle_intersection(circle1: &dyn Collidable, circle2: &dyn Collidable) -> bool 
    {   
        let p1 = circle1.get_position();
        let p2 = circle2.get_position();

        // Calculate the distance between the two circles.
        let mut dx = p1.x - p2.x;
        let mut dy = p1.y - p2.y;
      
        // Wrap the distance around the region if necessary.
        if dx > screen_width() / 2.0 {
          dx -= screen_width();
        } else if dx < -screen_width() / 2.0 {
          dx += screen_width();
        }
      
        if dy > screen_height() / 2.0 {
            dy -= screen_height();
        } else if dy < -screen_height() / 2.0 {
            dy += screen_height();
        }
      
        // Calculate the squared distance between the two circles.
        let distance_squared = dx * dx + dy * dy;
      
        // If the squared distance is less than the sum of the radii squared, then the
        // two circles are colliding.
        let radii = circle1.get_radius() + circle2.get_radius();

        return distance_squared < radii * radii;
    }
}

enum AsteroidSize {
    Small,
    Medium,
    Large,
}

/// Asteroid object
///
/// Asteroids move in a random direction. They rotate slowly and wrap around the 
/// screen when they reach the edge. When shot, they break into smaller asteroids 
/// until they are small enough to be destroyed.
/// 
/// Asteroids may spawn in two ways: either at a random position on the edge of
/// the screen, or at a specific position. The latter is used when an asteroid
/// is destroyed and spawns smaller asteroids.
/// 
/// # Examples
/// 
/// ```
/// let asteroid = Asteroid::spawn_new(AsteroidSize::Large);
/// let asteroid = Asteroid::spawn_new_at(AsteroidSize::Large, Vec2::new(0., 0.));
/// ```
struct Asteroid {
    size: AsteroidSize,
    diameter: f32,
    position: Vec2,
    velocity: Vec2,
    rotation: f32,
    rotation_speed: f32,
    vertices: Vec<Vec2>,
    is_alive: bool,
}

impl Asteroid {
    /// Spawn new asteroid at a given position. 
    /// 
    /// Asteroid size is used to determine the diameter, number of sides, and
    /// angular velocity of the asteroid. The position is used to determine the
    /// starting location of the asteroid. The velocity is determined randomly.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let asteroid = Asteroid::spawn_new(AsteroidSize::Medium, Vec2::new(0., 0.));
    /// ```
    fn spawn_new_at(size: AsteroidSize, position: Vec2) -> Self {

        let screen_edge: f32 = std::cmp::min(screen_width() as i32, screen_height() as i32) as f32;
        
        // Diameter magic numbers for asteroid sizes
        let diameter = match size {
            AsteroidSize::Small => screen_edge * 0.05,
            AsteroidSize::Medium => screen_edge * 0.1,
            AsteroidSize::Large => screen_edge * 0.2,
        };

        // Sides magic numbers for asteroid sizes
        let sides = match size {
            AsteroidSize::Small => 6.0,
            AsteroidSize::Medium => 9.0,
            AsteroidSize::Large => 12.0,
        };
        
        // Angular velocity magic numbers for asteroid sizes
        let angular_velocity = match size {
            AsteroidSize::Small => 0.2,
            AsteroidSize::Medium => 0.1,
            AsteroidSize::Large => 0.05,
        };

        let speed = match size {
            AsteroidSize::Small => screen_edge * 0.004,
            AsteroidSize::Medium => screen_edge * 0.002,
            AsteroidSize::Large => screen_edge * 0.001,
        };

        let mut vertices: Vec<Vec2> = Vec::new();

        // Generate vertices
        for i in 0..sides as usize {
            let radius = diameter / 2.0 * rand::gen_range(0.6, 1.0);
            let angle = i as f32 / sides * std::f32::consts::PI * 2.0;
            let x = angle.cos() * radius;
            let y = angle.sin() * radius;
            vertices.push(Vec2::new(x, y));
        }

        // Generate random direction and velocity
        let direction = rand::gen_range(0.0, std::f32::consts::PI * 2.0);
        let velocity = Mat2::from_angle(direction).mul_vec2(Vec2::X * speed);
        let rotation = rand::gen_range(0.0, std::f32::consts::PI * 2.0);
        let rotation_speed = angular_velocity * rand::gen_range(-1.0, 1.0);

        Self {
            size,
            diameter,
            position,
            velocity,
            rotation,
            rotation_speed,
            vertices,
            is_alive: true,
        }
    }
    
    /// Spawn new asteroid at a random position on the edge of the screen.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let asteroid = Asteroid::spawn_new(AsteroidSize::Large);
    /// ```
    fn spawn_new(size: AsteroidSize) -> Self {
        let position = match rand::gen_range(0, 4) { 
            0 => Vec2::new(0., rand::gen_range(0.0, screen_height())),
            1 => Vec2::new(screen_width(), rand::gen_range(0.0, screen_height())),
            2 => Vec2::new(rand::gen_range(0.0, screen_width()), 0.),
            3 => Vec2::new(rand::gen_range(0.0, screen_width()), screen_height()),
            _ => Vec2::new(0., 0.),
        };

        Self::spawn_new_at(size, position)
    }

    /// Destroy asteroid by marking it dead. Any calls to `is_alive` will return
    /// false after this function is called.
    fn destroy(&mut self) {
        self.is_alive = false;
    }

    /// Check if asteroid is still alive.
    fn is_alive(&self) -> bool {
        self.is_alive
    }

    /// Update asteroid position and rotation.
    /// 
    /// Asteroids move in a random direction. They rotate slowly and wrap around the
    /// screen when they reach the edge.
    fn update(&mut self) {
        self.position += self.velocity;
        self.rotation += self.rotation_speed;

        if self.position.x > screen_width() {
            self.position.x = 0.0;
        } else if self.position.x < 0.0 {
            self.position.x = screen_width();
        }

        if self.position.y > screen_height() {
            self.position.y = 0.0;
        } else if self.position.y < 0.0 {
            self.position.y = screen_height();
        }
    }

    /// Draw asteroid.
    /// 
    /// Asteroids are drawn as polygons with a random number of sides. The vertices
    /// are rotated by the asteroid's rotation. Asteroids are drawn multiple times
    /// when they wrap around the screen, to prevent them from disappearing when
    /// they reach the edge.
    fn draw(&self) {
        // Rotate vertices
        let rotation_matrix = Mat2::from_angle(self.rotation);        
        let rotated_vertices: Vec<Vec2> = self.vertices.iter().map(|v| rotation_matrix.mul_vec2(*v)).collect();

        // Draw asteroid
        self.draw_vertices_at(self.position, &rotated_vertices);

        // Calculate radius
        let radius = self.diameter / 2.0;

        // Horizontal overlaps
        if self.position.x > screen_width() - radius {
            self.draw_vertices_at(Vec2::new(self.position.x - screen_width(), self.position.y), &rotated_vertices);
        } else if self.position.x < radius {
            self.draw_vertices_at(Vec2::new(self.position.x + screen_width(), self.position.y), &rotated_vertices);
        }
        
        // Vertical overlaps
        if self.position.y > screen_height() - radius {
            self.draw_vertices_at( Vec2::new(self.position.x, self.position.y - screen_height()), &rotated_vertices);
        } else if self.position.y < radius {
            self.draw_vertices_at(Vec2::new(self.position.x, self.position.y + screen_height()), &rotated_vertices);
        }
    }

    /// Draw shape at position.
    fn draw_vertices_at(&self, position: Vec2, vertices: &Vec<Vec2>) {
        for i in 0..vertices.len() {
            let start = position + vertices[i];
            let end = position + vertices[(i + 1) % vertices.len()];
            
            draw_line(start.x, start.y, end.x, end.y, 2., WHITE);
        }
    }
}

impl Collidable for Asteroid {
    fn is_colliding(&self, other: &dyn Collidable) -> bool {
        <dyn Collidable>::circle_circle_intersection(self, other)
    }

    fn get_position(&self) -> Vec2 {
        self.position
    }

    fn get_radius(&self) -> f32 {
        self.diameter / 2.0
    }
}

/// Ship object
/// 
/// The ship is controlled by the player. It can move in any direction, and shoot
/// bullets. The ship has a cooldown on shooting, and can only shoot again after
/// a certain amount of time has passed. The ship has a maximum speed, and will
/// not accelerate past this speed. 
/// 
struct Ship {
    position: Vec2,
    velocity: Vec2,
    max_speed: f32,
    thrust: f32,
    rotation: f32,
    rotation_speed: f32,
    radius: f32,
    shot_cooldown: f64,
    shot_recharge: f64,
    shot_speed: f32,
    shot_lifespan: f32,
    respawn_lifespan: f64,
    shield_lifespan: f64,
    vertices: Vec<Vec2>,    
}

impl Ship {
    // Construct ship object
    fn spawn_new() -> Self {
        let screen_edge: f32 = std::cmp::min(screen_width() as i32, screen_height() as i32) as f32;

        let thrust = screen_edge * 0.0003;
        let max_speed = screen_edge * 0.005;
        
        Self {
            position: Vec2::new(screen_width() / 2.0, screen_height() / 2.0),
            velocity: Vec2::new(0., 0.),
            max_speed,
            thrust,
            rotation: 0.0,
            rotation_speed: 0.0,
            radius: screen_edge / 80.0,
            shot_cooldown: 0.0,
            shot_recharge: 0.2,
            shot_speed: screen_edge * 0.01,
            shot_lifespan: 0.5,
            respawn_lifespan: 0.0,
            shield_lifespan: 0.0,
            vertices: vec![
                Vec2::new(0., -screen_edge / 30.0),
                Vec2::new(screen_edge / 60.0, screen_edge / 60.0),
                Vec2::new(0., screen_edge / 100.0),
                Vec2::new(-screen_edge / 60.0, screen_edge / 60.0),
            ],
        }
    }

    fn respawn(&mut self) {
        self.respawn_lifespan = get_time() + 2.0;
        self.shield_lifespan = self.respawn_lifespan + 2.0;

        self.reset();
    }

    // Reset player position and velocity
    fn reset(&mut self) {
        self.position = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
        self.velocity = Vec2::new(0., 0.);
        self.rotation = 0.0;
        self.rotation_speed = 0.0;
    }

    fn is_respawning(&self) -> bool {
        get_time() < self.respawn_lifespan
    }

    fn is_shield_active(&self) -> bool {
        get_time() < self.shield_lifespan
    }

    fn get_exhaust_position(&self) -> Vec2 {
        self.position + Mat2::from_angle(self.rotation).mul_vec2(self.vertices[2])
    }

    // Accelerate ship in direction of rotation
    fn thrust(&mut self) {
        let rotation_matrix = Mat2::from_angle(self.rotation);
        self.velocity += rotation_matrix.mul_vec2(Vec2::new(0., -self.thrust));

        if self.velocity.length() > self.max_speed {
            self.velocity = self.velocity.normalize() * self.max_speed;
        }
    }

    // Steer ship
    fn steer(&mut self, direction: f32) {
        self.rotation_speed = direction;
    }

    // Shoot bullet
    fn shoot(&mut self) -> Option<Bullet> {
        let current_time = get_time();

        // If we're still in cooldown, don't shoot
        if self.shot_cooldown < current_time {
            self.shot_cooldown = current_time + self.shot_recharge;
        } else {
            return None;
        }

        // Spawn bullet
        let rotation_matrix = Mat2::from_angle(self.rotation);
        let position = rotation_matrix.mul_vec2(self.vertices[0].clone()) + self.position;
        let velocity = Mat2::from_angle(self.rotation).mul_vec2(Vec2::new(0.0, -self.shot_speed));
        
        Some(Bullet::spawn_new(position, velocity, self.shot_lifespan))
    }

    // Update ship position and rotation
    fn update(&mut self) {
        self.position += self.velocity;
        self.rotation += self.rotation_speed;        

        // Wrap around screen
        if self.position.x > screen_width() {
            self.position.x = 0.0;
        } else if self.position.x < 0.0 {
            self.position.x = screen_width();
        }

        if self.position.y > screen_height() {
            self.position.y = 0.0;
        } else if self.position.y < 0.0 {
            self.position.y = screen_height();
        }

        self.velocity *= 0.99;
    }

    // Render ship
    fn draw(&self) {
        if !self.is_respawning() {

            if self.is_shield_active() {
                let current_time = get_time();
                if (current_time * 50.0) as u32 % 2 == 0 {
                    draw_circle_lines(self.position.x, self.position.y, self.radius * 2.5 as f32, 2.0, WHITE);
                }
            }

            let rotation_matrix = Mat2::from_angle(self.rotation);        
            let rotated_vertices: Vec<Vec2> = self.vertices.iter().map(|v| rotation_matrix.mul_vec2(*v)).collect();

            for i in 0..rotated_vertices.len() {
                let start = self.position + rotated_vertices[i];
                let end = self.position + rotated_vertices[(i + 1) % rotated_vertices.len()];
                
                draw_line(start.x, start.y, end.x, end.y, 2., WHITE);
            }
        }
    }
}

impl Collidable for Ship {
    fn is_colliding(&self, other: &dyn Collidable) -> bool {
        if self.is_shield_active() || self.is_respawning() {
            return false;
        }

        <dyn Collidable>::circle_circle_intersection(self, other)
    }

    fn get_position(&self) -> Vec2 {
        self.position
    }

    fn get_radius(&self) -> f32 {
        self.radius
    }
}

/// Bullet object
/// 
/// Bullets are shot by the player. They move in a straight line, and disappear
/// after a certain amount of time. Bullets wrap around the screen when they
/// reach the edge.
struct Bullet {
    position: Vec2,
    velocity: Vec2,
    lifespan: f32,
}

impl Bullet {
    /// Spawn new bullet at a given position.
    fn spawn_new(position: Vec2, velocity: Vec2, lifespan: f32) -> Self {
        Self {
            position,
            velocity,
            lifespan,
        }
    }

    fn destroy(&mut self) {
        self.lifespan = 0.0;
    }

    /// Check if bullet is still alive.
    fn is_alive(&self) -> bool {
        self.lifespan > 0.0
    }

    /// Update bullet position and lifespan.
    fn update(&mut self) {
        self.position += self.velocity;
        self.lifespan -= 0.01;

        // Wrap around screen
        if self.position.x > screen_width() {
            self.position.x = 0.0;
        } else if self.position.x < 0.0 {
            self.position.x = screen_width();
        }

        if self.position.y > screen_height() {
            self.position.y = 0.0;
        } else if self.position.y < 0.0 {
            self.position.y = screen_height();
        }
    }

    /// Draw bullet.
    fn draw(&self) {
        draw_circle(self.position.x, self.position.y, 2., WHITE);
    }
}

impl Collidable for Bullet {
    fn is_colliding(&self, other: &dyn Collidable) -> bool {
        <dyn Collidable>::circle_circle_intersection(self, other)
    }

    fn get_position(&self) -> Vec2 {
        self.position
    }

    fn get_radius(&self) -> f32 {
        2.0
    }
}

/// Particle object
struct Particle {
    position: Vec2,
    velocity: Vec2,
    lifespan: f32,
    decay: f32,
}

impl Particle {
    /// Spawn new particle at a given position.
    fn spawn_new(position: Vec2, velocity: Vec2, lifespan: f32, decay: f32) -> Self {
        Self {
            position,
            velocity,
            lifespan,
            decay,
        }
    }

    /// Spawn particles in a radial pattern.
    fn spawn_radial(position: Vec2, count: u32) -> Vec<Particle> {
        let mut particles = Vec::new();

        for _ in 0..count {
            let direction = rand::gen_range(0.0, std::f32::consts::PI * 2.0);
            let speed = rand::gen_range(0.4, 1.0);
            let velocity = Mat2::from_angle(direction).mul_vec2(Vec2::X * speed);

            particles.push(Self::spawn_new(position, velocity, rand::gen_range(0.2, 1.0), 0.01));
        }

        particles
    }

    /// Spawn particles in a conical pattern.
    fn spawn_conical(position: Vec2, direction: f32, spread: f32, count: u32) -> Vec<Particle> {
        let mut particles = Vec::new();
    
        for _ in 0..count {
            // Generate a random direction within the specified spread
            let spread_angle = rand::gen_range(-spread / 2.0, spread / 2.0);
            let cone_direction = direction + spread_angle;
    
            // Generate a random speed within a range
            let speed = rand::gen_range(0.4, 1.0);
    
            // Calculate velocity based on the cone direction and speed
            let velocity = Mat2::from_angle(cone_direction).mul_vec2(Vec2::Y * speed);
    
            particles.push(Self::spawn_new(position, velocity, rand::gen_range(0.2, 1.0), 0.01));
        }
    
        particles
    }
    
    /// Spawn larger particles with a quicker expiration in a radial pattern.
    fn spawn_debris(position: Vec2, count: u32) -> Vec<Particle> {
        // let mut rng = ::rand::thread_rng();
        let mut particles = Vec::new();

        for _ in 0..count {
            let direction = rand::gen_range(0.0, std::f32::consts::PI * 2.0);
            let speed = rand::gen_range(0.4, 1.0);
            let velocity = Mat2::from_angle(direction).mul_vec2(Vec2::X * speed);

            particles.push(Self::spawn_new(position, velocity, rand::gen_range(2.0, 5.0), 0.1));
        }

        particles
    }

    /// Destroy particle by marking it dead. Any calls to `is_alive` will return
    /// false after this function is called.
    fn destroy(&mut self) {
        self.lifespan = 0.0;
    }

    /// Check if particle is still alive.
    fn is_alive(&self) -> bool {
        self.lifespan > 0.0
    }

    /// Update particle position and lifespan.
    fn update(&mut self) {
        self.position += self.velocity;
        self.lifespan -= self.decay;
    }

    /// Draw particle.
    fn draw(&self) {
        draw_circle(self.position.x, self.position.y, 1.0 + self.lifespan, WHITE);
    }
}

#[derive(PartialEq)]
enum GameState {
    AttractMode,
    Playing,
    GameOver,
}

/// Game world object
/// 
/// The game world contains all game objects. It is responsible for updating and
/// drawing all objects.
struct GameWorld {
    ship: Ship,
    asteroids: Vec<Asteroid>,
    particles: Vec<Particle>,
    player_bullets: Vec<Bullet>,    
    player_lives: u32,
    player_score: u32,
    wave_number: u32,
    font: Font,
    game_state: GameState,
}

impl GameWorld {
    /// Create a new instance of the GameWorld object.
    // fn new() -> Self {
    fn new(font: Font) -> Self {
        Self {
            ship: Ship::spawn_new(),
            asteroids: Vec::new(),
            particles: Vec::new(),
            player_bullets: Vec::new(),
            wave_number: 0,
            player_lives: 3,
            player_score: 0,
            font,
            game_state: GameState::AttractMode,
        }
    }

    /// Update game world and render.
    fn do_frame(&mut self) {
        match self.game_state {
            GameState::AttractMode => {
                self.game_attract_mode();
            },
            GameState::Playing => {
                self.game_play_mode();
            },
            GameState::GameOver => {
                self.game_over_mode();
            },
        }
    }
    
    /// Check running in attract mode.
    fn game_attract_mode(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            self.start();
        }

        self.update();
        self.draw();
    }

    /// Game running in play mode.
    fn game_play_mode(&mut self) {
        if !self.ship.is_respawning() {
            self.input();
        }

        self.update();
        self.draw();
    }

    /// Game running in game over mode.
    fn game_over_mode(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            self.game_state = GameState::AttractMode;
        }

        self.update();
        self.draw();
    }

    /// Check if we're playing.
    fn is_playing(&self) -> bool {
        self.game_state == GameState::Playing
    }

    /// Check if we're dead.
    fn is_game_over(&self) -> bool {
        self.game_state == GameState::GameOver
    }

    /// Check if we're in attract mode.
    fn is_attract_mode(&self) -> bool {
        self.game_state == GameState::AttractMode
    }

    /// Start attract mode.
    fn attract_mode(&mut self) {        
        self.asteroids.clear();

        for _ in 0..20 {
            let size = match rand::gen_range(0, 3) {
                0 => AsteroidSize::Small,
                1 => AsteroidSize::Medium,
                2 => AsteroidSize::Large,
                _ => AsteroidSize::Small,
            };
            
            self.asteroids.push(Asteroid::spawn_new(size));
        }

        self.game_state = GameState::AttractMode;
    }

    /// Start a new game.
    fn start(&mut self) {
        self.player_lives = 3;
        self.player_score = 0;
        self.ship.reset();

        self.wave_number = 0;
        self.next_wave();

        self.game_state = GameState::Playing;
    }

    /// Start a new wave.
    fn next_wave(&mut self) {
        self.wave_number += 1;

        self.asteroids.clear();

        for _ in 0..self.wave_number + 4 {
            self.asteroids.push(Asteroid::spawn_new(AsteroidSize::Large));
        }
    }

    /// Handle player input.
    fn input(&mut self) {        
        // Steering
        if is_key_down(KeyCode::Left) {
            self.ship.steer(-0.1);
        } else if is_key_down(KeyCode::Right) {
            self.ship.steer(0.1);
        } else {
            self.ship.steer(0.0);
        }

        // Thrust and acceleration
        if is_key_down(KeyCode::Up) {
            self.ship.thrust();

            self.particles.append(&mut Particle::spawn_conical(self.ship.get_exhaust_position(), self.ship.rotation, 0.5, 1));
        }

        // Shooting
        if is_key_pressed(KeyCode::Space) {
            if let Some(bullet) = self.ship.shoot() {
                self.player_bullets.push(bullet);
            }
        }
    }

    /// Draw all game objects.
    fn draw(&self) {        
        // Draw ship if we're playing
        if self.is_playing() {
            self.ship.draw();
        }

        // Draw bullets
        for bullet in &self.player_bullets {
            bullet.draw();
        }

        // Draw asteroids
        for asteroid in &self.asteroids {
            asteroid.draw();
        }

        // Draw particles
        for particle in &self.particles {
            particle.draw();
        }

        // Draw HUD text
        if self.is_playing() {
            // Draw score
            draw_text_ex(
                &format!("Score: {}", self.player_score), 80.0, 40.0,            
                TextParams {
                    font_size: 30,
                    font: Some(&self.font),
                    ..Default::default()
                },
            );

            // Draw lives
            draw_text_ex(
                &format!("Lives: {}", self.player_lives), 80.0, 80.0,            
                TextParams {
                    font_size: 30,
                    font: Some(&self.font),
                    ..Default::default()
                },
            );

            // Draw wave number
            draw_text_ex(
                &format!("Wave: {}", self.wave_number), screen_width() * 0.75, 40.0,
                TextParams {
                    font_size: 30,
                    font: Some(&self.font),
                    ..Default::default()
                },
            );
        }

        // Draw game over if we're dead
        if self.is_game_over() {
            let text_size = measure_text("Game Over", Some(&self.font), 60, 1.0);    
            // let text_size = measure_text("Game Over", None, 60, 1.0);    
            draw_text_ex(
                "Game Over", (screen_width() - text_size.width) / 2.0, screen_height() / 2.0,
                TextParams {
                    font_size: 60,
                    font: Some(&self.font),
                    ..Default::default()
                },
            );
        }

        // Draw attract mode text
        if self.is_attract_mode() {
            let text_size = measure_text("Asteroids", Some(&self.font), 90, 1.0);
            // let text_size = measure_text("Asteroids", None, 90, 1.0);    
            draw_text_ex(
                "Asteroids", (screen_width() - text_size.width) / 2.0, screen_height() / 2.0,
                TextParams {
                    font_size: 90,
                    font: Some(&self.font),
                    ..Default::default()
                },
            );

            let text_size = measure_text("Press [SPACE] to Start", Some(&self.font), 40, 1.0);    
            // let text_size = measure_text("Press [SPACE] to Start", None, 40, 1.0);    
            draw_text_ex(
                "Press [SPACE] to Start", (screen_width() - text_size.width) / 2.0, screen_height() - 50.0,
                TextParams {
                    font_size: 40,
                    font: Some(&self.font),
                    ..Default::default()
                },
            );
        }

    }

    /// Update all game objects.
    fn update(&mut self) {
        // Update ship
        self.ship.update();

        // Update bullets
        for bullet in &mut self.player_bullets {
            bullet.update();
        }

        // Update asteroids
        for asteroid in &mut self.asteroids {
            asteroid.update();
        }

        // Update particles
        for particle in &mut self.particles {
            particle.update();
        }
        
        self.collision();

        // Remove dead bullets
        self.player_bullets.retain(|bullet| bullet.is_alive());

        // Remove dead asteroids
        self.asteroids.retain(|asteroid| asteroid.is_alive());

        // Remove dead particles
        self.particles.retain(|particle| particle.is_alive());

        // Check if all asteroids are destroyed
        if self.asteroids.len() == 0 {
            self.next_wave();
        }
    }

    /// Handle collisions between game objects.
    fn collision(&mut self) {
        // Only work out collision if we're playing
        if self.game_state == GameState::Playing {
            
            // New asteroids to spawn        
            let mut asteroid_spawns = Vec::new();
            
            // Collision loop
            for asteroid in &mut self.asteroids {
                
                // Ship to asteroid collision
                if self.ship.is_colliding(asteroid) {

                    self.particles.append(&mut Particle::spawn_radial(self.ship.position, 100));
                    self.particles.append(&mut Particle::spawn_debris(self.ship.position, 50));

                    if self.player_lives == 0 {
                        self.game_state = GameState::GameOver;
                    } else {
                        self.player_lives -= 1;
                        self.ship.respawn();
                    }
                }

                // Bullet to asteroid collision
                for bullet in &mut self.player_bullets {
                    if bullet.is_colliding(asteroid) {
                        
                        // Update score and spawn particles
                        match asteroid.size {
                            AsteroidSize::Small => {
                                self.player_score += 100;

                                self.particles.append(&mut Particle::spawn_radial(asteroid.position, 10));
                            },
                            AsteroidSize::Medium => {
                                self.player_score += 50;

                                asteroid_spawns.push(Asteroid::spawn_new_at(AsteroidSize::Small, asteroid.position));
                                asteroid_spawns.push(Asteroid::spawn_new_at(AsteroidSize::Small, asteroid.position));

                                self.particles.append(&mut Particle::spawn_radial(asteroid.position, 20));
                                self.particles.append(&mut Particle::spawn_debris(asteroid.position, 5));
                            },
                            AsteroidSize::Large => {
                                self.player_score += 20;

                                asteroid_spawns.push(Asteroid::spawn_new_at(AsteroidSize::Medium, asteroid.position));
                                asteroid_spawns.push(Asteroid::spawn_new_at(AsteroidSize::Medium, asteroid.position));
                                
                                self.particles.append(&mut Particle::spawn_radial(asteroid.position, 30));
                                self.particles.append(&mut Particle::spawn_debris(asteroid.position, 10));
                            },
                        }

                        // Destroy asteroid and bullet
                        asteroid.destroy();
                        bullet.destroy();

                        
                    }
                }
            }

            // Add newly spawned asteroids to current asteroid list
            self.asteroids.append(&mut asteroid_spawns);        
        }
    }

}

#[macroquad::main("Asteroids")]
async fn main() {
    let font = load_ttf_font("./Hyperspace.ttf")
        .await
        .unwrap();

    let mut game = GameWorld::new(font);
    // let mut game = GameWorld::new(None);

    // Start in attact mode
    game.attract_mode();

    loop {
        clear_input_queue();
        clear_background(BLACK);

        // Do a game frame
        game.do_frame();

        next_frame().await
    }
}