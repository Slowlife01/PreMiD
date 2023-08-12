use super::{Connection, SocketConnection};
use crate::{
    error::{DiscordError, Result},
    event_handler::HandlerRegistry,
    models::{payload::Payload, Event, Message},
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use parking_lot::Mutex;
use serde_json::Value as JsonValue;
use std::{
    io::ErrorKind,
    sync::{atomic::Ordering, Arc},
    thread, time,
};

type Tx = Sender<Message>;
type Rx = Receiver<Message>;

// TODO: Refactor connection manager
#[derive(Clone)]
pub struct Manager {
    connection: Arc<Option<Mutex<SocketConnection>>>,
    client_id: u64,
    outbound: (Rx, Tx),
    inbound: (Rx, Tx),
    handshake_completed: bool,
    stop: bool,
    event_handler_registry: HandlerRegistry<'static>,
}

impl Manager {
    pub fn new(client_id: u64, event_handler_registry: HandlerRegistry<'static>) -> Self {
        let connection = Arc::new(None);
        let (sender_o, receiver_o) = unbounded();
        let (sender_i, receiver_i) = unbounded();

        Self {
            connection,
            client_id,
            stop: false,
            handshake_completed: false,
            inbound: (receiver_i, sender_i),
            outbound: (receiver_o, sender_o),
            event_handler_registry,
        }
    }

    pub fn start(&mut self) -> std::thread::JoinHandle<()> {
        let manager_inner = self.clone();
        thread::spawn(move || {
            send_and_receive_loop(manager_inner);
        })
    }

    pub fn stop(&mut self) {
        self.stop = true;
    }

    pub fn send(&self, message: Message) -> Result<()> {
        self.outbound.1.send(message)?;

        Ok(())
    }

    pub fn recv(&self) -> Result<Message> {
        self.inbound.0.recv().map_err(DiscordError::from)
    }

    pub fn get_client_id(&self) -> u64 {
        self.client_id
    }

    fn connect(&mut self) -> Result<()> {
        if self.connection.is_some() {
            return Ok(());
        }

        trace!("Connecting");

        let mut new_connection = SocketConnection::connect()?;

        trace!("Performing handshake");
        let msg = new_connection.handshake(self.client_id)?;
        let payload: Payload<JsonValue> = serde_json::from_str(&msg.payload)?;
        self.event_handler_registry
            .handle(Event::Ready, into_error!(payload.data)?)?;
        trace!("Handshake completed");

        self.connection = Arc::new(Some(Mutex::new(new_connection)));

        trace!("Connected");

        Ok(())
    }

    fn disconnect(&mut self) {
        self.handshake_completed = false;
        self.connection = Arc::new(None);
    }
}

fn send_and_receive_loop(mut manager: Manager) {
    trace!("Starting sender loop");

    let mut inbound = manager.inbound.1.clone();
    let outbound = manager.outbound.0.clone();

    loop {
        let connection = manager.connection.clone();
        if manager.stop {
            break;
        }

        match *connection {
            Some(ref conn) => {
                let mut connection = conn.lock();
                match send_and_receive(
                    &mut connection,
                    &mut manager.event_handler_registry,
                    &mut inbound,
                    &outbound,
                ) {
                    Err(DiscordError::IoError(ref err)) if err.kind() == ErrorKind::WouldBlock => {}
                    Err(DiscordError::IoError(_)) | Err(DiscordError::ConnectionClosed) => {
                        manager.disconnect()
                    }
                    Err(DiscordError::RecvTimeoutError(_)) => continue,
                    Err(why) => trace!("discord error: {}", why),
                    _ => {}
                }

                thread::sleep(time::Duration::from_millis(500));
            }
            None => match manager.connect() {
                Err(err) => {
                    if !err.io_would_block() {
                        error!("Failed to connect: {:?}", err)
                    }

                    crate::STARTED.store(false, Ordering::Relaxed);

                    break;
                }
                _ => manager.handshake_completed = true,
            },
        }
    }
}

fn send_and_receive(
    connection: &mut SocketConnection,
    _event_handler_registry: &mut HandlerRegistry<'_>,
    _inbound: &mut Tx,
    outbound: &Rx,
) -> Result<()> {
    while let Ok(msg) = outbound.try_recv() {
        trace!("Sending message");
        connection.send(&msg)?;
        trace!("Sent message");
    }

    Ok(())
}
