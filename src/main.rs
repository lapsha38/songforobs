use anyhow::Result;
use mpris::{Player, PlayerFinder};
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time::{interval, sleep};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();
    
    info!("Starting track monitoring service...");

    let mut current_song: String = String::new();
    let mut check_interval: tokio::time::Interval = interval(Duration::from_secs(2));

    loop {
        check_interval.tick().await;

        match check_tracks(&mut current_song).await {
            Ok(_) => {}
            Err(e) => {
                error!("Error checking tracks: {}", e);
                // Don't exit, just wait for next iteration
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn check_tracks(current_song: &mut String) -> Result<()> {
    let players: Vec<Player> = get_players().await?;

    for player in &players {
        match get_current_track_info(&player).await {
            Ok(current_track_info) => {
                if current_track_info != *current_song && !current_track_info.is_empty() {
                    info!("New track: {}", current_track_info);
                    push_to_file(&current_track_info).await?;
                    *current_song = current_track_info;
                }
            }
            Err(e) => {
                error!(
                    "Error getting info from player {}: {}",
                    player.identity(),
                    e
                );
                // Continue checking other players
            }
        }
    }

    Ok(())
}

async fn get_players() -> Result<Vec<Player>> {
    // find all available MPRIS players
    let finder: PlayerFinder = PlayerFinder::new()?;
    let players_list: Vec<mpris::Player> = finder.find_all()?;
    Ok(players_list)
}

async fn get_current_track_info(player: &Player) -> Result<String> {
    let track: &mpris::Metadata = &player.get_metadata()?;
    let title: String = track.title().unwrap_or("Unknown").to_string();
    let artists: Vec<String> = match track.artists() {
        Some(artist_list) => artist_list.iter().map(|s| s.to_string()).collect(),
        None => vec!["Unknown".to_string()],
    };
    let song_info: String = format!("{} - {}", artists.join(", "), title);
    Ok(song_info)
}

async fn push_to_file(content: &str) -> Result<()> {
    let filepath: &str = "/tmp/current_track.txt";
    let mut file: File = File::create(filepath).await?;
    file.write_all(content.as_bytes()).await?;
    Ok(())
}
