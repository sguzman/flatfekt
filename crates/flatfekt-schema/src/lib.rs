use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Scene {
    pub schema_version: Option<String>,
    pub entities: Option<Vec<EntitySpec>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EntitySpec {
    pub id: String,
    pub transform: Option<Transform2d>,
    pub sprite: Option<SpriteSpec>,
    pub text: Option<TextSpec>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Transform2d {
    pub x: f32,
    pub y: f32,
    pub rotation: Option<f32>,
    pub scale: Option<f32>,
    pub z: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpriteSpec {
    pub image: String,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextSpec {
    pub value: String,
    pub font: Option<String>,
    pub size: Option<f32>,
}

