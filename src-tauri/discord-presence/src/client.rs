use std::sync::atomic::Ordering;

use crate::{
    connection::Manager as ConnectionManager,
    event_handler::{Context as EventContext, HandlerRegistry},
    models::{
        message::Message,
        payload::Payload,
        rich_presence::{Activity, SetActivityArgs},
        Command, Event, OpCode,
    },
    DiscordError, Result,
};
use serde::Serialize;

macro_rules! event_handler_function {
    ( $( $name:ident, $event:expr ),* ) => {
        event_handler_function!{@gen $([ $name, $event])*}
    };

    (@gen $( [ $name:ident, $event:expr ] ), *) => {
        $(
            #[doc = concat!("Listens for the `", stringify!($event), "` event")]
            pub fn $name<F>(&mut self, handler: F)
                where F: Fn(EventContext) + 'static + Send + Sync
            {
                self.on_event($event, handler);
            }
        )*
    }
}

/// The Discord client
#[derive(Clone)]
pub struct Client {
    connection_manager: ConnectionManager,
    event_handler_registry: HandlerRegistry<'static>,
}

impl Drop for Client {
    fn drop(&mut self) {
        self.connection_manager.stop();
    }
}

#[cfg(feature = "bevy")]
impl bevy::ecs::system::Resource for Client {}

impl Client {
    /// Creates a new `Client`
    pub fn new(client_id: u64) -> Self {
        let event_handler_registry = HandlerRegistry::new();
        let connection_manager = ConnectionManager::new(client_id, event_handler_registry.clone());
        Self {
            connection_manager,
            event_handler_registry,
        }
    }

    /// Start the connection manager
    ///
    /// Only join the thread if there is no other task keeping the program alive.
    ///
    /// This must be called before all and any actions such as `set_activity`
    #[must_use]
    pub fn start(&mut self) -> std::thread::JoinHandle<()> {
        let thread = self.connection_manager.start();

        crate::STARTED.store(true, Ordering::Relaxed);

        self.on_ready(|_| {
            trace!("Discord client is ready!");
            crate::READY.store(true, Ordering::Relaxed);
        });

        thread
    }

    pub fn client_id(&self) -> u64 {
        self.connection_manager.get_client_id()
    }

    /// Check if the client is ready
    pub fn is_ready() -> bool {
        crate::READY.load(Ordering::Acquire)
    }

    /// Check if the client has started
    pub fn is_started() -> bool {
        crate::STARTED.load(Ordering::Acquire)
    }

    fn execute<A>(&mut self, cmd: Command, args: A, evt: Option<Event>) -> Result<()>
    where
        A: Serialize + Send + Sync,
    {
        if !crate::STARTED.load(Ordering::Relaxed) || !crate::READY.load(Ordering::Relaxed) {
            return Err(DiscordError::NotStarted);
        }

        trace!("Executing command: {:?}", cmd);
        let message = Message::new(
            OpCode::Frame,
            Payload::with_nonce(cmd, Some(args), None, evt),
        );

        self.connection_manager.send(message?)?;

        Ok(())
    }

    /// Set the users current activity
    pub fn set_activity<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(Activity) -> Activity,
    {
        self.execute(Command::SetActivity, SetActivityArgs::new(f), None)
    }

    /// Clear the users current activity
    pub fn clear_activity(&mut self) -> Result<()> {
        self.execute(Command::SetActivity, SetActivityArgs::default(), None)
    }

    /// Register a handler for a given event
    pub fn on_event<F>(&mut self, event: Event, handler: F)
    where
        F: Fn(EventContext) + 'static + Send + Sync,
    {
        self.event_handler_registry.register(event, handler);
    }

    /// Block the current thread until the event is fired
    ///
    /// Returns the context the event was fired in
    ///
    /// NOTE: Please only use this for the ready event, or if you know what you are doing.
    ///
    /// # Panics
    ///
    /// Panics if the channel is disconnected for whatever reason.
    pub fn block_until_event(&mut self, event: Event) -> Result<crate::event_handler::Context> {
        let (tx, rx) = crossbeam_channel::bounded::<crate::event_handler::Context>(1);

        let handler = move |info| tx.send(info).unwrap();

        self.event_handler_registry.register(event, handler);

        Ok(rx.recv()?)
    }

    event_handler_function!(on_ready, Event::Ready);

    event_handler_function!(on_error, Event::Error);

    event_handler_function!(on_activity_join, Event::ActivityJoin);

    event_handler_function!(on_activity_join_request, Event::ActivityJoinRequest);

    event_handler_function!(on_activity_spectate, Event::ActivitySpectate);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_started() {
        assert!(!Client::is_started());

        crate::STARTED.store(true, Ordering::Relaxed);

        assert!(Client::is_started());
    }

    #[test]
    fn test_is_ready() {
        assert!(!Client::is_ready());

        crate::READY.store(true, Ordering::Relaxed);

        assert!(Client::is_ready());
    }
}
