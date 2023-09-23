use macroquad::prelude::*;

/// Collidable trait
/// 
/// This trait is used to determine if two objects are colliding. It is used by
/// the collision detection system to determine if two objects are colliding with
/// each other even when straddling the edge of the screen (due to wrapping).
/// 
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

/// Asteroid size
/// 
/// Asteroids come in three sizes: small, medium, and large. The size determines
/// the diameter, number of sides, and angular velocity of the asteroid.
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

/// SaucerSize
/// 
/// Saucers come in two sizes: small and large. The size determines the visual representation
/// of the saucer as well as its logic. Small saucers are faster and aim at the player, while
/// large saucers are slower and shoot in random directions.
enum SaucerSize {
    Small,
    Large,
}

/// Saucer Object
///
/// Saucers move from left to right or right to left, and shoot bullets at the player. They
/// wrap around the screen when they reach the edge. They can change direction periodically.
/// The direction change is always less that 10 degrees. Saucers come in two sizes: small and
/// large. Small saucers are faster and aim at the player, while large saucers are slower and
/// shoot in random directions. 
struct Saucer {
    size: SaucerSize,
    diameter: f32,
    position: Vec2,
    velocity: Vec2,
    direction: f32,
    direction_change_period: f64,
    shoot_period: f64,
    vertices: Vec<Vec2>,
    is_alive: bool,
}

impl Saucer {    
    /// Spawn new saucer
    fn spawn_new(size: SaucerSize) -> Self {
        let screen_edge: f32 = std::cmp::min(screen_width() as i32, screen_height() as i32) as f32;
        
        // Diameter magic numbers for asteroid sizes
        let diameter = match size {
            SaucerSize::Small => screen_edge * 0.035,
            SaucerSize::Large => screen_edge * 0.07,
        };
        
        let speed = match size {
            SaucerSize::Small => screen_edge * 0.0025,
            SaucerSize::Large => screen_edge * 0.00125,
        };

        let (position, direction) = match rand::gen_range(0, 2) { 
            0 => (Vec2::new(0., rand::gen_range(0.0, screen_height())), 0.0),
            1 => (Vec2::new(screen_width(), rand::gen_range(0.0, screen_height())), std::f32::consts::PI),
            _ => (Vec2::new(0., 0.), 0.0),
        };

        // Generate random direction and velocity
        let velocity = Mat2::from_angle(direction).mul_vec2(Vec2::X * speed);

        // Generate vertices
        let radius = diameter / 2.0;
        let mut vertices: Vec<Vec2> = Vec::new();
        vertices.push(Vec2::new(-radius * 1.25, 0.0));
        vertices.push(Vec2::new(-radius / 2.0, radius / 2.0));
        vertices.push(Vec2::new(radius / 2.0, radius / 2.0));
        vertices.push(Vec2::new(radius * 1.25, 0.0));
        vertices.push(Vec2::new(-radius * 1.25, 0.0));
        vertices.push(Vec2::new(-radius / 2.0, -radius / 2.0));
        vertices.push(Vec2::new(-radius / 3.0, -radius));
        vertices.push(Vec2::new(radius / 3.0, -radius));
        vertices.push(Vec2::new(radius / 2.0, -radius / 2.0));
        vertices.push(Vec2::new(radius * 1.25, 0.0));
        vertices.push(Vec2::new(radius / 2.0, -radius / 2.0));
        vertices.push(Vec2::new(-radius / 2.0, -radius / 2.0));        

        Self {
            size,
            diameter,
            position,
            velocity,
            direction,
            direction_change_period: get_time() + 1.0,
            shoot_period: get_time() + 1.0,
            vertices,
            is_alive: true,
        }
    }

    /// Destroy saucer by marking it dead. Any calls to `is_alive` will return
    /// false after this function is called.
    fn destroy(&mut self) {
        self.is_alive = false;
    }

    /// Check if saucer is still alive.
    fn is_alive(&self) -> bool {
        self.is_alive
    }

    /// Shoot bullet. Saucers shoot bullets at the player. Small saucers aim at the
    /// player, while large saucers shoot in random directions.
    fn shoot(&mut self, position: Vec2) -> Option<Bullet> {
        // Decide if we should shoot
        if self.shoot_period < get_time() {            
            
            // Reset period
            self.shoot_period = get_time() + 1.0;

            // Shoot
            if rand::gen_range(0.0, 1.0) > 0.5 {                
                match self.size {
                    SaucerSize::Small => {
                        let velocity = (position - self.position).normalize() * 2.0;
                        return Some(Bullet::spawn_new(self.position, velocity, 100.0, BulletType::Enemy))
                    },
                    SaucerSize::Large => {
                        let direction = rand::gen_range(0.0, 2.0 * std::f32::consts::PI);
                        let velocity = Mat2::from_angle(direction).mul_vec2(Vec2::X * 2.0);
                        return Some(Bullet::spawn_new(self.position, velocity, 100.0, BulletType::Enemy))
                    },
                };
            }
        }

        None
    }

    /// Update saucer position
    fn update(&mut self) {
        self.position += self.velocity;

        // Navigation check
        if self.direction_change_period < get_time() {
            
            // Reset period
            self.direction_change_period = get_time() + 1.0;

            // Change direction?
            if rand::gen_range(0.0, 1.0) > 0.5 {
                self.direction += rand::gen_range(-1.0, 1.0) * 10.0 / 180.0 * std::f32::consts::PI;
                self.velocity = Mat2::from_angle(self.direction).mul_vec2(Vec2::X * self.velocity.length());
            }
        }

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

    /// Draw saucer.    
    fn draw(&self) {
        // Draw asteroid
        self.draw_vertices_at(self.position, &self.vertices);

        // Calculate radius
        let radius = self.diameter / 2.0;

        // Horizontal overlaps
        if self.position.x > screen_width() - radius {
            self.draw_vertices_at(Vec2::new(self.position.x - screen_width(), self.position.y), &self.vertices);
        } else if self.position.x < radius {
            self.draw_vertices_at(Vec2::new(self.position.x + screen_width(), self.position.y), &self.vertices);
        }
        
        // Vertical overlaps
        if self.position.y > screen_height() - radius {
            self.draw_vertices_at( Vec2::new(self.position.x, self.position.y - screen_height()), &self.vertices);
        } else if self.position.y < radius {
            self.draw_vertices_at(Vec2::new(self.position.x, self.position.y + screen_height()), &self.vertices);
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

impl Collidable for Saucer {
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
    hyperspace_cooldown: f64,
    hyperspace_recharge: f64,
    shot_cooldown: f64,
    shot_recharge: f64,
    shot_speed: f32,
    shot_lifespan: f32,
    respawn_lifespan: f64,
    shield_lifespan: f64,
    vertices: Vec<Vec2>,    
}

impl Ship {
    /// Construct ship object
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
            hyperspace_cooldown: 0.0,
            hyperspace_recharge: 5.0,
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

    /// Respawn ship.
    /// 
    /// When player dies, respawn the ship after a short delay. The ship will be
    /// invulnerable for a short period of time after respawning.
    fn respawn(&mut self) {
        self.respawn_lifespan = get_time() + 2.0;
        self.shield_lifespan = self.respawn_lifespan + 2.0;

        self.reset();
    }

    /// Reset player position and velocity.
    fn reset(&mut self) {
        self.position = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
        self.velocity = Vec2::new(0., 0.);
        self.rotation = 0.0;
        self.rotation_speed = 0.0;
    }

    /// Check if ship is still during respawn period.
    fn is_respawning(&self) -> bool {
        get_time() < self.respawn_lifespan
    }

    /// Check if shield is still active.
    fn is_shield_active(&self) -> bool {
        get_time() < self.shield_lifespan
    }

    /// Get position of exhaust. This is used to fire particles when the ship is
    /// accelerating.
    fn get_exhaust_position(&self) -> Vec2 {
        self.position + Mat2::from_angle(self.rotation).mul_vec2(self.vertices[2])
    }

    /// Activate hyperspace. This teleports the ship to a random location on the
    /// screen.
    fn hyperspace(&mut self) -> Option<Vec2> {
        let current_time = get_time();

        // Make sure we're not in cooldown
        if self.hyperspace_cooldown < current_time {
            let old_position = self.position.clone();

            self.hyperspace_cooldown = current_time + self.hyperspace_recharge;
            self.position = Vec2::new(rand::gen_range(0.0, screen_width()), rand::gen_range(0.0, screen_height()));
        
            Some(old_position)
        } else {
            None
        }
    }

    /// Accelerate ship in direction of rotation
    fn thrust(&mut self) {
        let rotation_matrix = Mat2::from_angle(self.rotation);
        self.velocity += rotation_matrix.mul_vec2(Vec2::new(0., -self.thrust));

        if self.velocity.length() > self.max_speed {
            self.velocity = self.velocity.normalize() * self.max_speed;
        }
    }

    /// Steer ship
    fn steer(&mut self, direction: f32) {
        self.rotation_speed = direction;
    }

    /// Shoot bullet
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
        
        Some(Bullet::spawn_new(position, velocity, self.shot_lifespan, BulletType::Player))
    }

    /// Update ship position and rotation
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

    /// Render ship
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

/// Bullet type
/// 
/// Bullets come in two types: player and enemy. Player bullets are smaller and
/// have a shorter lifespan. They also wrap around the screen when they reach the
/// edge. Enemy bullets are larger and have a longer lifespan. They disappear when
/// they reach the edge.
/// 
#[derive(PartialEq)]
enum BulletType {
    Player,
    Enemy,
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
    bullet_type: BulletType,
}

impl Bullet {
    /// Spawn new bullet at a given position.
    fn spawn_new(position: Vec2, velocity: Vec2, lifespan: f32, bullet_type: BulletType) -> Self {
        Self {
            position,
            velocity,
            lifespan,
            bullet_type,
        }
    }

    /// Destroy bullet by marking it dead. Any calls to `is_alive` will return
    /// false after this function is called.
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

        // Handle screen edges   
        if self.bullet_type == BulletType::Player {
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
        } else {
            if self.position.x > screen_width() || self.position.x < 0.0 || 
                self.position.y > screen_height() || self.position.y < 0.0 
            {
                self.lifespan = 0.0;
            }
        }
    }

    /// Draw bullet.
    fn draw(&self) {
        if self.bullet_type == BulletType::Player {
            draw_circle(self.position.x, self.position.y, 2., WHITE);
        } else {
            draw_circle(self.position.x, self.position.y, 3., WHITE);
        }
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
/// 
/// Particles are spawned when objects are destroyed. They move in a random
/// direction, and disappear after a certain amount of time. 
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

    /// Spawn larger particles with a quicker expiration in a radial pattern.
    fn spawn_ring(position: Vec2, radius: f32, count: u32) -> Vec<Particle> {
        // let mut rng = ::rand::thread_rng();
        let mut particles = Vec::new();

        for p in 0..count {
            let direction = std::f32::consts::PI * 2.0 / count as f32 * p as f32;
            let speed = rand::gen_range(0.4, 1.0);
            let velocity = Mat2::from_angle(direction).mul_vec2(Vec2::X * speed);

            particles.push(Self::spawn_new(position - velocity * radius, velocity, rand::gen_range(0.2, 1.0), 0.025));
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

/// Game state
/// 
/// The game can be in one of three states: attract mode, playing, or game over.
/// Attract mode is the initial state, and is entered when the game starts. The
/// game will return to attract mode when the player dies. The game will enter
/// play mode when the player presses the space bar. The game will enter game
/// over mode when the player loses all lives.
/// 
#[derive(PartialEq)]
enum GameState {
    AttractMode,
    Playing,
    GameOver,
}

/// Game input
/// 
/// The game input is used to control the ship. The ship can be steered left or
/// right, and can be accelerated. The ship can also shoot bullets.
/// 
enum GameInput {
    Left,
    Right,
    Thruster,
    Cannon,
    None
}

/// Game world object
/// 
/// The game world contains all game objects. It is responsible for updating and
/// drawing all objects.
struct GameWorld {
    ship: Ship,
    asteroids: Vec<Asteroid>,
    saucers:Vec<Saucer>,
    particles: Vec<Particle>,
    enemy_bullets: Vec<Bullet>,
    player_bullets: Vec<Bullet>,    
    player_lives: u32,
    player_score: u32,
    wave_number: u32,
    wave_spawn_time: f64,
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
            saucers: Vec::new(),
            particles: Vec::new(),
            enemy_bullets: Vec::new(),
            player_bullets: Vec::new(),
            player_lives: 0,
            player_score: 0,
            wave_number: 0,
            wave_spawn_time: 0.0,
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
    
    /// Game running in attract mode.
    fn game_attract_mode(&mut self) {
        if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left) || touches().len() > 0 {
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
        if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left) || touches().len() > 0 {
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

        self.saucers.clear();
        self.saucers.push(Saucer::spawn_new(SaucerSize::Large));

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

        self.saucers.clear();
        self.wave_spawn_time = get_time() + 10.0;
    }

    /// Handle player input.
    fn input(&mut self) {        
        
        // Steering
        let mut steering : GameInput = GameInput::None;
        
        // Translate inputs into steering
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_position = mouse_position();
            let mouse_direction = (Vec2::new(mouse_position.0, mouse_position.1) - self.ship.position).normalize();
            let ship_direction = Mat2::from_angle(self.ship.rotation).mul_vec2(Vec2::Y);
            let angle_difference = ship_direction.angle_between(mouse_direction);
        
            if angle_difference > 0.1 {
                steering = GameInput::Left;
            } else if angle_difference < -0.1 {
                steering = GameInput::Right;
            } 
        } else if is_key_down(KeyCode::Left) {
            steering = GameInput::Left;
        } else if is_key_down(KeyCode::Right) {
            steering = GameInput::Right;
        }
            
        // Steer ship
        match steering {
            GameInput::Left => {
                self.ship.steer(-0.1);
            },
            GameInput::Right => {
                self.ship.steer(0.1);
            },
            _ => {
                self.ship.steer(0.0);
            }
        }

        // Thrusters
        let mut thrusters : GameInput = GameInput::None;

        // Translate inputs into thrusters
        if is_mouse_button_down(MouseButton::Right) || is_key_down(KeyCode::Up) || touches().len() == 2 {
            thrusters = GameInput::Thruster;
        } 

        // Thrust and acceleration
        match thrusters {
            GameInput::Thruster => {
                self.ship.thrust();

                self.particles.append(&mut Particle::spawn_conical(self.ship.get_exhaust_position(), self.ship.rotation, 0.5, 1));
            },
            _ => { }
        }

        if is_key_down(KeyCode::Down) {
            if let Some(position) = self.ship.hyperspace() {
                self.particles.append(&mut Particle::spawn_ring(position, self.ship.radius * 6.0, 200));
                self.particles.append(&mut Particle::spawn_ring(self.ship.position, self.ship.radius * 6.0, 200));
            }
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

        // Draw enemy bullets
        for bullet in &self.enemy_bullets {
            bullet.draw();
        }

        // Draw asteroids
        for asteroid in &self.asteroids {
            asteroid.draw();
        }

        // Draw saucers
        for saucer in &self.saucers {
            saucer.draw();
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
            draw_text_ex(
                "Asteroids", (screen_width() - text_size.width) / 2.0, screen_height() / 2.0,
                TextParams {
                    font_size: 90,
                    font: Some(&self.font),
                    ..Default::default()
                },
            );

            let text_size = measure_text("Press [SPACE] to Start", Some(&self.font), 40, 1.0);    
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

        // Update player bullets
        for bullet in &mut self.player_bullets {
            bullet.update();
        }
        
        // Update enemy bullets
        for bullet in &mut self.enemy_bullets {
            bullet.update();
        }

        // Update asteroids
        for asteroid in &mut self.asteroids {
            asteroid.update();
        }

        // Update saucers
        for saucer in &mut self.saucers {
            if let Some(bullet) = saucer.shoot(self.ship.position) {
                self.enemy_bullets.push(bullet);
            }
            
            saucer.update();
        }

        // Update particles
        for particle in &mut self.particles {
            particle.update();
        }
        
        self.collision();

        // Remove dead player bullets
        self.player_bullets.retain(|bullet| bullet.is_alive());

        // Remove dead enemy bullets
        self.enemy_bullets.retain(|bullet| bullet.is_alive());

        // Remove dead asteroids
        self.asteroids.retain(|asteroid| asteroid.is_alive());

        // Remove dead saucers
        self.saucers.retain(|saucer| saucer.is_alive());

        // Remove dead particles
        self.particles.retain(|particle| particle.is_alive());

        // Check if all asteroids are destroyed
        if self.asteroids.len() + self.saucers.len() == 0 {
            self.next_wave();
        } else {
            // Spawn saucers
            let current_time = get_time();            
            if self.wave_spawn_time < current_time {
                self.wave_spawn_time = current_time + 10.0;

                if rand::gen_range(0.0, 1.0) > 0.75 {
                    if self.player_score < 10000 {
                        self.saucers.push(Saucer::spawn_new(SaucerSize::Large));
                    } else {
                        self.saucers.push(Saucer::spawn_new(SaucerSize::Small));
                    }
                }
            }
        }
    }

    /// Handle collisions between game objects.
    /// TODO: Refactor and clean up... there's a lot of repeated code to work with.
    fn collision(&mut self) {
        // Only work out collision if we're playing
        if self.game_state != GameState::Playing {
            return;
        }
       
        // Keep track of score to add a life if we reach a certain threshold
        let current_score = self.player_score / 10000;

        // New asteroids to spawn        
        let mut asteroid_spawns = Vec::new();
            
        // Collision loop
        for asteroid in &mut self.asteroids {
            
            // Ship to asteroid collision
            if self.ship.is_colliding(asteroid) {

                self.particles.append(&mut Particle::spawn_radial(self.ship.position, 100));
                self.particles.append(&mut Particle::spawn_debris(self.ship.position, 50));

                // Lose a life or game over if no more left
                if self.player_lives == 0 {
                    self.game_state = GameState::GameOver;
                } else {
                    self.player_lives -= 1;
                    self.ship.respawn();
                }
            }

            // Saucer to asteroid collisions
            for saucer in &mut self.saucers {

                // Do we have a collision?
                if saucer.is_colliding(asteroid) {                      

                    // Update score and spawn particles
                    match saucer.size {
                        SaucerSize::Small => {
                            self.player_score += 1000;

                            self.particles.append(&mut Particle::spawn_radial(saucer.position, 100));
                            self.particles.append(&mut Particle::spawn_debris(saucer.position, 50));
                        },
                        SaucerSize::Large => {
                            self.player_score += 200;
                            
                            self.particles.append(&mut Particle::spawn_radial(saucer.position, 200));
                            self.particles.append(&mut Particle::spawn_debris(saucer.position, 100));
                        },
                    }

                    // Destroy asteroid and saucer
                    saucer.destroy();

                    self.particles.append(&mut Particle::spawn_radial(asteroid.position, 100));
                    self.particles.append(&mut Particle::spawn_debris(asteroid.position, 50));

                    asteroid.destroy();
                }
            }
            
            // Collect player and enemy bullets that collide with asteroids
            let all_bullets = self.player_bullets.iter_mut().chain(self.enemy_bullets.iter_mut());

            // Bullet to asteroid collision
            for bullet in all_bullets { // &mut self.player_bullets {
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

        // Saucer to ship collision
        for saucer in &mut self.saucers {
            
            // Ship to saucer collision
            if self.ship.is_colliding(saucer) {
                // Update score and spawn particles
                match saucer.size {
                    SaucerSize::Small => {
                        self.player_score += 1000;

                        self.particles.append(&mut Particle::spawn_radial(saucer.position, 100));
                        self.particles.append(&mut Particle::spawn_debris(saucer.position, 50));
                    },
                    SaucerSize::Large => {
                        self.player_score += 200;
                        
                        self.particles.append(&mut Particle::spawn_radial(saucer.position, 200));
                        self.particles.append(&mut Particle::spawn_debris(saucer.position, 100));
                    },
                }

                // Destroy asteroid and bullet
                saucer.destroy();

                self.particles.append(&mut Particle::spawn_radial(self.ship.position, 100));
                self.particles.append(&mut Particle::spawn_debris(self.ship.position, 50));

                // Lose a life or game over if no more left
                if self.player_lives == 0 {
                    self.game_state = GameState::GameOver;
                } else {
                    self.player_lives -= 1;
                    self.ship.respawn();
                }
            }

            // Bullet to saucer collision
            for bullet in &mut self.player_bullets {
                if bullet.is_colliding(saucer) {
                    
                    // Update score and spawn particles
                    match saucer.size {
                        SaucerSize::Small => {
                            self.player_score += 1000;

                            self.particles.append(&mut Particle::spawn_radial(saucer.position, 100));
                            self.particles.append(&mut Particle::spawn_debris(saucer.position, 50));
                        },
                        SaucerSize::Large => {
                            self.player_score += 200;
                            
                            self.particles.append(&mut Particle::spawn_radial(saucer.position, 200));
                            self.particles.append(&mut Particle::spawn_debris(saucer.position, 100));
                        },
                    }

                    // Destroy asteroid and bullet
                    saucer.destroy();
                    bullet.destroy();
                }
            }
        }
        
        // Bullet to ship collisions
        for bullet in &mut self.enemy_bullets {
            if bullet.is_colliding(&self.ship) {

                self.particles.append(&mut Particle::spawn_radial(self.ship.position, 100));
                self.particles.append(&mut Particle::spawn_debris(self.ship.position, 50));

                // Destroy bullet
                bullet.destroy();

                // Lose a life or game over if no more left
                if self.player_lives == 0 {
                    self.game_state = GameState::GameOver;
                } else {
                    self.player_lives -= 1;
                    self.ship.respawn();
                }
            }
        }

        // Check if we need to add a life
        if self.player_score / 10000 > current_score {
            self.player_lives += 1;
        }
    }
}

/// App
/// 
/// The app is the entry point for the game. It creates a new game world and
/// runs the game loop. The game loop is responsible for updating and drawing
/// the game world.
/// 
#[macroquad::main("Asteroids")]
async fn main() {
    let font = load_ttf_font("./Hyperspace.ttf")
        .await
        .unwrap();

    // Construct game world; use loaded font for text rendering
    let mut game = GameWorld::new(font);

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