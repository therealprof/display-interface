#[cfg(feature = "async")]
use display_interface::AsyncWriteOnlyDataCommand;
#[cfg(not(feature = "async"))]
use display_interface::WriteOnlyDataCommand;
use display_interface::{DataFormat, DisplayError};
#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c as AsyncI2c;

/// I2C communication interface
pub struct I2cInterface<I2C> {
    i2c: I2C,
    addr: u8,
    data_byte: u8,
}

impl<I2C> I2cInterface<I2C> {
    /// Create new I2C interface for communication with a display driver
    pub fn new(i2c: I2C, addr: u8, data_byte: u8) -> Self {
        Self {
            i2c,
            addr,
            data_byte,
        }
    }

    /// Consume the display interface and return
    /// the underlying peripheral driver
    pub fn release(self) -> I2C {
        self.i2c
    }
}

#[maybe_async_cfg::maybe(
    sync(
        cfg(not(feature = "async")),
        keep_self,
        idents(
            AsyncWriteOnlyDataCommand(sync = "WriteOnlyDataCommand"),
            AsyncI2c(sync = "I2c"),
        )
    ),
    async(feature = "async", keep_self)
)]
impl<I2C> AsyncWriteOnlyDataCommand for I2cInterface<I2C>
where
    I2C: AsyncI2c,
{
    async fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result<(), DisplayError> {
        // Copy over given commands to new aray to prefix with command identifier
        match cmds {
            DataFormat::U8(slice) => {
                let mut writebuf: [u8; 8] = [0; 8];
                writebuf[1..=slice.len()].copy_from_slice(&slice[0..slice.len()]);

                self.i2c
                    .write(self.addr, &writebuf[..=slice.len()])
                    .await
                    .map_err(|_| DisplayError::BusWriteError)
            }
            _ => Err(DisplayError::DataFormatNotImplemented),
        }
    }

    async fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        match buf {
            DataFormat::U8(slice) => {
                // No-op if the data buffer is empty
                if slice.is_empty() {
                    return Ok(());
                }

                let mut writebuf = [0; 17];

                // Data mode
                writebuf[0] = self.data_byte;

                for chunk in slice.chunks(16) {
                    let chunk_len = chunk.len();

                    // Copy over all data from buffer, leaving the data command byte intact
                    writebuf[1..=chunk_len].copy_from_slice(chunk);

                    self.i2c
                        .write(self.addr, &writebuf[0..=chunk_len])
                        .await
                        .map_err(|_| DisplayError::BusWriteError)?;
                }

                Ok(())
            }
            DataFormat::U8Iter(iter) => {
                let mut writebuf = [0; 17];
                let mut i = 1;
                let len = writebuf.len();

                // Data mode
                writebuf[0] = self.data_byte;

                for byte in iter.into_iter() {
                    writebuf[i] = byte;
                    i += 1;

                    if i == len {
                        self.i2c
                            .write(self.addr, &writebuf[0..=len])
                            .await
                            .map_err(|_| DisplayError::BusWriteError)?;
                        i = 1;
                    }
                }

                if i > 1 {
                    self.i2c
                        .write(self.addr, &writebuf[0..=i])
                        .await
                        .map_err(|_| DisplayError::BusWriteError)?;
                }

                Ok(())
            }
            _ => Err(DisplayError::DataFormatNotImplemented),
        }
    }
}
