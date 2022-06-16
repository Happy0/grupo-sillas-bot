
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct LolApiError {
    pub description: String,
    pub http_code: String
}

#[derive(Serialize, Deserialize)]
pub struct GameSummary {
    pub metadata: Metadata,
    pub info: GameInfo
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub matchId: String
}

#[derive(Serialize, Deserialize)]
pub struct GameInfo {
    pub participants: Vec<Participant>,
    pub gameDuration: u64,
    pub gameId: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Participant {
    pub championName: String,
    pub puuid: String,
    pub win: bool,
    pub kills: u64,
    pub deaths: u64,
    pub assists: u64
}

#[derive(Debug)]
pub struct UserGameSummary {
    pub game_id: u64,
    pub match_id: String,
    pub participant: Participant,
    pub game_duration_millis: u64
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LeagueEntry {
    pub tier: String,
    pub rank: String,
    pub leaguePoints: u64,
    pub queueType: String,
    pub leagueId: String
}

impl std::convert::From<reqwest::Error> for LolApiError {
    fn from(error: reqwest::Error) -> Self {

        let result = LolApiError {
            http_code: "500".to_string(),
            description: error.to_string()
        };

        return result;
    }
}

