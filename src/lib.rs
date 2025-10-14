#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString, c_char};

use autd3capi_driver::*;

use autd3_link_ethercrab::{EtherCrabOption as RawOption, *};

#[repr(C)]
pub struct EtherCrabOption {
    pub ifname: *const c_char,
    pub dc_configuration_sync0_period: Duration,
    pub state_check_period: Duration,
    pub sync_tolerance: Duration,
    pub sync_timeout: Duration,
}

impl TryFrom<EtherCrabOption> for EtherCrabOptionFull {
    type Error = std::str::Utf8Error;

    fn try_from(value: EtherCrabOption) -> Result<Self, Self::Error> {
        unsafe {
            let ifname = if value.ifname.is_null() {
                None
            } else {
                std::ffi::CStr::from_ptr(value.ifname)
                    .to_str()
                    .map(String::from)
                    .map(Some)?
            };
            Ok(RawOption {
                ifname,
                state_check_period: value.state_check_period.into(),
                sync0_period: value.dc_configuration_sync0_period.into(),
                sync_tolerance: value.sync_tolerance.into(),
                sync_timeout: value.sync_timeout.into(),
            }
            .into())
        }
    }
}

#[unsafe(no_mangle)]
#[must_use]
pub unsafe extern "C" fn AUTDLinkEtherCrab(
    err_handler: ConstPtr,
    err_context: ConstPtr,
    option: EtherCrabOption,
) -> ResultLink {
    unsafe {
        let out_func = move |slave: usize, status: Status| {
            let (out_f, context) = {
                (
                    std::mem::transmute::<ConstPtr, unsafe extern "C" fn(ConstPtr, u32, Status)>(
                        err_handler,
                    ),
                    err_context,
                )
            };
            out_f(context, slave as _, status);
        };
        option
            .try_into()
            .map(|option: EtherCrabOptionFull| EtherCrab::new(out_func, option))
            .into()
    }
}

#[unsafe(no_mangle)]
#[must_use]
#[allow(unused_variables)]
pub unsafe extern "C" fn AUTDLinkEtherCrabIsDefault(option: EtherCrabOption) -> bool {
    option
        .try_into()
        .is_ok_and(|option: EtherCrabOptionFull| option == EtherCrabOptionFull::default())
}

#[unsafe(no_mangle)]
#[must_use]
pub unsafe extern "C" fn AUTDLinkEtherCrabStatusGetMsg(src: Status, dst: *mut c_char) -> u32 {
    unsafe {
        let msg = format!("{src}");
        if dst.is_null() {
            return msg.len() as u32 + 1;
        }
        let c_string = CString::new(msg).unwrap();
        let c_str: &CStr = c_string.as_c_str();
        strcpy(dst, c_str.as_ptr());
        0
    }
}
