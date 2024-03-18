use crate::types;
use crate::utils;
use std::ptr;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Security::SecurityImpersonation;
use windows_sys::Win32::Security::TOKEN_ALL_ACCESS;
use windows_sys::Win32::Security::{DuplicateTokenEx, TokenPrimary};
use windows_sys::Win32::System::Environment::*;
use windows_sys::Win32::System::RemoteDesktop::*;
use windows_sys::Win32::System::Services::*;
use windows_sys::Win32::System::Threading::CREATE_UNICODE_ENVIRONMENT;
use windows_sys::Win32::System::Threading::{
    CreateProcessAsUserW, TerminateProcess, PROCESS_INFORMATION, STARTUPINFOW,
};

static mut C_SERVICE_STATUS_HANDLE: SERVICE_STATUS_HANDLE = 0;
static mut C_SERVICE_STATUS: SERVICE_STATUS = unsafe { std::mem::zeroed() };
static mut H_DESKTOP_ROCESS: HANDLE = 0;

unsafe extern "system" fn service_ctrl_handler(ctrl: u32) {
    match ctrl {
        SERVICE_CONTROL_PAUSE
        | SERVICE_CONTROL_STOP
        | SERVICE_CONTROL_PRESHUTDOWN
        | SERVICE_CONTROL_SHUTDOWN => {
            C_SERVICE_STATUS.dwCurrentState = SERVICE_STOPPED;
            if H_DESKTOP_ROCESS != 0 {
                TerminateProcess(H_DESKTOP_ROCESS, 1);
                CloseHandle(H_DESKTOP_ROCESS);
            }
            SetServiceStatus(C_SERVICE_STATUS_HANDLE, &C_SERVICE_STATUS);
        }
        _ => {}
    }
}

unsafe extern "system" fn service_main(_: u32, _: *mut *mut u16) {
    C_SERVICE_STATUS_HANDLE = RegisterServiceCtrlHandlerW(
        types::DESK_SEVICE_NAME.as_ptr() as *const u16,
        Some(service_ctrl_handler),
    );
    if C_SERVICE_STATUS_HANDLE == 0 {
        return;
    }

    C_SERVICE_STATUS = SERVICE_STATUS {
        dwServiceType: SERVICE_WIN32_OWN_PROCESS,
        dwControlsAccepted: SERVICE_ACCEPT_STOP
            | SERVICE_ACCEPT_PAUSE_CONTINUE
            | SERVICE_ACCEPT_SHUTDOWN
            | SERVICE_ACCEPT_PRESHUTDOWN,
        dwWin32ExitCode: 0,
        dwCheckPoint: 0,
        dwServiceSpecificExitCode: 0,
        dwWaitHint: 0,
        dwCurrentState: SERVICE_RUNNING,
    };
    if SetServiceStatus(C_SERVICE_STATUS_HANDLE, &C_SERVICE_STATUS) == FALSE {
        //output errors log
        return;
    }
    if let Some(mut execute_path) = utils::get_executable_path() {
        execute_path.push_str(" -main");
        H_DESKTOP_ROCESS = launch_desktop_process(execute_path);
        log::info!("launched desktop process:{}", H_DESKTOP_ROCESS);
    } else {
        //output errors log
    }
}

pub fn service_dispatch() -> i32 {
    let service_table: &[SERVICE_TABLE_ENTRYW] = &[
        SERVICE_TABLE_ENTRYW {
            lpServiceName: types::DESK_SEVICE_NAME.as_ptr() as *mut u16,
            lpServiceProc: Some(service_main),
        },
        SERVICE_TABLE_ENTRYW {
            lpServiceName: ptr::null_mut(),
            lpServiceProc: None,
        },
    ];
    unsafe { StartServiceCtrlDispatcherW(service_table.as_ptr()) }
}

//Launching a process as a specific user in Windows.
pub fn launch_desktop_process(execute_path: String) -> isize {
    unsafe {
        let session_id = WTSGetActiveConsoleSessionId();
        let mut h_token: HANDLE = 0;
        let mut h_token_dup: HANDLE = 0;
        let mut lp_environment: *mut std::ffi::c_void = std::ptr::null_mut();

        let mut si: STARTUPINFOW = std::mem::zeroed();
        si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        let mut pi: PROCESS_INFORMATION = std::mem::zeroed();
        let mut h_process: HANDLE = 0;

        if WTSQueryUserToken(session_id, &mut h_token) == FALSE {
            let err_code = GetLastError();
            log::error!(
                "WTSQueryUserToken fail: {}, {}",
                err_code,
                utils::get_last_error_message(err_code)
            );
            CloseHandle(h_token);
            return h_process;
        }

        if DuplicateTokenEx(
            h_token,
            TOKEN_ALL_ACCESS,
            std::ptr::null(),
            SecurityImpersonation,
            TokenPrimary,
            &mut h_token_dup,
        ) == FALSE
        {
            let err_code = GetLastError();
            log::error!(
                "DuplicateTokenEx fail: {}, {}",
                err_code,
                utils::get_last_error_message(err_code)
            );
            CloseHandle(h_token_dup);
            CloseHandle(h_token);
            return h_process;
        }

        if CreateEnvironmentBlock(&mut lp_environment as *mut _ as *mut _, h_token_dup, 0) == FALSE
        {
            log::error!("CreateEnvironmentBlock fail: {}", GetLastError());
            DestroyEnvironmentBlock(lp_environment);
            CloseHandle(h_token_dup);
            CloseHandle(h_token);
            return h_process;
        }

        let desktop = widestring::U16CString::from_str("winsta0\\default").unwrap();
        si.lpDesktop = desktop.as_ptr() as *mut _;

        log::info!("CreateProcessAsUserW with execute path: {}", execute_path);

        if CreateProcessAsUserW(
            h_token_dup,
            std::ptr::null(),
            widestring::U16CString::from_str(execute_path)
                .unwrap()
                .as_ptr() as *mut u16,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            FALSE,
            CREATE_UNICODE_ENVIRONMENT,
            lp_environment,
            std::ptr::null(),
            &mut si,
            &mut pi,
        ) == TRUE
        {
            //CloseHandle(pi.hProcess);
            CloseHandle(pi.hThread);
            h_process = pi.hProcess;
        } else {
            let err_code = GetLastError();
            log::error!(
                "CreateProcessAsUserW fail: {}, {}",
                err_code,
                utils::get_last_error_message(err_code)
            );
        }

        DestroyEnvironmentBlock(lp_environment);
        CloseHandle(h_token_dup);
        CloseHandle(h_token);
        return h_process;
    }
}
