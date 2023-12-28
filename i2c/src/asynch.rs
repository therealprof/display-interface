use display_interface::{AsyncWriteOnlyDataCommand, DataFormat, DisplayError};

use crate::I2CInterface;

impl<I2C> AsyncWriteOnlyDataCommand for I2CInterface<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
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
