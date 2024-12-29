use tokio::sync::watch;

use super::value::{Type, Value};

#[derive(Debug)]
pub struct Socket {
    data_type: Type,
    receiver: watch::Receiver<Value>,
    transceiver: watch::Sender<Value>,
}

#[derive(Debug)]
pub struct Sink {
    data_type: Type,
}

#[derive(thiserror::Error, Debug)]
pub enum WireError {
    #[error("Incorrect type provided")]
    IncorrectType,

    #[error(transparent)]
    ChannelClosed(#[from] watch::error::SendError<Value>),
}

impl Socket {
    pub fn new(data_type: Type) -> Self {
        let (transceiver, receiver) = watch::channel(data_type.default());

        Self {
            data_type,
            receiver,
            transceiver,
        }
    }

    pub fn send(&self, value: Value) -> Result<(), WireError> {
        let coerced = value
            .try_coerce(&self.data_type)
            .ok_or_else(|| WireError::IncorrectType)?;

        self.transceiver.send(coerced)?;

        Ok(())
    }
}
