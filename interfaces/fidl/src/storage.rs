// XPARQ OS FIDL Storage Interface
#![no_std]

use super::{FidlInterface, FidlMethod, FidlResult, FidlError, FidlSerializable};
use arrayvec::ArrayVec;

macro_rules! println {
    ($($arg:tt)*) => {};
}

pub trait StorageService: FidlInterface {
    fn create_file(&self, request: &CreateFileRequest) -> FidlResult<CreateFileResponse>;
    fn open_file(&self, request: &OpenFileRequest) -> FidlResult<OpenFileResponse>;
    fn read_file(&self, request: &ReadFileRequest) -> FidlResult<ReadFileResponse>;
    fn write_file(&self, request: &WriteFileRequest) -> FidlResult<WriteFileResponse>;
    fn delete_file(&self, request: &DeleteFileRequest) -> FidlResult<DeleteFileResponse>;
    fn list_files(&self, request: &ListFilesRequest) -> FidlResult<ListFilesResponse>;
}

#[derive(Debug)]
pub struct StorageManager {
    service_handle: u32,
}

impl StorageManager {
    pub fn new(service_handle: u32) -> Self {
        Self { service_handle }
    }
}

impl FidlInterface for StorageManager {
    fn interface_name() -> &'static str { "xparq.storage.manager" }
    fn interface_version() -> u32 { 1 }
    fn method_count() -> u32 { 6 }
}

impl StorageService for StorageManager {
    fn create_file(&self, _request: &CreateFileRequest) -> FidlResult<CreateFileResponse> {
        Ok(CreateFileResponse { success: true, file_handle: None, error_code: None })
    }
    fn open_file(&self, _request: &OpenFileRequest) -> FidlResult<OpenFileResponse> {
        Ok(OpenFileResponse { success: true, file_handle: None, error_code: None })
    }
    fn read_file(&self, _request: &ReadFileRequest) -> FidlResult<ReadFileResponse> {
        Ok(ReadFileResponse { success: true, data: ArrayVec::new(), error_code: None })
    }
    fn write_file(&self, _request: &WriteFileRequest) -> FidlResult<WriteFileResponse> {
        Ok(WriteFileResponse { success: true, bytes_written: 0, error_code: None })
    }
    fn delete_file(&self, _request: &DeleteFileRequest) -> FidlResult<DeleteFileResponse> {
        Ok(DeleteFileResponse { success: true, error_code: None })
    }
    fn list_files(&self, _request: &ListFilesRequest) -> FidlResult<ListFilesResponse> {
        Ok(ListFilesResponse { files: ArrayVec::new(), error_code: None })
    }
}

#[derive(Debug, Clone)]
pub struct CreateFileRequest {
    pub path: &'static str,
    pub mode: FileMode,
}

#[derive(Debug, Clone)]
pub struct CreateFileResponse {
    pub success: bool,
    pub file_handle: Option<FileHandle>,
    pub error_code: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct OpenFileRequest {
    pub path: &'static str,
    pub mode: FileMode,
}

#[derive(Debug, Clone)]
pub struct OpenFileResponse {
    pub success: bool,
    pub file_handle: Option<FileHandle>,
    pub error_code: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ReadFileRequest {
    pub file_handle: FileHandle,
    pub offset: u64,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct ReadFileResponse {
    pub success: bool,
    pub data: ArrayVec<u8, 4096>,
    pub error_code: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct WriteFileRequest {
    pub file_handle: FileHandle,
    pub offset: u64,
    pub data: &'static [u8],
}

#[derive(Debug, Clone)]
pub struct WriteFileResponse {
    pub success: bool,
    pub bytes_written: u64,
    pub error_code: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct DeleteFileRequest {
    pub path: &'static str,
}

#[derive(Debug, Clone)]
pub struct DeleteFileResponse {
    pub success: bool,
    pub error_code: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ListFilesRequest {
    pub directory: &'static str,
    pub pattern: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct ListFilesResponse {
    pub files: ArrayVec<FileInfo, 64>,
    pub error_code: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct FileHandle {
    pub file_id: u64,
    pub mode: FileMode,
}

#[derive(Debug, Clone, Copy)]
pub struct DirectoryHandle {
    pub directory_id: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct FileInfo {
    pub path: &'static str,
    pub size: u64,
    pub mode: FileMode,
    pub created_at: u64,
    pub modified_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileMode {
    Read = 0,
    Write = 1,
    ReadWrite = 2,
    Append = 3,
}
