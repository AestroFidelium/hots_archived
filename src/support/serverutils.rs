use bincode::{Decode, Encode};
use bincode::{config, de};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::time::Instant;
pub use tracing::{debug, error, info};

use std::cell::RefCell;
use std::io;
use std::sync::Arc;
use tokio::io::AsyncReadExt;

use crate::ecs::{Camera, Keyboard, Mouse};

pub async fn read_from_stream<R, T>(stream: &mut R) -> std::io::Result<T>
where
    R: tokio::io::AsyncRead + Unpin,
    T: de::Decode<()>,
{
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;

    if n == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Server closed connection before responding",
        ));
    }

    let (msg, _): (T, _) = bincode::decode_from_slice(&buf[..n], config::standard())
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    Ok(msg)
}

/// Сообщения, отправляемые клиентом серверу.
#[derive(Serialize, Deserialize, Debug, Encode, Decode, PartialEq, Clone, PartialOrd)]
pub enum Inputs {
    MousePosition(f32, f32),
    RightClick,
    LeftClick,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
}

/// Сообщения, отправляемые клиентом серверу.
#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub enum ClientMessage {
    /// Отправляется для проверки соединения (ping).
    /// Ожидаемый ответ — [`ServerMessage::Pong`].
    Ping,

    /// Запрос на авторизацию пользователя.  
    ///
    /// - Первый параметр — логин [`String`].  
    /// - Второй параметр — пароль [`String`].
    Login(String, String),

    /// Нажатие клавиш
    ///
    /// - Запрос о нажатии клавиш
    Input(Vec<Inputs>),

    /// Ходить в точку
    RightClicked(f32, f32, f32),

    /// Уведомление сервера о разрыве соединения.
    Disconnect,
}

/// Сообщения, отправляемые сервером клиенту.
#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub enum ServerMessage {
    /// Ответ на [`ClientMessage::Ping`].  
    /// Не содержит данных.
    Pong,

    /// Ответ на [`ClientMessage::Ping`].  
    /// [`f32`] время по utc времени с начало эры динозавров
    Delay(f32),

    /// Ответ на [`ClientMessage::Login`].  
    ///
    /// Авторизация прошла успешно.  
    /// [`String`] содержит никнейм игрока.
    SuccessfullyLogin(String),

    /// Ответ на [`ClientMessage::Login`].  
    ///
    /// Ошибка: логин не найден.
    FailedLogin,

    /// Ответ на [`ClientMessage::Login`].
    ///
    /// Ошибка, пароль неверный.
    FailedLoginPassword,

    /// Подтверждение разрыва соединения.  
    /// Сообщает об успешном завершении сеанса.
    Goodbye,

    /// Снимок состояния мира
    ///
    /// ⚠️ Временное решение — в будущем будет заменено на передачу дельты изменений.
    DrawPlayers(Vec<NetPlayer>),

    /// Полный снапшот мира
    ///
    FullSnapshot(WorldSnapshot),
}

#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub struct WorldSnapshot {
    pub players: Vec<PlayerSnapshot>,
}

impl WorldSnapshot {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub struct PlayerSnapshot {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
}

pub struct ClientInfo {
    pub models: Vec<NetModel>,
    pub mouse: Mouse,
    pub keyboard: Keyboard,
    pub camera: Camera,
}

pub struct NetModel {
    pub pos: [f32; 3],
    pub name: String,
    pub rot: [f32; 3],
    pub anim: usize,
    pub anim_dur: f32,
}

impl Default for ClientInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientInfo {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            camera: Camera::new(),
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
        }
    }
}

#[derive(Default)]
pub struct GameServer {
    pub clients: Vec<Player>,
}

impl GameServer {
    pub fn create() -> Arc<RwLock<GameServer>> {
        Arc::new(RwLock::new(GameServer::default()))
    }
}

#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub struct NetPlayer {
    pub hero_name: Option<String>,
    pub position: Option<[f32; 3]>,
    pub rotation: Option<[f32; 3]>,
    pub animation_playing: usize,
    pub animation_timer: f32,
}

pub struct Player {
    pub socket: Arc<RwLock<TcpStream>>,
    pub ip: String,

    nickname: Option<RefCell<String>>,
    pub animation_playing: usize,
    pub animation_timer: f32,
    pub hero: Option<String>, // units : Option<Vec<GltfModel>>,
    pub position: Option<[f32; 3]>,
    pub rotation: Option<[f32; 3]>,
}

impl Player {
    pub fn new(socket: Arc<RwLock<TcpStream>>, ip: String) -> Self {
        Self {
            socket,
            ip,
            nickname: None,
            hero: None,
            position: None,
            rotation: None,
            animation_playing: 0usize,
            animation_timer: 0f32,
        }
    }

    pub fn get_nickname(&self) -> String {
        self.nickname
            .as_ref()
            .map(|rc| rc.borrow().to_string())
            .unwrap_or_else(|| "Anonymous".to_string())
    }

    pub fn set_nickname(&mut self, nickname: String) {
        self.nickname = Some(RefCell::new(nickname));
    }
}

impl From<Player> for NetPlayer {
    fn from(val: Player) -> Self {
        NetPlayer {
            hero_name: val.hero,
            position: None,
            rotation: None,
            animation_playing: val.animation_playing,
            animation_timer: val.animation_timer,
        }
    }
}

impl From<&Player> for NetPlayer {
    fn from(val: &Player) -> Self {
        NetPlayer {
            hero_name: val.hero.clone(),
            position: val.position,
            rotation: val.rotation,
            animation_playing: val.animation_playing,
            animation_timer: val.animation_timer,
        }
    }
}
