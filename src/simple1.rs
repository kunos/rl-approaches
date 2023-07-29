use crate::math::{vec2, Vec2};

// Map is intended for stuff that does not move nor update and it is optimized to be retrieved by position ie. a grid of size x size (water, walls, floors, doors?)
struct Map {
    size: usize,
    map: Vec<MapClass>,
}

impl Map {
    fn new(size: usize) -> Self {
        let map = Vec::with_capacity(size * size);
        Map { size, map }
    }

    fn class_at(&self, pos: Vec2) -> Option<MapClass> {
        todo!()
    }

    // Check if a position can be occupied by an Entity ie. maybe there's a wall there?
    fn validate_move(&self, pos: Vec2) -> bool {
        if let Some(class) = self.class_at(pos) {
            match class {
                MapClass::Wall | MapClass::ClosedDoor | MapClass::None => false,
                _ => true,
            }
        } else {
            true
        }
    }
}

enum MapClass {
    None,
    Water,
    Floor,
    Wall,
    ClosedDoor,
    OpenDoor,
}

#[derive(Clone, Copy)]
struct PlayerData {}

#[derive(Clone, Copy)]
struct MonsterData {}

#[derive(Clone, Copy)]
struct ItemData {}

type EntityId = usize;

#[derive(Clone, Copy)]
enum EntityClass {
    Player(PlayerData),
    Monster(MonsterData),
    Item(ItemData),
}

// Entities are things that move and update themselves (Players, Monsters etc.)
#[derive(Clone, Copy)]
struct Entity {
    pos: Vec2,
    health: f32,
    class: EntityClass,
}

impl Entity {
    fn update(&mut self, my_id: EntityId, entities: &[Entity], map: &Map) -> Vec<SideEffect> {
        match &mut self.class {
            EntityClass::Player(_player) => {
                // These 2 consts could be coming from the PlayerData
                const ATTACK_DISTANCE: f32 = 2.0;
                const MY_ATTACK_STRENGTH: f32 = 0.25;

                // Example move
                let new_pos = vec2(self.pos.x + 0.1, self.pos.y);

                if map.validate_move(new_pos) {
                    // We can move there
                    self.pos = new_pos;
                }

                // Example Attack anything that is close enough to us
                let mut side_effects = Vec::new();
                entities
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != my_id)
                    .for_each(|(i, e)| {
                        if self.pos.distance(e.pos) < ATTACK_DISTANCE {
                            side_effects.push(SideEffect::Attack {
                                entity0: my_id,
                                entity1: i,
                                strength: MY_ATTACK_STRENGTH,
                            });
                        }
                    });

                side_effects
            }
            EntityClass::Monster(_) => todo!(),
            EntityClass::Item(_) => todo!(),
        }
    }
}

#[derive(Clone, Copy)]
enum SideEffect {
    Attack {
        entity0: EntityId,
        entity1: EntityId,
        strength: f32,
    },
    MapAttack {
        // This is something that will influence the map
        entity0: EntityId,
        map_pos: Vec2,
        strength: f32,
    },
}

struct Game {
    map: Map,
    entities: Vec<Entity>,
}

impl Game {
    fn new() -> Self {
        let map = Map::new(1024);
        let entities = Vec::new();

        Game { map, entities }
    }

    fn apply_side_effect(&mut self, effect: SideEffect) {
        match effect {
            SideEffect::Attack {
                entity1, strength, ..
            } => {
                self.entities[entity1].health -= strength;
            }
            SideEffect::MapAttack {
                entity0,
                map_pos,
                strength,
            } => {
                // We are &mut self so map is mut and we can change it as we like ie. change a Wall into a Floor or into a DestroyedWall
                todo!()
            }
        }
    }

    fn update(&mut self) -> bool {
        // As long as Entity is trivial to clone this is ok and will pay dividends later if the update map
        // is called in parallel with rayon
        let entities0 = self.entities.clone();

        // Update entities and collect side effects vectors
        let side_effects: Vec<Vec<SideEffect>> = self
            .entities
            .iter_mut()
            .enumerate()
            .map(|(i, e)| e.update(i, &entities0, &self.map))
            .collect();

        // Apply side effects
        side_effects.iter().for_each(|eff| {
            eff.iter().for_each(|side_effect| {
                self.apply_side_effect(*side_effect);
            })
        });

        // Last step, kill entities. After this line all the EntityIds are to be considered invalid
        self.entities.retain(|e| e.health <= 0.0);

        // TODO
        // Because EntityId is simply an index into the self.entity vector these Ids are only valid within a frame
        // Thus entities cannot store an Id in them for complex logic that extends more than 1 frame
        // If that is a necessity then sommething else needs to be used (Generational Indices, maps etc)

        // Keep running if there are at least 2 entities alive
        self.entities.len() > 1
    }

    // An alternative way to update entities is by using indices everywhere
    // Pros:
    // - everything can be done in one pass, including map alternations
    // Cons:
    // - very error prone and verbose with self.entities[x] everywhere
    // - impossible to execute in parallel
    // - the entire Entity update logic goes in this func
    fn update_indexed(&mut self) -> bool {
        for i in 0..self.entities.len() {
            match self.entities[i].class {
                // These 2 consts could be coming from the PlayerData
                EntityClass::Player(player) => {
                    const ATTACK_DISTANCE: f32 = 2.0;
                    const MY_ATTACK_STRENGTH: f32 = 0.25;

                    // Do bad things to other entities
                    for ei in 0..self.entities.len() {
                        // Check it's not me
                        if i != ei {
                            if self.entities[i].pos.distance(self.entities[ei].pos)
                                < ATTACK_DISTANCE
                            {
                                // Just do it
                                self.entities[ei].health -= MY_ATTACK_STRENGTH;
                            }
                        }
                    }
                }
                EntityClass::Monster(_) => todo!(),
                EntityClass::Item(_) => todo!(),
            }
        }

        // Last step, kill entities.
        self.entities.retain(|e| e.health <= 0.0);

        self.entities.len() > 1
    }
}

pub fn run() {
    let mut game = Game::new();

    while game.update() {}
}
