//! Real-time ETW bridge. The callback copies only EVENT_RECORD metadata and
//! user bytes, then hands the record to the same decoder used by the spool
//! path. SQLite, models, and filesystem writes stay outside the callback.

use super::{FileObjectTable, PathInterner, SequenceAllocator};
use clens_types::{EventPayload, SysEvent, SysKind};
use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use windows_sys::Win32::System::Diagnostics::Etw::{
    CloseTrace, OpenTraceW, ProcessTrace, TdhGetEventInformation, TdhGetProperty,
    TdhGetPropertySize, EVENT_PROPERTY_INFO, EVENT_RECORD, EVENT_TRACE_LOGFILEW,
    EVENT_TRACE_LOGFILEW_0, EVENT_TRACE_LOGFILEW_1, PROCESSTRACE_HANDLE,
    PROCESS_TRACE_MODE_EVENT_RECORD, PROCESS_TRACE_MODE_REAL_TIME, PROPERTY_DATA_DESCRIPTOR,
    TRACE_EVENT_INFO,
};

#[derive(Clone)]
pub struct RawEtwRecord {
    pub provider: String,
    pub provider_guid: windows_sys::core::GUID,
    pub event_id: u16,
    pub version: u8,
    pub opcode: u8,
    pub process_id: u32,
    pub thread_id: u32,
    pub timestamp_qpc: u64,
    pub user_data: Vec<u8>,
    pub tdh_properties: Vec<String>,
    pub path: Option<String>,
    pub event: SysEvent,
}

#[derive(Debug)]
struct CallbackContext {
    provider: String,
    sender: mpsc::SyncSender<RawEtwRecord>,
    decoder: ManualEventDecoder,
    dropped: Arc<AtomicU64>,
}

#[derive(Debug, Default)]
pub struct ManualEventDecoder {
    paths: PathInterner,
    files: FileObjectTable,
    sequences: SequenceAllocator,
}

impl ManualEventDecoder {
    pub fn decode(
        &mut self,
        provider: &str,
        record: &EVENT_RECORD,
        user_data: &[u8],
        properties: &[String],
    ) -> SysEvent {
        let kind = classify(
            provider,
            record.EventHeader.EventDescriptor.Opcode,
            properties,
        );
        let path = property_text(record, properties, &["FileName", "FilePath", "ImageName"])
            .or_else(|| extract_path(user_data));
        let path_id = path.as_ref().map_or(0, |value| self.paths.intern(value));
        let file_object = property_number(
            record,
            properties,
            &["FileObject", "FileObjectPointer", "Object"],
        );
        if let (Some(object), Some(value)) = (file_object, path.as_ref()) {
            self.files.insert(object, path_id, value.clone());
        }
        let desired_access = property_number(
            record,
            properties,
            &["DesiredAccess", "AccessMask", "CreateOptions"],
        )
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or_default();
        let io_len = property_number(record, properties, &["IoSize", "TransferSize", "Size"])
            .and_then(|value| u32::try_from(value).ok());
        let payload = match kind {
            SysKind::ProcStart => EventPayload::Proc {
                ppid: property_number(record, properties, &["ParentId", "ParentProcessId"])
                    .and_then(|value| u32::try_from(value).ok())
                    .unwrap_or_default(),
                image: path.clone().unwrap_or_default().into(),
                cmd: property_text(record, properties, &["CommandLine", "CommandLineString"])
                    .unwrap_or_default()
                    .into(),
                exit_code: None,
            },
            SysKind::FileCreate
            | SysKind::FileRead
            | SysKind::FileWrite
            | SysKind::FileClose
            | SysKind::FileRename
            | SysKind::FileSetInfo
            | SysKind::FileDelete => EventPayload::File {
                path: if path_id != 0 {
                    path_id
                } else {
                    file_object
                        .and_then(|object| self.files.path_id(object))
                        .unwrap_or_default()
                },
                desired_access,
                io_len,
                write_attrs: kind == SysKind::FileSetInfo,
            },
            _ => EventPayload::None,
        };
        self.sequences.assign(SysEvent {
            schema_v: 1,
            seq: 0,
            ts_qpc: record.EventHeader.TimeStamp as u64,
            pid: record.EventHeader.ProcessId,
            tid: record.EventHeader.ThreadId,
            kind,
            is_rundown: false,
            payload,
        })
    }

    pub fn path_table(&self) -> Vec<Box<str>> {
        self.paths.snapshot()
    }

    pub fn path_for_event(&self, event: &SysEvent) -> Option<String> {
        match &event.payload {
            EventPayload::File { path, .. } => self.paths.resolve(*path).map(str::to_owned),
            EventPayload::Proc { image, .. } if !image.is_empty() => Some(image.to_string()),
            _ => None,
        }
    }

    pub fn file_object_table(&self) -> &FileObjectTable {
        &self.files
    }
}

pub struct RealtimeConsumer {
    handle: std::thread::JoinHandle<Result<(), String>>,
    trace_handle: PROCESSTRACE_HANDLE,
    dropped: Arc<AtomicU64>,
}

impl std::fmt::Debug for RealtimeConsumer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RealtimeConsumer")
            .field("trace_handle", &self.trace_handle.Value)
            .finish_non_exhaustive()
    }
}

impl RealtimeConsumer {
    pub fn spawn(
        logger_name: &str,
        provider: &str,
        sender: mpsc::SyncSender<RawEtwRecord>,
    ) -> Result<Self, String> {
        let name: Vec<u16> = logger_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let provider_name = provider.to_owned();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let dropped = Arc::new(AtomicU64::new(0));
        let dropped_for_thread = Arc::clone(&dropped);
        let handle = std::thread::Builder::new()
            .name(format!("etw-{}", provider.to_ascii_lowercase()))
            .spawn(move || run_trace(name, provider_name, sender, dropped_for_thread, ready_tx))
            .map_err(|error| error.to_string())?;
        let (trace_handle, _) = ready_rx.recv().map_err(|error| error.to_string())??;
        Ok(Self {
            handle,
            trace_handle,
            dropped,
        })
    }

    pub fn dropped_events(&self) -> u64 {
        self.dropped.load(Ordering::Acquire)
    }

    pub fn stop(self) -> Result<(), String> {
        let close_result = unsafe { CloseTrace(self.trace_handle) };
        let join_result = self
            .handle
            .join()
            .map_err(|_| "ETW consumer thread panicked".to_owned())?;
        if close_result != 0 {
            return Err(format!("CloseTrace failed with Win32 error {close_result}"));
        }
        join_result
    }
}

unsafe extern "system" fn record_callback(record: *mut EVENT_RECORD) {
    if record.is_null() {
        return;
    }
    let record_ref = unsafe { &*record };
    if record_ref.UserDataLength > 0 && record_ref.UserData.is_null() {
        return;
    }
    let user_data = if record_ref.UserDataLength == 0 {
        Vec::new()
    } else {
        unsafe {
            std::slice::from_raw_parts(
                record_ref.UserData.cast::<u8>(),
                usize::from(record_ref.UserDataLength),
            )
            .to_vec()
        }
    };
    let context = record_ref.UserContext.cast::<CallbackContext>();
    if context.is_null() {
        return;
    }
    let context = unsafe { &mut *context };
    let tdh_properties = tdh_property_names(record_ref).unwrap_or_default();
    let event = context
        .decoder
        .decode(&context.provider, record_ref, &user_data, &tdh_properties);
    let path = context.decoder.path_for_event(&event);
    let raw = RawEtwRecord {
        provider: context.provider.clone(),
        provider_guid: record_ref.EventHeader.ProviderId,
        event_id: record_ref.EventHeader.EventDescriptor.Id,
        version: record_ref.EventHeader.EventDescriptor.Version,
        opcode: record_ref.EventHeader.EventDescriptor.Opcode,
        process_id: record_ref.EventHeader.ProcessId,
        thread_id: record_ref.EventHeader.ThreadId,
        timestamp_qpc: record_ref.EventHeader.TimeStamp as u64,
        user_data,
        tdh_properties,
        path,
        event,
    };
    if context.sender.try_send(raw).is_err() {
        context.dropped.fetch_add(1, Ordering::Relaxed);
    }
}

/// TDH oracle used by the slow path and golden tests. It asks Windows for the
/// manifest schema of the exact provider/event/version and returns the field
/// names without guessing from an opcode. A failed lookup is surfaced to the
/// caller as an empty property list and the raw event remains available.
pub fn tdh_property_names(record: &EVENT_RECORD) -> Result<Vec<String>, u32> {
    let mut required = 0_u32;
    let first = unsafe {
        TdhGetEventInformation(
            record,
            0,
            std::ptr::null(),
            std::ptr::null_mut(),
            &mut required,
        )
    };
    if required == 0 || (first != 122 && first != 0) {
        return Err(first);
    }
    let words = usize::try_from(required)
        .map_err(|_| u32::MAX)?
        .saturating_add(7)
        / 8;
    let mut storage = vec![0_u64; words];
    let info = storage.as_mut_ptr().cast::<TRACE_EVENT_INFO>();
    let result =
        unsafe { TdhGetEventInformation(record, 0, std::ptr::null(), info, &mut required) };
    if result != 0 {
        return Err(result);
    }
    let info_ref = unsafe { &*info };
    let count = usize::try_from(info_ref.PropertyCount).map_err(|_| u32::MAX)?;
    let property_ptr =
        std::ptr::addr_of!(info_ref.EventPropertyInfoArray).cast::<EVENT_PROPERTY_INFO>();
    let properties = unsafe { std::slice::from_raw_parts(property_ptr, count) };
    let mut names = Vec::new();
    for property in properties {
        let offset = property.NameOffset as usize;
        if offset >= usize::try_from(required).unwrap_or(0) {
            continue;
        }
        let bytes = unsafe {
            std::slice::from_raw_parts(
                storage.as_ptr().cast::<u8>().add(offset),
                usize::try_from(required)
                    .unwrap_or(0)
                    .saturating_sub(offset),
            )
        };
        let units = bytes
            .chunks(2)
            .take_while(|pair| pair.len() == 2 && (pair[0] != 0 || pair[1] != 0))
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        if let Ok(name) = String::from_utf16(&units) {
            names.push(name);
        }
    }
    Ok(names)
}

fn run_trace(
    name: Vec<u16>,
    provider: String,
    sender: mpsc::SyncSender<RawEtwRecord>,
    dropped: Arc<AtomicU64>,
    ready: mpsc::SyncSender<Result<(PROCESSTRACE_HANDLE, Arc<AtomicU64>), String>>,
) -> Result<(), String> {
    let mut context = Box::new(CallbackContext {
        provider,
        sender,
        decoder: ManualEventDecoder::default(),
        dropped: Arc::clone(&dropped),
    });
    let mut logfile = EVENT_TRACE_LOGFILEW {
        LoggerName: name.as_ptr().cast_mut(),
        Context: (&mut *context as *mut CallbackContext).cast::<c_void>(),
        ..Default::default()
    };
    logfile.Anonymous1 = EVENT_TRACE_LOGFILEW_0 {
        ProcessTraceMode: PROCESS_TRACE_MODE_REAL_TIME | PROCESS_TRACE_MODE_EVENT_RECORD,
    };
    logfile.Anonymous2 = EVENT_TRACE_LOGFILEW_1 {
        EventRecordCallback: Some(record_callback),
    };
    let trace_handle = unsafe { OpenTraceW(&mut logfile) };
    if trace_handle.Value == u64::MAX {
        let _ = ready.send(Err(
            "OpenTraceW returned INVALID_PROCESSTRACE_HANDLE".to_owned()
        ));
        return Err("OpenTraceW failed".to_owned());
    }
    ready
        .send(Ok((trace_handle, dropped)))
        .map_err(|error| error.to_string())?;
    let result = unsafe { ProcessTrace(&trace_handle, 1, std::ptr::null(), std::ptr::null()) };
    if result != 0 {
        return Err(format!("ProcessTrace failed with Win32 error {result}"));
    }
    Ok(())
}

fn classify(provider: &str, opcode: u8, properties: &[String]) -> SysKind {
    let has = |names: &[&str]| {
        names.iter().any(|name| {
            properties
                .iter()
                .any(|property| property.eq_ignore_ascii_case(name))
        })
    };
    match provider {
        "Process" => {
            if opcode == 2 || has(&["ExitStatus", "ExitTime"]) {
                SysKind::ProcStop
            } else {
                SysKind::ProcStart
            }
        }
        "File" if has(&["ReadSize", "ReadLength", "ReadOffset"]) => SysKind::FileRead,
        "File" if has(&["WriteSize", "WriteLength", "WriteOffset"]) => SysKind::FileWrite,
        "File" if has(&["FileInformationClass", "FileInformation"]) => SysKind::FileSetInfo,
        "File" => match opcode {
            2 => SysKind::FileRead,
            3 => SysKind::FileWrite,
            4 => SysKind::FileClose,
            5 => SysKind::FileSetInfo,
            6 => SysKind::FileDelete,
            _ => SysKind::FileCreate,
        },
        "Registry" => {
            if opcode & 1 == 0 {
                SysKind::RegQuery
            } else {
                SysKind::RegSet
            }
        }
        "Network" => match opcode {
            2 => SysKind::NetDisconnect,
            3 => SysKind::NetSend,
            4 => SysKind::NetRecv,
            5 => SysKind::NetListen,
            6 => SysKind::NetBind,
            _ => SysKind::NetConnect,
        },
        _ => SysKind::ImageLoad,
    }
}

fn property_value(record: &EVENT_RECORD, properties: &[String], names: &[&str]) -> Option<Vec<u8>> {
    let property = properties
        .iter()
        .find(|property| names.iter().any(|name| property.eq_ignore_ascii_case(name)))?;
    let mut wide: Vec<u16> = property.encode_utf16().chain(std::iter::once(0)).collect();
    let descriptor = PROPERTY_DATA_DESCRIPTOR {
        PropertyName: wide.as_mut_ptr() as usize as u64,
        ArrayIndex: u32::MAX,
        Reserved: 0,
    };
    let mut size = 0_u32;
    let size_result =
        unsafe { TdhGetPropertySize(record, 0, std::ptr::null(), 1, &descriptor, &mut size) };
    if size_result != 0 || size == 0 || size > 1024 * 1024 {
        return None;
    }
    let mut value = vec![0_u8; size as usize];
    let result = unsafe {
        TdhGetProperty(
            record,
            0,
            std::ptr::null(),
            1,
            &descriptor,
            size,
            value.as_mut_ptr(),
        )
    };
    (result == 0).then_some(value)
}

fn property_number(record: &EVENT_RECORD, properties: &[String], names: &[&str]) -> Option<u64> {
    let value = property_value(record, properties, names)?;
    match value.len() {
        1 => Some(u64::from(value[0])),
        2 => Some(u64::from(u16::from_le_bytes([value[0], value[1]]))),
        4 => Some(u64::from(u32::from_le_bytes(value[..4].try_into().ok()?))),
        8 => Some(u64::from_le_bytes(value[..8].try_into().ok()?)),
        _ => None,
    }
}

fn property_text(record: &EVENT_RECORD, properties: &[String], names: &[&str]) -> Option<String> {
    let value = property_value(record, properties, names)?;
    if value.len() >= 2 && value.len().is_multiple_of(2) {
        let units = value
            .chunks(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .take_while(|unit| *unit != 0)
            .collect::<Vec<_>>();
        if let Ok(text) = String::from_utf16(&units) {
            if !text.is_empty() {
                return Some(text);
            }
        }
    }
    std::str::from_utf8(&value)
        .ok()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

fn extract_path(bytes: &[u8]) -> Option<String> {
    let mut start = None;
    let mut result = None;
    let mut index = 0;
    while index + 1 < bytes.len() {
        let code = u16::from_le_bytes([bytes[index], bytes[index + 1]]);
        let is_path_char = code == u16::from(b'\\')
            || code == u16::from(b'/')
            || code == u16::from(b':')
            || code >= 0x20;
        if is_path_char && code < 0x7f {
            start.get_or_insert(index);
        } else if let Some(offset) = start.take() {
            if index.saturating_sub(offset) >= 6 {
                result = String::from_utf16(
                    &bytes[offset..index]
                        .chunks(2)
                        .filter(|pair| pair.len() == 2)
                        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                        .collect::<Vec<_>>(),
                )
                .ok()
                .filter(|value| value.contains('\\') || value.contains('/'));
                if result.is_some() {
                    break;
                }
            }
        }
        index += 2;
    }
    result
}
