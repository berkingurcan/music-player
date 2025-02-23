use std::env;

use clap::ArgMatches;
use music_player_client::{
    library::LibraryClient, playback::PlaybackClient, playlist::PlaylistClient,
    tracklist::TracklistClient,
};
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::{Player, PlayerEngine},
};
use owo_colors::OwoColorize;
use tabled::{builder::Builder, Style};

use crate::scan::scan_music_library;

pub async fn parse_args(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(matches) = matches.subcommand_matches("open") {
        let audio_format = AudioFormat::default();
        let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();

        let (mut player, _) = Player::new(move || backend(None, audio_format), |_| {});

        let song = matches.value_of("song").unwrap();

        player.load(song, true, 0);

        player.await_end_of_track().await;
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("scan") {
        scan_music_library(true).await.map_err(|e| e.to_string())?;
        return Ok(());
    }

    if let Some(matches) = matches.subcommand_matches("albums") {
        let mut client = LibraryClient::new().await?;

        if matches.is_present("id") {
            let id = matches.value_of("id").unwrap();
            let album = client.album(id).await?;
            if album.is_none() {
                return Err("Album not found".into());
            }
            let album = album.unwrap();

            let mut builder = Builder::default();
            builder.set_columns(["id", "title", "album"]);
            album.tracks.iter().for_each(|track| {
                let title = format!(
                    "{} {}",
                    format_number(usize::try_from(track.track_number).unwrap()),
                    track.title
                );
                let album_title = album.title.to_string();
                builder.add_record([track.id.as_str(), title.as_str(), album_title.as_str()]);
            });
            let table = builder.build().with(Style::psql());
            println!("\n{}", table);

            return Ok(());
        }

        let result = client.albums().await?;

        let mut builder = Builder::default();
        builder.set_columns(["id", "name"]);
        result.iter().for_each(|album| {
            builder.add_record([
                album.id.as_str(),
                album.title.magenta().to_string().as_str(),
            ]);
        });
        let table = builder.build().with(Style::psql());
        println!("\n{}", table);

        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("artists") {
        let mut client = LibraryClient::new().await?;
        let result = client.artists().await?;

        let mut builder = Builder::default();
        builder.set_columns(["id", "name"]);
        result.iter().for_each(|artist| {
            builder.add_record([
                artist.id.as_str(),
                artist.name.magenta().to_string().as_str(),
            ]);
        });
        let table = builder.build().with(Style::psql());
        println!("\n{}", table);

        return Ok(());
    }

    if let Some(matches) = matches.subcommand_matches("playlist") {
        let mut client = PlaylistClient::new().await?;

        if let Some(matches) = matches.subcommand_matches("add") {
            let id = matches.value_of("id").unwrap();

            return Ok(());
        }

        if let Some(_matches) = matches.subcommand_matches("ls") {
            return Ok(());
        }

        if let Some(matches) = matches.subcommand_matches("clear") {
            let id = matches.value_of("id");

            return Ok(());
        }

        if let Some(matches) = matches.subcommand_matches("play") {
            let id = matches.value_of("id");

            return Ok(());
        }

        if let Some(matches) = matches.subcommand_matches("remove") {
            let id = matches.value_of("id").unwrap();

            return Ok(());
        }

        if let Some(_matches) = matches.subcommand_matches("shuffle") {
            return Ok(());
        }

        if let Some(_matches) = matches.subcommand_matches("all") {
            return Ok(());
        }
    }

    if let Some(matches) = matches.subcommand_matches("queue") {
        let mut client = TracklistClient::new().await?;

        if let Some(_) = matches.subcommand_matches("list") {
            let (mut previous_tracks, next_tracks) = client.list().await?;
            let last_track = previous_tracks.pop().unwrap();
            for (i, track) in previous_tracks.iter().enumerate() {
                println!("{} {}", format_number(i + 1), track.title);
            }
            println!(
                "{} {}",
                format_number(previous_tracks.len() + 1).magenta(),
                last_track.title.magenta()
            );
            for (i, track) in next_tracks.iter().enumerate() {
                println!(
                    "{} {}",
                    format_number(i + previous_tracks.len() + 2),
                    track.title
                );
            }
            return Ok(());
        }

        if let Some(matches) = matches.subcommand_matches("add") {
            let id = matches.value_of("track_id").unwrap();
            client.add(id).await?;
            return Ok(());
        }

        if let Some(matches) = matches.subcommand_matches("remove") {
            let song = matches.value_of("song").unwrap();

            return Ok(());
        }

        if let Some(matches) = matches.subcommand_matches("clear") {
            let all = matches.is_present("all");

            return Ok(());
        }
    }

    if let Some(_matches) = matches.subcommand_matches("tracks") {
        let mut client = LibraryClient::new().await?;
        let result = client.songs().await?;

        let mut builder = Builder::default();
        builder.set_columns(["id", "title"]);
        result.iter().for_each(|song| {
            builder.add_record([song.id.as_str(), song.title.magenta().to_string().as_str()]);
        });
        let table = builder.build().with(Style::psql());
        println!("\n{}", table);

        return Ok(());
    }

    if let Some(matches) = matches.subcommand_matches("search") {
        let client = LibraryClient::new().await?;

        let query = matches.value_of("query").unwrap();
        todo!("search for {}", query);
    }

    if let Some(_) = matches.subcommand_matches("pause") {
        let mut client = PlaybackClient::new().await?;
        client.pause().await?;
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("play") {
        let mut client = PlaybackClient::new().await?;
        client.play().await?;
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("next") {
        let mut client = PlaybackClient::new().await?;
        client.next().await?;
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("prev") {
        let mut client = PlaybackClient::new().await?;
        client.prev().await?;
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("stop") {
        let mut client = PlaybackClient::new().await?;
        client.stop().await?;
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("current") {
        let mut client = PlaybackClient::new().await?;
        let (result, _, _, _) = client.current().await?;
        if result.is_none() {
            println!("No song is currently playing");
            return Ok(());
        }

        let result = result.unwrap();
        let artists = result.artists;
        let title = result.title;
        println!("");
        println!("Title  : {}", title.magenta());
        println!(
            "Artist : {}",
            artists
                .iter()
                .map(|a| a.name.clone())
                .collect::<Vec<String>>()
                .join(", ")
                .magenta()
        );
        let album = result.album;
        if album.is_some() {
            println!("Album  : {}", album.unwrap().title.magenta());
        }
        return Ok(());
    }

    if let Some(matches) = matches.subcommand_matches("connect") {
        let host = matches.value_of("host").unwrap();
        let port = matches.value_of("port").unwrap();
        env::set_var("MUSIC_PLAYER_HOST", host);
        env::set_var("MUSIC_PLAYER_PORT", port);
        env::set_var("MUSIC_PLAYER_MODE", "client");
    }

    return Err("No subcommand found".into());
}

fn format_number(number: usize) -> String {
    if number < 10 {
        return format!("0{}", number);
    }
    format!("{}", number)
}
