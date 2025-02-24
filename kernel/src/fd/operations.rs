/// File operations interface to be implemented by resources
pub trait FileOperations: Send + Sync + 'static {
    /// Read data from the file to a buffer
    fn read(&self, buf: &mut [u8]) -> Result<usize, FileOperationError>;
    /// Write data from a buffer to the file
    fn write(&self, buf: &[u8]) -> Result<usize, FileOperationError>;
    /// Seek to a specific offset in the file
    fn seek(&self, offset: usize) -> Result<usize, FileOperationError>;
    /// Close the file
    fn close(&self) -> Result<(), FileOperationError>;
}

#[derive(Debug)]
pub enum FileOperationError {
    ReadError,
    WriteError,
    SeekError,
    CloseError,
}
