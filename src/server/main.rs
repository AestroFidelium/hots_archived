use std::sync::Arc;
use std::time::Duration;

use heroes_of_the_storm_on_rust::utils::init_config;
use heroes_of_the_storm_on_rust::{
    AsyncReadExt, AsyncWriteExt, ClientMessage, PlayerSnapshot, Position, Rotation, ServerMessage,
    WorldSnapshot, implement_new_tag,
};
use image::EncodableLayout;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};
use tokio::time::sleep;


use bevy_ecs::prelude::*;
use tracing::{debug, error, info};

#[derive(Component)]
pub struct User {
    pub tx: mpsc::UnboundedSender<Vec<u8>>,
}

macro_rules! define_component {
    ($name:ident($inner:ty)) => {
        #[derive(Component)]
        pub struct $name($inner);

        impl std::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

define_component!(Nickname(String));
implement_new_tag!(Snapshotted);

#[derive(Resource)]
struct UserReceiver(mpsc::UnboundedReceiver<TcpStream>);

#[derive(Resource)]
struct ClientMessageReceiver(mpsc::UnboundedReceiver<(Entity, ClientMessage)>);

#[derive(Resource)]
struct ClientMessageSender(mpsc::UnboundedSender<(Entity, ClientMessage)>);

fn add_new_users(
    mut commands: Commands,
    mut receiver: ResMut<UserReceiver>,
    message_sender: Res<ClientMessageSender>,
) {
    while let Ok(socket) = receiver.0.try_recv() {
        let (reader, writer) = socket.into_split();
        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();

        let user_entity = commands.spawn(User { tx }).id();
        info!("Новый пользователь добавлен в мир");
        let client_msg_tx_clone = message_sender.0.clone();

        tokio::spawn(async move {
            let mut reader = reader;
            let mut buffer = [0u8; 1024];

            loop {
                // Пытаемся прочитать сообщение (длину)
                let header = match reader.read_exact(&mut buffer[..4]).await {
                    Ok(_) => u32::from_le_bytes(buffer[..4].try_into().unwrap()),
                    Err(_) => {
                        info!("Клиент отключился или ошибка чтения заголовка");
                        break;
                    }
                };

                // Читаем тело сообщения
                if header > 0 && (header as usize) <= 1024 {
                    match reader.read_exact(&mut buffer[..header as usize]).await {
                        Ok(_) => {
                            info!("Получено от клиента {} байт", header);

                            // 2. Декодируем сообщение клиента
                            if let Ok((msg, _)) = bincode::decode_from_slice::<ClientMessage, _>(
                                &buffer[..header as usize],
                                bincode::config::standard(),
                            ) {
                                // 3. Отправляем сообщение вместе с Entity в канал для ECS
                                if client_msg_tx_clone.send((user_entity, msg)).is_err() {
                                    error!("Не удалось отправить ClientMessage в канал ECS");
                                    break; // Канал закрыт, завершаем задачу
                                }
                            } else {
                                error!("Не удалось декодировать ClientMessage");
                            }
                        }
                        Err(e) => {
                            error!("Ошибка чтения тела сообщения: {}", e);
                            break;
                        }
                    }
                } else {
                    error!("Недопустимый размер сообщения: {}", header);
                    break;
                }
            }
        });

        // Задача для записи
        tokio::spawn(async move {
            let mut writer = writer;

            info!("Задача записи запущена");

            while let Some(data) = rx.recv().await {
                info!("Получено {} байт из канала для отправки", data.len());

                match writer.write_all(&data).await {
                    Ok(_) => {
                        info!("Данные успешно отправлены клиенту");
                    }
                    Err(e) => {
                        error!("Ошибка отправки: {}", e);
                        break;
                    }
                }
            }

            info!("Задача записи завершена");
        });
    }
}

fn handle_client_events(
    mut receiver: ResMut<ClientMessageReceiver>,
    mut players: Query<(&mut Position, &Rotation)>, // Добавьте другие компоненты, которые могут меняться
) {
    // Читаем все сообщения, пришедшие с прошлого запуска
    while let Ok((entity, message)) = receiver.0.try_recv() {
        match message {
            ClientMessage::RightClicked(x, y, z) => {
                info!(
                    "Обработка RightClicked от {:?} на позиции: ({}, {}, {})",
                    entity, x, y, z
                );

                // Пример обработки: если пользователь существует, меняем его позицию (или цель движения)
                if let Ok((mut position, _rotation)) = players.get_mut(entity) {
                    // В реальной игре вы бы установили компонент Destination или Target
                    // Для простоты, обновим Position напрямую:
                    position.set_x(x);
                    position.set_y(y);
                    position.set_z(z);
                    info!(
                        "Позиция пользователя {:?} обновлена до ({}, {}, {})",
                        entity, x, y, z
                    );
                }
            }
            // Добавьте обработку других ClientMessage, если они есть
            _ => {
                debug!("Получено необработанное сообщение клиента: {:?}", message);
            }
        }
    }
}

fn welcome_new_users(mut commands: Commands, query: Query<(Entity, &User), Added<User>>) {
    for (entity, user) in &query {
        commands
            .entity(entity)
            .insert(Nickname("anonymous".to_string()));
        commands.entity(entity).insert(Position::new(0.0, 0.0, 0.0));
        commands.entity(entity).insert(Rotation::new(0.0, 0.0, 0.0));
        info!("Отправил сообщение пользователю \"pong\"");
        user.tx
            .send(bincode::encode_to_vec(ServerMessage::Pong, bincode::config::standard()).unwrap())
            .ok();
    }
}

fn send_world_to_client(
    new_users: Query<&User, Added<User>>,
    all_users: Query<(&User, &Position, &Rotation)>,
) {
    if new_users.is_empty() {
        return;
    }

    let mut snapshot = WorldSnapshot::new();

    for (_, position, rotation) in all_users {
        snapshot.players.push(PlayerSnapshot {
            position: position.into(),
            rotation: rotation.into(),
        });
    }

    let msg = ServerMessage::FullSnapshot(snapshot);
    for user in new_users {
        debug!("Sending full snapshot of the world to new user");
        user.tx
            .send(bincode::encode_to_vec(&msg, bincode::config::standard()).unwrap())
            .ok();
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_config();

    let mut world = World::new();

    let mut schedule = Schedule::default();

    let (tx, rx) = mpsc::unbounded_channel();

    let (user_tx, user_rx) = mpsc::unbounded_channel();
    let (msg_tx, msg_rx) = mpsc::unbounded_channel(); // Новый канал для сообщений клиента

    world.insert_resource(UserReceiver(user_rx));
    world.insert_resource(ClientMessageSender(msg_tx)); // Новый ресурс-отправитель
    world.insert_resource(ClientMessageReceiver(msg_rx));

    schedule.add_systems((
        add_new_users,
        welcome_new_users,
        send_world_to_client.after(welcome_new_users),
        handle_client_events,
        // send_world_to_client.run_if(any_with_component::<User>), // только если есть Added<User>
    ));

    tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
        info!("Слушаем на 127.0.0.1:8080");

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    info!("Новое подключение от: {}", addr);
                    if tx.send(socket).is_err() {
                        error!("Не удалось отправить socket в канал");
                        break;
                    }
                }
                Err(e) => {
                    error!("Ошибка при принятии подключения: {}", e);
                }
            }
        }
    });

    info!("Сервер запускается...");
    loop {
        schedule.run(&mut world);
        sleep(Duration::from_millis(150)).await;
    }
}
