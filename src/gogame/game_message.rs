use crate::core::entities::Stone;
use crate::gogame::board::Board;
use iced_native::Event;

#[derive(Clone, Debug)]
pub enum GameMessage {
    BoardLoaded(Board),
    EventOccurred(Event),
    AfterStonePlayed(Vec<Stone>, Vec<Stone>),
    AfterGenMove(Vec<Stone>, Vec<Stone>),
    GtpError(String),
}
