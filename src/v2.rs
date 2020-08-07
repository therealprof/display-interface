use crate::DisplayError;

#[derive(Clone, Debug)]
pub enum WriteMode {
    Data,
    Command,
}
pub trait WriteInterface<DataFormat> {
    fn write(&mut self, mode: WriteMode, buf: &[DataFormat]) -> Result<(), DisplayError> {
        self.write_iter(mode, &mut buf.into_iter())
    }

    fn write_iter(
        &mut self,
        mode: WriteMode,
        iter: &mut dyn Iterator<Item = &DataFormat>,
    ) -> Result<(), DisplayError>;
}

pub trait ReadInterface<DataFormat> {
    fn read(&mut self, buf: &mut [DataFormat]) -> Result<(), DisplayError> {
        let mut n = 0;
        self.read_stream(&mut |b| {
            if n == buf.len() {
                return false;
            }
            buf[n] = b;
            n += 1;
            true
        })
    }

    fn read_stream(&mut self, f: &mut dyn FnMut(DataFormat) -> bool) -> Result<(), DisplayError>;
}

pub trait ReadWriteInterface<T>: ReadInterface<T> + WriteInterface<T> {}
impl<DataFormat, T> ReadWriteInterface<DataFormat> for T where
    T: ReadInterface<DataFormat> + WriteInterface<DataFormat>
{
}

pub struct ReadIterator<'a, DataFormat> {
    reader: &'a mut dyn ReadInterface<DataFormat>,
}

impl<'a, DataFormat> ReadIterator<'a, DataFormat> {
    fn new(reader: &'a mut dyn ReadInterface<DataFormat>) -> ReadIterator<'a, DataFormat> {
        ReadIterator { reader: reader }
    }
}

impl<'a, DataFormat> Iterator for ReadIterator<'a, DataFormat>
where
    DataFormat: Default,
{
    type Item = Result<DataFormat, DisplayError>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut next: DataFormat = Default::default();
        match self.reader.read_stream(&mut |b| {
            next = b;
            false
        }) {
            Ok(_) => Some(Ok(next)),
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'a, DataFormat> IntoIterator for &'a mut dyn ReadInterface<DataFormat>
where
    DataFormat: Default,
{
    type Item = Result<DataFormat, DisplayError>;
    type IntoIter = ReadIterator<'a, DataFormat>;

    fn into_iter(self) -> Self::IntoIter {
        ReadIterator::new(self)
    }
}
