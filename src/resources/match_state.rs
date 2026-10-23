use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct MatchState {
    pub paused: bool,
    /// False while the map-creation screen owns the game.
    pub map_ready: bool,
    pub speed: f32,
    pub winner: Option<usize>,
    pub elapsed: f32,
    pub restart_requested: bool,
    pub browser_hidden: bool,
}
impl Default for MatchState {
    fn default() -> Self {
        Self {
            paused: false,
            map_ready: true,
            speed: 1.0,
            winner: None,
            elapsed: 0.0,
            restart_requested: false,
            browser_hidden: false,
        }
    }
}
