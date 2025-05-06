#![allow(non_camel_case_types)]

use std::os::raw::{
    c_int,
    c_uint,
    c_long,
    c_ulong,
    c_void,
    c_char,
    c_uchar,
};

pub type snd_pcm_info_t = _snd_pcm_info;
pub type _snd_pcm_format = c_int;
pub use self::_snd_pcm_format as snd_pcm_format_t;
pub type _snd_pcm_stream = c_uint;
pub const SND_PCM_FORMAT_FLOAT_LE: _snd_pcm_format = 14;
pub const SND_PCM_STREAM_PLAYBACK: _snd_pcm_stream = 0;
pub const SND_PCM_STREAM_CAPTURE: _snd_pcm_stream = 1;
pub type snd_pcm_hw_params_t = _snd_pcm_hw_params;
pub type snd_pcm_t = _snd_pcm;
pub use self::_snd_pcm_stream as snd_pcm_stream_t;
pub type _snd_pcm_access = c_uint;
pub use self::_snd_pcm_access as snd_pcm_access_t;
pub const SND_PCM_ACCESS_RW_INTERLEAVED: _snd_pcm_access = 3;
pub type snd_pcm_uframes_t = c_ulong;
pub type snd_pcm_sframes_t = c_long;
pub type snd_output_t = _snd_output;
pub type snd_seq_t = _snd_seq;
pub const SND_SEQ_OPEN_OUTPUT: i32 = 1;
pub const SND_SEQ_OPEN_INPUT: i32 = 2; 
pub const SND_SEQ_OPEN_DUPLEX: i32 = SND_SEQ_OPEN_OUTPUT | SND_SEQ_OPEN_INPUT;

pub const SND_SEQ_PORT_CAP_READ: c_uint = 1 << 0;
pub const SND_SEQ_PORT_CAP_WRITE: c_uint = 1 << 1;
pub const SND_SEQ_PORT_CAP_SUBS_READ: c_uint = 1 << 5;
pub const SND_SEQ_PORT_CAP_SUBS_WRITE: c_uint = 1 << 6;
pub const SND_SEQ_PORT_CAP_NO_EXPORT: c_uint = 1 << 7;
pub const SND_SEQ_PORT_TYPE_APPLICATION: c_uint = 1 << 20;
pub const SND_SEQ_PORT_TYPE_MIDI_GENERIC:c_uint = 1 << 1;
pub const SND_SEQ_CLIENT_SYSTEM: u8 = 0;
pub const SND_SEQ_PORT_SYSTEM_ANNOUNCE: u8 = 1;
pub const SND_SEQ_USER_CLIENT: snd_seq_client_type = 1;
pub const SND_SEQ_KERNEL_CLIENT: snd_seq_client_type = 2;
pub type snd_seq_client_type = c_uint;
pub use self::snd_seq_client_type as snd_seq_client_type_t;

pub const SND_SEQ_ADDRESS_SUBSCRIBERS: c_uint = 254;
pub const SND_SEQ_ADDRESS_UNKNOWN: c_uint =	253;
pub const SND_SEQ_QUEUE_DIRECT: c_uint =	253;

pub type snd_seq_event_type = u8;
pub const SND_SEQ_EVENT_NOTEON: snd_seq_event_type = 6;
pub const SND_SEQ_EVENT_NOTEOFF: snd_seq_event_type = 7;
pub const SND_SEQ_EVENT_KEYPRESS: snd_seq_event_type = 8;
pub const SND_SEQ_EVENT_CONTROLLER: snd_seq_event_type = 10;
pub const SND_SEQ_EVENT_PGMCHANGE: snd_seq_event_type = 11;
pub const SND_SEQ_EVENT_CHANPRESS: snd_seq_event_type = 12;
pub const SND_SEQ_EVENT_PITCHBEND: snd_seq_event_type = 13;

pub const SND_SEQ_EVENT_CLIENT_START: snd_seq_event_type = 60;
pub const SND_SEQ_EVENT_CLIENT_EXIT: snd_seq_event_type = 61;
pub const SND_SEQ_EVENT_CLIENT_CHANGE: snd_seq_event_type = 62;
pub const SND_SEQ_EVENT_PORT_START: snd_seq_event_type = 63;
pub const SND_SEQ_EVENT_PORT_EXIT: snd_seq_event_type = 64;
pub const SND_SEQ_EVENT_PORT_CHANGE: snd_seq_event_type = 65;
pub const SND_SEQ_EVENT_PORT_SUBSCRIBED: snd_seq_event_type = 66;
pub const SND_SEQ_EVENT_PORT_UNSUBSCRIBED: snd_seq_event_type = 67;

pub type snd_seq_event_type_t = c_uchar;
pub type snd_seq_tick_time_t = c_uint;
pub type snd_seq_port_subscribe_t = _snd_seq_port_subscribe;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct snd_midi_event {
    _unused: [u8; 0],
}
pub type snd_midi_event_t = snd_midi_event;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _snd_seq_port_subscribe {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _snd_output {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _snd_pcm {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _snd_ctl {
    _unused: [u8; 0],
}
pub type snd_ctl_t = _snd_ctl;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _snd_pcm_info {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _snd_pcm_hw_params {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _snd_seq {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct snd_seq_real_time {
    pub tv_sec: c_uint,
    pub tv_nsec: c_uint,
}
pub type snd_seq_real_time_t = snd_seq_real_time;

#[repr(C)]
#[derive(Copy, Clone)]
pub union snd_seq_timestamp {
    pub tick: snd_seq_tick_time_t,
    pub time: snd_seq_real_time,
    _bindgen_union_align: [u32; 2usize],
}
pub type snd_seq_timestamp_t = snd_seq_timestamp;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct snd_seq_addr {
    pub client: c_uchar,
    pub port: c_uchar,
}
pub type snd_seq_addr_t = snd_seq_addr;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct snd_seq_event {
    pub type_: snd_seq_event_type_t,
    pub flags: c_uchar,
    pub tag: c_uchar,
    pub queue: c_uchar,
    pub time: snd_seq_timestamp_t,
    pub source: snd_seq_addr_t,
    pub dest: snd_seq_addr_t,
    pub data: snd_seq_event__bindgen_ty_1,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct snd_seq_ev_note {
    pub channel: c_uchar,
    pub note: c_uchar,
    pub velocity: c_uchar,
    pub off_velocity: c_uchar,
    pub duration: c_uint,
}
pub type snd_seq_ev_note_t = snd_seq_ev_note;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct snd_seq_ev_ctrl {
    pub channel: c_uchar,
    pub unused: [c_uchar; 3usize],
    pub param: c_uint,
    pub value: c_int,
}
pub type snd_seq_ev_ctrl_t = snd_seq_ev_ctrl;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct snd_seq_ev_raw8 {
    pub d: [c_uchar; 12usize],
}
pub type snd_seq_ev_raw8_t = snd_seq_ev_raw8;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct snd_seq_ev_raw32 {
    pub d: [c_uint; 3usize],
}
pub type snd_seq_ev_raw32_t = snd_seq_ev_raw32;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct snd_seq_ev_ext {
    pub len: c_uint,
    pub ptr: *mut c_void,
}
pub type snd_seq_ev_ext_t = snd_seq_ev_ext;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct snd_seq_ev_queue_control {
    pub queue: c_uchar,
    pub unused: [c_uchar; 3usize],
    pub param: snd_seq_ev_queue_control__bindgen_ty_1,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct snd_seq_queue_skew {
    pub value: c_uint,
    pub base: c_uint,
}
pub type snd_seq_queue_skew_t = snd_seq_queue_skew;

#[repr(C)]
#[derive(Copy, Clone)]
pub union snd_seq_ev_queue_control__bindgen_ty_1 {
    pub value: c_int,
    pub time: snd_seq_timestamp_t,
    pub position: c_uint,
    pub skew: snd_seq_queue_skew_t,
    pub d32: [c_uint; 2usize],
    pub d8: [c_uchar; 8usize],
    _bindgen_union_align: [u32; 2usize],
}
pub type snd_seq_ev_queue_control_t = snd_seq_ev_queue_control;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct snd_seq_connect {
    pub sender: snd_seq_addr_t,
    pub dest: snd_seq_addr_t,
}
pub type snd_seq_connect_t = snd_seq_connect;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct snd_seq_result {
    pub event: c_int,
    pub result: c_int,
}
pub type snd_seq_result_t = snd_seq_result;

#[repr(C)]
#[derive(Copy, Clone)]
pub union snd_seq_event__bindgen_ty_1 {
    pub note: snd_seq_ev_note_t,
    pub control: snd_seq_ev_ctrl_t,
    pub raw8: snd_seq_ev_raw8_t,
    pub raw32: snd_seq_ev_raw32_t,
    pub ext: snd_seq_ev_ext_t,
    pub queue: snd_seq_ev_queue_control_t,
    pub time: snd_seq_timestamp_t,
    pub addr: snd_seq_addr_t,
    pub connect: snd_seq_connect_t,
    pub result: snd_seq_result_t,
    _bindgen_union_align: [u32; 3usize],
}

pub type snd_seq_event_t = snd_seq_event;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _snd_seq_client_info {
    _unused: [u8; 0],
}
pub type snd_seq_client_info_t = _snd_seq_client_info;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _snd_seq_port_info {
    _unused: [u8; 0],
}
pub type snd_seq_port_info_t = _snd_seq_port_info;

#[link(name = "asound")]
extern "C" {
    
    pub fn snd_seq_open(
        handle: *mut *mut snd_seq_t,
        name: *const u8,
        streams: c_int,
        mode: c_int,
    ) -> c_int;
    
    pub fn snd_card_next(card: *mut c_int) -> c_int;
    
    pub fn snd_strerror(errnum: c_int) -> *const c_char;
    
    pub fn snd_ctl_open(
        ctl: *mut *mut snd_ctl_t,
        name: *const u8,
        mode: c_int,
    ) -> c_int;
    
    pub fn snd_ctl_pcm_next_device(
        ctl: *mut snd_ctl_t,
        device: *mut c_int,
    ) -> c_int;
    
    pub fn snd_device_name_hint(
        card: c_int,
        iface: *const u8,
        hints: *mut *mut *mut c_void,
    ) -> c_int;
    
    pub fn snd_device_name_get_hint(
        hint: *const c_void,
        id: *const u8,
    ) -> *mut c_char;
    
    pub fn snd_pcm_open(
        pcm: *mut *mut snd_pcm_t,
        name: *const u8,
        stream: snd_pcm_stream_t,
        mode: c_int,
    ) -> c_int;
    
    pub fn snd_pcm_hw_params_malloc(ptr: *mut *mut snd_pcm_hw_params_t) -> c_int;
    
    pub fn snd_pcm_hw_params_any(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
    ) -> c_int;
    
    pub fn snd_pcm_hw_params_set_access(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        _access: snd_pcm_access_t,
    ) -> c_int;
    
    pub fn snd_pcm_hw_params_set_format(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        val: snd_pcm_format_t,
    ) -> c_int;
    
    pub fn snd_pcm_hw_params_set_rate_near(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        val: *mut c_uint,
        dir: *mut c_int,
    ) -> c_int;
    
    pub fn snd_pcm_hw_params_set_channels(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        val: c_uint,
    ) -> c_int;
    
    pub fn snd_pcm_hw_params(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
    ) -> c_int;
    
    pub fn snd_pcm_hw_params_set_rate_resample(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        val: c_uint,
    ) -> c_int;
    
    pub fn snd_pcm_hw_params_get_buffer_size(
        params: *const snd_pcm_hw_params_t,
        val: *mut snd_pcm_uframes_t,
    ) -> c_int;
    
    pub fn snd_pcm_hw_params_free(obj: *mut snd_pcm_hw_params_t);
    
    pub fn snd_pcm_prepare(pcm: *mut snd_pcm_t) -> c_int;
    
    pub fn snd_pcm_hw_params_get_channels(
        params: *const snd_pcm_hw_params_t,
        val: *mut c_uint,
    ) -> c_int;
    
    pub fn snd_pcm_hw_params_get_rate(
        params: *const snd_pcm_hw_params_t,
        val: *mut c_uint,
        dir: *mut c_int,
    ) -> c_int;
    
    pub fn snd_pcm_hw_params_get_period_size(
        params: *const snd_pcm_hw_params_t,
        frames: *mut snd_pcm_uframes_t,
        dir: *mut c_int,
    ) -> c_int;
    
    pub fn snd_pcm_hw_params_get_period_time(
        params: *const snd_pcm_hw_params_t,
        val: *mut c_uint,
        dir: *mut c_int,
    ) -> c_int;
    
    pub fn snd_pcm_writei(
        pcm: *mut snd_pcm_t,
        buffer: *const c_void,
        size: snd_pcm_uframes_t,
    ) -> snd_pcm_sframes_t;
    pub fn snd_pcm_hw_params_set_periods_near(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        val: *mut c_uint,
        dir: *mut c_int,
    ) -> c_int;
    pub fn snd_pcm_hw_params_set_buffer_size_near(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        val: *mut snd_pcm_uframes_t,
    ) -> c_int;
    
    pub fn snd_pcm_readi(
        pcm: *mut snd_pcm_t,
        buffer: *mut c_void,
        size: snd_pcm_uframes_t,
    ) -> snd_pcm_sframes_t;
    
    pub fn snd_seq_set_client_name(
        seq: *mut snd_seq_t,
        name: *const u8,
    ) -> c_int;
    
    pub fn snd_seq_create_simple_port(
        seq: *mut snd_seq_t,
        name: *const u8,
        caps: c_uint,
        type_: c_uint,
    ) -> c_int;
    
    pub fn snd_seq_port_subscribe_malloc(
        ptr: *mut *mut snd_seq_port_subscribe_t,
    ) -> c_int;
    
    
    pub fn snd_seq_client_info_malloc(
        ptr: *mut *mut snd_seq_client_info_t,
    ) -> c_int;
    
    pub fn snd_seq_port_info_malloc(ptr: *mut *mut snd_seq_port_info_t) -> c_int;
    
    pub fn snd_seq_port_subscribe_set_sender(
        info: *mut snd_seq_port_subscribe_t,
        addr: *const snd_seq_addr_t,
    );
    
    pub fn snd_seq_port_subscribe_set_dest(
        info: *mut snd_seq_port_subscribe_t,
        addr: *const snd_seq_addr_t,
    );
    
    pub fn snd_seq_subscribe_port(
        handle: *mut snd_seq_t,
        sub: *mut snd_seq_port_subscribe_t,
    ) -> c_int;
    
    pub fn snd_seq_client_id(handle: *mut snd_seq_t) -> c_int;
    
    pub fn snd_seq_client_info_set_client(
        info: *mut snd_seq_client_info_t,
        client: c_int,
    );
    
    pub fn snd_seq_query_next_client(
        handle: *mut snd_seq_t,
        info: *mut snd_seq_client_info_t,
    ) -> c_int;
    
    pub fn snd_seq_client_info_get_client(
        info: *const snd_seq_client_info_t,
    ) -> c_int;
    
    pub fn snd_seq_port_info_set_client(
        info: *mut snd_seq_port_info_t,
        client: c_int,
    );
    
    pub fn snd_seq_port_info_set_port(info: *mut snd_seq_port_info_t, port: c_int);
    
    pub fn snd_seq_query_next_port(
        handle: *mut snd_seq_t,
        info: *mut snd_seq_port_info_t,
    ) -> c_int;
    
    pub fn snd_seq_port_info_get_addr(info: *const snd_seq_port_info_t) -> *const snd_seq_addr_t;
    
    pub fn snd_seq_client_info_get_name(
        info: *mut snd_seq_client_info_t,
    ) -> *const c_char;
    
    pub fn snd_seq_client_info_get_type(
        info: *const snd_seq_client_info_t,
    ) -> snd_seq_client_type_t;
    
    pub fn snd_seq_port_info_get_capability(
        info: *const snd_seq_port_info_t,
    ) -> c_uint;
    
    pub fn snd_seq_port_info_get_name(
        info: *const snd_seq_port_info_t,
    ) -> *const c_char;
    
    pub fn snd_seq_unsubscribe_port(
        handle: *mut snd_seq_t,
        sub: *mut snd_seq_port_subscribe_t,
    ) -> c_int;
    
    pub fn snd_seq_event_input(
        handle: *mut snd_seq_t,
        ev: *mut *mut snd_seq_event_t,
    ) -> c_int;
    pub fn snd_midi_event_new(
        bufsize: usize,
        rdev: *mut *mut snd_midi_event_t,
    ) -> c_int;
    pub fn snd_midi_event_free(dev: *mut snd_midi_event_t);
    pub fn snd_midi_event_init(dev: *mut snd_midi_event_t);
    pub fn snd_midi_event_reset_encode(dev: *mut snd_midi_event_t);
    pub fn snd_midi_event_encode(
        dev: *mut snd_midi_event_t,
        buf: *const c_uchar,
        count: c_long,
        ev: *mut snd_seq_event_t,
    ) -> c_long;
    pub fn snd_seq_event_output_direct(
        handle: *mut snd_seq_t,
        ev: *mut snd_seq_event_t,
    ) -> c_int;
}

