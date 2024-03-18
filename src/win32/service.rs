use std::ffi::OsString;
use std::fmt;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_SUCCESS, TRUE};
use windows_sys::Win32::Security;
use windows_sys::Win32::System::Services;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceStatus {
    Querying,
    Running,
    Stopped,
    DoesNotExist,
    Unknown,
}
impl Default for ServiceStatus {
    fn default() -> Self {
        ServiceStatus::Unknown
    }
}

impl fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceStatus::Querying => write!(f, "Querying"),
            ServiceStatus::Running => write!(f, "Running"),
            ServiceStatus::Stopped => write!(f, "Stopped"),
            ServiceStatus::DoesNotExist => write!(f, "Does Not Exist"),
            ServiceStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Clone)]
pub struct Service {
    pub service_name: Vec<u16>,
    service_manager_handle: Security::SC_HANDLE,
    service_handle: Security::SC_HANDLE,
}

impl Service {
    pub fn new(service_name: &str) -> Self {
        Service {
            service_name: OsString::from(service_name)
                .encode_wide()
                .chain(once(0))
                .collect(),
            service_manager_handle: 0,
            service_handle: 0,
        }
    }

    pub fn open(&mut self) -> u32 {
        unsafe {
            self.service_manager_handle =
                Services::OpenSCManagerW(ptr::null(), ptr::null(), Services::SC_MANAGER_ALL_ACCESS);
            if self.service_manager_handle == 0 {
                return GetLastError();
            }

            self.service_handle = Services::OpenServiceW(
                self.service_manager_handle,
                self.service_name.as_ptr(),
                Services::SERVICE_ALL_ACCESS,
            );
            if self.service_handle == 0 {
                return GetLastError();
            }

            return ERROR_SUCCESS;
        }
    }

    pub fn register(&mut self, display_name: &str, binary_path: &str) -> bool {
        let display_name_wide: Vec<u16> = OsString::from(display_name)
            .encode_wide()
            .chain(once(0))
            .collect();
        let binary_path_wide: Vec<u16> = OsString::from(binary_path)
            .encode_wide()
            .chain(once(0))
            .collect();

        unsafe {
            self.service_handle = Services::CreateServiceW(
                self.service_manager_handle,
                self.service_name.as_ptr(),
                display_name_wide.as_ptr(),
                Services::SERVICE_ALL_ACCESS,
                Services::SERVICE_WIN32_OWN_PROCESS,
                Services::SERVICE_AUTO_START,
                Services::SERVICE_ERROR_NORMAL,
                binary_path_wide.as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
            !(self.service_handle == 0)
        }
    }

    pub fn unregister(&mut self) -> bool {
        unsafe {
            if Services::DeleteService(self.service_handle) != 0 {
                Services::CloseServiceHandle(self.service_handle);
                true
            } else {
                false
            }
        }
    }

    pub fn start(&self) -> bool {
        unsafe { Services::StartServiceW(self.service_handle, 0, ptr::null_mut()) == TRUE }
    }

    pub fn stop(&self) -> bool {
        let mut status: Services::SERVICE_STATUS = unsafe { std::mem::zeroed() };
        unsafe {
            Services::ControlService(
                self.service_handle,
                Services::SERVICE_CONTROL_STOP,
                &mut status,
            ) == TRUE
        }
    }

    pub fn query_status(&self) -> ServiceStatus {
        let mut status: Services::SERVICE_STATUS = unsafe { std::mem::zeroed() };
        unsafe {
            Services::QueryServiceStatus(self.service_handle, &mut status);
        }
        match status.dwCurrentState {
            Services::SERVICE_RUNNING
            | Services::SERVICE_START_PENDING
            | Services::SERVICE_CONTINUE_PENDING => ServiceStatus::Running,
            Services::SERVICE_STOPPED
            | Services::SERVICE_PAUSED
            | Services::SERVICE_STOP_PENDING
            | Services::SERVICE_PAUSE_PENDING => ServiceStatus::Stopped,
            _ => ServiceStatus::Unknown,
        }
    }
}
