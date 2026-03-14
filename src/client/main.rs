#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(trivial_bounds)]
// #[macro_use]
extern crate glium;

use tokio::time::sleep;

use heroes_of_the_storm_on_rust::{prelude::*, utils::init_config};

async fn connect_to_server() -> std::io::Result<()> {
    let stream = {
        let mut delay = 1;

        loop {
            match TcpStream::connect("127.0.0.1:8080").await {
                Ok(s) => {
                    info!("Successful connected to the server");
                    break s;
                }
                Err(_e) => {
                    error!("Failed to connect to server",);
                    info!("Retrying in {} seconds...", delay);

                    tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                    delay = (delay * 2).min(30); // увеличиваем до максимум 30 секунд
                }
            }
        }
    };

    let (mut reader, mut writer) = stream.into_split();

    if let Ok(msg) = read_from_stream::<_, ServerMessage>(&mut reader).await {
        if let ServerMessage::Pong = msg {
            info!("Got a pong message from the server")
        }
    }

    if let Ok(msg) = read_from_stream::<_, ServerMessage>(&mut reader).await {
        if let ServerMessage::FullSnapshot(snapshot) = msg
            && let Ok(mut game) = GAME.try_write()
        {
            for (index, player) in snapshot.players.iter().enumerate() {
                info!("yes{}", index);

                game.models.push(NetModel {
                    pos: player.position,
                    name: "TracerHero".to_string(),
                    rot: player.rotation,
                    anim: 1,
                    anim_dur: 3f32,
                });
            }
        }
    }

    tokio::spawn(async move {
        loop {
            let mut output: Option<Vec<u8>> = None; // Use an Option to hold the encoded message

            // Scope to ensure 'game' (the ReadGuard) is dropped before .await
            {
                if let Ok(game) = GAME.try_read() {
                    if game.mouse.is_button_hold(MouseButton::Right) {
                        if let Some(position) = game
                            .camera
                            .cursor_to_world(game.mouse.position[0], game.mouse.position[1])
                        {

                            output = Some(bincode::encode_to_vec(
                                ClientMessage::RightClicked(position.x, position.y, position.z),
                                bincode::config::standard(),
                            )
                            .unwrap());
                        }
                    }
                }
            } // 'game' (RwLockReadGuard) drops here!

            // Now that the lock is released, we can use .await
            if let Some(output_bytes) = output {
                // Error handling for write_all is good practice, added for completeness
                if let Err(e) = writer.write_all(&output_bytes[..]).await {
                    error!("Failed to write to stream: {}", e);
                    // You might want to break the loop or handle the error more gracefully
                }
            }

            sleep(tokio::time::Duration::from_millis(1)).await;
        }
    });

    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_config();

    tokio::spawn(connect_to_server());

    State::<Application>::run_loop();
    Ok(())
}
