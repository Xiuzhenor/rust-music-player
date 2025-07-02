use color_eyre::Result;
use rmusic_player::MusicPlayer;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let music_player = MusicPlayer::new();
    let result = music_player.run(terminal);
    ratatui::restore();
    result
}
