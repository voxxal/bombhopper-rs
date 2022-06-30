//! Bombhopper Api for rust
use std::{
    collections::HashMap,
    ops::{Add, Mul, Sub},
};

use serde::Serialize;

#[derive(Serialize, PartialEq, Default)]
pub struct Point {
    x: f32,
    y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for Point {
    type Output = Self;

    fn mul(self, other: f32) -> Self::Output {
        Point {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AmmoType {
    Empty,
    #[serde(rename = "bullet")]
    Bomb,
    Grenade,
}

#[derive(Serialize)]
pub enum Ammo {
    #[serde(rename = "infiniteAmmo")]
    Infinite(AmmoType),

    /// Note that finite is reversed, so if an ammo is in front, it'll be fired last
    #[serde(rename = "magazine")]
    Finite(Vec<AmmoType>),
}

impl Ammo {
    pub fn finite_seq(s: &str) -> Result<Self, String> {
        let mut mag = vec![];
        for c in s.chars().rev() {
            mag.push(match c.to_ascii_lowercase() {
                'b' => AmmoType::Bomb,
                'g' => AmmoType::Grenade,
                'e' => AmmoType::Empty,
                // TODO potentially migrate to Error enum
                _ => return Err(String::from("Ammo Doesn't exist")),
            })
        }
        Ok(Self::Finite(mag))
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum Shape {
    Polygon { vertices: Vec<Point> },
    Circle { x: f32, y: f32, radius: f32 },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

macro_rules! define_entities {
    ( $( $material: ident),* ) => {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase", tag = "type", content = "params")]
        pub enum Entity {
            #[serde(rename_all = "camelCase")]
            Player {
                is_static: bool,
                angle: i32,
                x: f32,
                y: f32,
                #[serde(flatten)]
                ammo: Ammo,
            },
            #[serde(rename_all = "camelCase", rename = "endpoint")]
            Door {
                is_static: bool,
                angle: i32,
                x: f32,
                y: f32,
                right_facing: bool,
            },
            #[serde(rename_all = "camelCase")]
            Text {
                angle: i32,
                x: f32,
                y: f32,
                // TODO maybe use &str and deal with lifetimes
                #[serde(rename = "copy")]
                text: HashMap<String, String>,
                anchor: Point,
                align: TextAlign,
                fill_color: i32,
                opacity: f32,
            },
            #[serde(rename_all = "camelCase")]
            Paint {
                fill_color: i32,
                opacity: f32,
                vertices: Vec<Point>,
            },
            $(
            #[serde(rename_all = "camelCase")]
            $material {
                is_static: bool,
                #[serde(flatten)]
                shape: Shape,
            },
            )*
        }
    };
}

define_entities!(Normal, Ice, Breakable, Deadly, Bouncy);

impl Entity {
    pub fn new_text(pos: Point, text: &str) -> Self {
        Self::Text {
            angle: 0,
            x: pos.x,
            y: pos.y,
            text: HashMap::from([(String::from("en"), text.to_string())]),
            anchor: Point::new(0.5, 0.5),
            align: TextAlign::Left,
            fill_color: 16777215,
            opacity: 1.0,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    pub name: String,
    pub timings: [i32; 2],
    pub entities: Vec<Entity>,
    format_version: u8,
}

impl Level {
    pub fn new(name: String, timings: [i32; 2]) -> Self {
        Self {
            name,
            timings,
            entities: vec![],
            format_version: 0,
        }
    }
    /// Pushes entity onto entities vector
    pub fn push(&mut self, entity: Entity) {
        self.entities.push(entity);
    }
    /// Clears all entities from entities vector
    pub fn clear(&mut self) {
        self.entities.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn ammo() {
        // Tests if infinte bombs goes correctly
        assert_eq!(
            r#"{"type":"player","params":{"isStatic":false,"angle":0,"x":0.0,"y":0.0,"infiniteAmmo":"bullet"}}"#,
            serde_json::to_string(&Entity::Player {
                is_static: false,
                angle: 0,
                x: 0.0,
                y: 0.0,
                ammo: Ammo::Infinite(AmmoType::Bomb)
            })
            .unwrap()
        );

        // Tests if finite seq works (and finite)
        assert_eq!(
            r#"{"type":"player","params":{"isStatic":false,"angle":0,"x":0.0,"y":0.0,"magazine":["grenade","empty","bullet","bullet"]}}"#,
            serde_json::to_string(&Entity::Player {
                is_static: false,
                angle: 0,
                x: 0.0,
                y: 0.0,
                ammo: Ammo::finite_seq("bbeg").unwrap()
            })
            .unwrap()
        )
    }

    #[test]
    fn default_level() {
        let mut level = Level::new(String::from("My level"), [0, 0]);
        level.push(Entity::new_text(
            Point::new(200.0, 520.0),
            "This is the default level!\nEdit to your liking",
        ));

        level.push(Entity::Normal {
            is_static: true,
            shape: Shape::Polygon {
                vertices: vec![
                    Point::new(400.0, 820.0),
                    Point::new(400.0, 880.0),
                    Point::new(520.0, 880.0),
                    Point::new(520.0, 820.0),
                ],
            },
        });

        level.push(Entity::Ice {
            is_static: true,
            shape: Shape::Polygon {
                vertices: vec![
                    Point::new(-260.0, 580.0),
                    Point::new(-260.0, 820.0),
                    Point::new(400.0, 820.0),
                    Point::new(400.0, 760.0),
                    Point::new(160.0, 760.0),
                    Point::new(-20.0, 640.0),
                    Point::new(-140.0, 640.0),
                ],
            },
        });

        level.push(Entity::Door {
            is_static: true,
            angle: 0,
            x: 550.0,
            y: 630.0,
            right_facing: true,
        });

        level.push(Entity::Player {
            is_static: false,
            angle: 0,
            x: -60.0,
            y: 620.0,
            ammo: Ammo::finite_seq("beg").unwrap(),
        });

        level.push(Entity::Normal {
            is_static: false,
            shape: Shape::Polygon {
                vertices: vec![
                    Point::new(-236.0, 292.0),
                    Point::new(-176.0, 292.0),
                    Point::new(-176.0, 352.0),
                    Point::new(-236.0, 352.0),
                ],
            },
        });

        assert_eq!(
            r#"{"name":"My level","timings":[0,0],"entities":[{"type":"text","params":{"angle":0,"x":200.0,"y":520.0,"copy":{"en":"This is the default level!\nEdit to your liking"},"anchor":{"x":0.5,"y":0.5},"align":"left","fillColor":16777215,"opacity":1.0}},{"type":"normal","params":{"isStatic":true,"vertices":[{"x":400.0,"y":820.0},{"x":400.0,"y":880.0},{"x":520.0,"y":880.0},{"x":520.0,"y":820.0}]}},{"type":"ice","params":{"isStatic":true,"vertices":[{"x":-260.0,"y":580.0},{"x":-260.0,"y":820.0},{"x":400.0,"y":820.0},{"x":400.0,"y":760.0},{"x":160.0,"y":760.0},{"x":-20.0,"y":640.0},{"x":-140.0,"y":640.0}]}},{"type":"endpoint","params":{"isStatic":true,"angle":0,"x":550.0,"y":630.0,"rightFacing":true}},{"type":"player","params":{"isStatic":false,"angle":0,"x":-60.0,"y":620.0,"magazine":["grenade","empty","bullet"]}},{"type":"normal","params":{"isStatic":false,"vertices":[{"x":-236.0,"y":292.0},{"x":-176.0,"y":292.0},{"x":-176.0,"y":352.0},{"x":-236.0,"y":352.0}]}}],"formatVersion":0}"#,
            serde_json::to_string(&level).unwrap()
        );
    }
}
