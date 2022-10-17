pub struct PlayerRanges(pub Vec<PlayerRange>);

impl PlayerRanges {
    pub fn create() -> PlayerRanges {
        PlayerRanges { 0: vec![] }
    }

    pub fn create_player_range(&mut self, addr: cosmwasm_std::Addr, start: u64, end: u64) -> () {
        let player_range = PlayerRange {
            player_addr: addr,
            start_range: start,
            end_range: end,
        };
        self.0.push(player_range)
    }
}

pub struct PlayerRange {
    pub player_addr: cosmwasm_std::Addr,
    pub start_range: u64,
    pub end_range: u64,
}
