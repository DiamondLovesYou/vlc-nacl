
#![allow(unused_variables)]

use libc::{int32_t, int64_t, uint32_t};
use std::sync::{Arc, RwLock};
use std::time::{Instant};

use std::collections::{VecDeque, HashMap};
use std::ops::{Range};
use url::Url;

use super::prelude::*;
use super::interface::*;
use super::instance::Instance;
use super::resource::{ResState, ResourceRc};
use super::sys::{PP_Bool, PP_CompletionCallback, PP_Var,
                 PPB_URLLoader_1_0, PPB_URLRequestInfo_1_0,
                 PPB_URLResponseInfo_1_0,
                 PP_URLRequestProperty, PP_URLResponseProperty,
                 PP_Time, PP_FALSE};

pub type UrlLoader = Resource<UrlLoaderState>;
pub type UrlRequestInfo = Resource<UrlRequestInfoState>;
pub type UrlResponseInfo = Resource<UrlResponseInfoState>;

#[derive(Debug)]
pub struct Reader {
    info: Arc<UrlInfo>,
    parts: VecDeque<Range<usize>>,
    opened: Instant,
    cursor: usize,
}

#[derive(Debug)]
pub struct UrlLoaderState {
    id: PP_Resource,
    instance: Instance,

    request: RwLock<Option<UrlRequestInfo>>,
    response: RwLock<Option<UrlResponseInfo>>,

    reader: RwLock<Option<Reader>>,
}

impl UrlLoaderState {
    pub fn get_request(&self) -> Code<Option<UrlRequestInfo>> { Ok(try!(self.request.read()).clone()) }
    pub fn get_response(&self) -> Code<Option<UrlResponseInfo>> { Ok(try!(self.response.read()).clone()) }
}
impl ResourceState for UrlLoaderState {
    fn into_resstate(this: Arc<UrlLoaderState>) -> ResState {
        ResState::UrlLoader(this)
    }
    fn from_resstate(this: Arc<ResourceRc>) -> Code<Resource<UrlLoaderState>> {
        let state = try!(Self::state_from_resstate(&this))
            .clone();

        Ok(Resource::new(this, state))
    }
    fn state_from_resstate(rs: &Arc<ResourceRc>) -> Code<&Arc<UrlLoaderState>> {
        match rs.state() {
            &ResState::UrlLoader(ref u) => Ok(u),
            _ => Err(Error::BadArgument),
        }
    }
    fn resource_id(this: &Arc<UrlLoaderState>) -> PP_Resource { this.id }
    fn resource_instance(this: &Arc<Self>) -> Instance { this.instance.clone() }
}

#[derive(Clone, Debug)]
pub struct RequestInfo {
    url: StringVar,
    method: StringVar,
    record_download_progress: bool,
    record_upload_progress: bool,
    referrer_url: StringVar,
    agent: StringVar,
}

#[derive(Debug)]
pub struct UrlRequestInfoState {
    id: PP_Resource,
    instance: Instance,
    info: RwLock<RequestInfo>,
}
impl UrlRequestInfoState {
    pub fn with_info_ref<F, U>(&self, f: F) -> Code<U>
        where F: FnOnce(&RequestInfo) -> Code<U>,
    {
        let l = try!(self.info.read());
        f(&*l)
    }
    pub fn with_info_mut<F, U>(&self, f: F) -> Code<U>
        where F: FnOnce(&mut RequestInfo) -> Code<U>,
    {
        let mut l = try!(self.info.write());
        f(&mut *l)
    }

    pub fn info(&self) -> Code<RequestInfo> {
        self.with_info_ref(|r| Ok(r.clone()) )
    }
}
impl ResourceState for UrlRequestInfoState {
    fn into_resstate(this: Arc<UrlRequestInfoState>) -> ResState {
        ResState::UrlRequestInfo(this)
    }
    fn from_resstate(this: Arc<ResourceRc>) -> Code<Resource<UrlRequestInfoState>> {
        let state = try!(Self::state_from_resstate(&this))
            .clone();

        Ok(Resource::new(this, state))
    }
    fn state_from_resstate(rs: &Arc<ResourceRc>) -> Code<&Arc<UrlRequestInfoState>> {
        match rs.state() {
            &ResState::UrlRequestInfo(ref u) => Ok(u),
            _ => Err(Error::BadArgument),
        }
    }
    fn resource_id(this: &Arc<UrlRequestInfoState>) -> PP_Resource { this.id }
    fn resource_instance(this: &Arc<Self>) -> Instance { this.instance.clone() }
}

#[derive(Debug)]
pub struct UrlResponseInfoState {
    id: PP_Resource,
    instance: Instance,

    url: StringVar,
    redirect_url: StringVar,
    redirect_method: StringVar,
    status: i32,
    status_line: StringVar,
    headers: StringVar,
}
impl UrlResponseInfoState {

}
impl ResourceState for UrlResponseInfoState {
    fn into_resstate(this: Arc<UrlResponseInfoState>) -> ResState {
        ResState::UrlResponseInfo(this)
    }
    fn from_resstate(this: Arc<ResourceRc>) -> Code<Resource<UrlResponseInfoState>> {
        let state = try!(Self::state_from_resstate(&this))
            .clone();

        Ok(Resource::new(this, state))
    }
    fn state_from_resstate(rs: &Arc<ResourceRc>) -> Code<&Arc<UrlResponseInfoState>> {
        match rs.state() {
            &ResState::UrlResponseInfo(ref u) => Ok(u),
            _ => Err(Error::BadArgument),
        }
    }
    fn resource_id(this: &Arc<UrlResponseInfoState>) -> PP_Resource { this.id }
    fn resource_instance(this: &Arc<Self>) -> Instance { this.instance.clone() }
}

#[derive(Debug)]
pub struct UrlInfo {
    data: Vec<u8>,
    content_type: String,
}

pub struct UrlManager {
    urls: HashMap<Url, Arc<UrlInfo>>,
}

extern "C" fn ppb_url_loader_create(instance: PP_Instance) -> PP_Resource {
    unimplemented!()
}
extern "C" fn ppb_url_loader_is(resource: PP_Resource) -> PP_Bool {
    unimplemented!()
}
extern "C" fn ppb_url_loader_open(loader: PP_Resource,
                                  request_info: PP_Resource,
                                  callback: PP_CompletionCallback) -> int32_t {
    unimplemented!()
}
extern "C" fn ppb_url_loader_follow_redirect(loader: PP_Resource,
                                             callback: PP_CompletionCallback) -> int32_t {
    unimplemented!()
}
extern "C" fn ppb_url_loader_get_upload_progress(loader: PP_Resource,
                                                 bytes_sent: *mut int64_t,
                                                 total_bytes_to_be_sent: *mut int64_t) -> PP_Bool {
    unimplemented!()
}
extern "C" fn ppb_url_loader_get_download_progress(loader: PP_Resource,
                                                   bytes_received: *mut int64_t,
                                                   total_bytes_to_be_received: *mut int64_t) -> PP_Bool {
    unimplemented!()
}
extern "C" fn ppb_url_loader_get_response_info(loader: PP_Resource) -> PP_Resource {
    unimplemented!()
}
extern "C" fn ppb_url_loader_read_response_body(loader: PP_Resource,
                                                buffer: *mut ::libc::c_void,
                                                bytes_to_read: int32_t,
                                                callback: PP_CompletionCallback) -> int32_t {
    unimplemented!()
}
extern "C" fn ppb_url_loader_finish_streaming_to_file(loader: PP_Resource,
                                                      callback: PP_CompletionCallback) -> int32_t {
    unimplemented!()
}
extern "C" fn ppb_url_loader_close(loader: PP_Resource) {
    unimplemented!()
}

static URL_LOADER_INTERFACE: PPB_URLLoader_1_0 = PPB_URLLoader_1_0 {
    Create: Some(ppb_url_loader_create),
    IsURLLoader: Some(ppb_url_loader_is),
    Open: Some(ppb_url_loader_open),
    FollowRedirect: Some(ppb_url_loader_follow_redirect),
    GetUploadProgress: Some(ppb_url_loader_get_upload_progress),
    GetDownloadProgress: Some(ppb_url_loader_get_download_progress),
    GetResponseInfo: Some(ppb_url_loader_get_response_info),
    ReadResponseBody: Some(ppb_url_loader_read_response_body),
    FinishStreamingToFile: Some(ppb_url_loader_finish_streaming_to_file),
    Close: Some(ppb_url_loader_close),
};

extern "C" fn ppb_url_request_info_create(instance: PP_Instance) -> PP_Resource {
    unimplemented!()
}
extern "C" fn ppb_url_request_info_is(resource: PP_Resource) -> PP_Bool {
    unimplemented!()
}
extern "C" fn ppb_url_request_info_set_property(request: PP_Resource,
                                                property: PP_URLRequestProperty,
                                                value: PP_Var) -> PP_Bool {
    unimplemented!()
}
extern "C" fn ppb_url_request_info_append_data_to_body(request: PP_Resource,
                                                       data: *const ::libc::c_void,
                                                       len: uint32_t) -> PP_Bool {
    unimplemented!()
}
extern "C" fn ppb_url_request_info_append_file_to_body(request: PP_Resource,
                                                       file_ref: PP_Resource,
                                                       start_offset: int64_t,
                                                       number_of_bytes: int64_t,
                                                       expected_last_modified_time: PP_Time) -> PP_Bool {
    unimplemented!()
}

static URL_REQUEST_INFO_INTERFACE: PPB_URLRequestInfo_1_0 = PPB_URLRequestInfo_1_0 {
    Create: Some(ppb_url_request_info_create),
    IsURLRequestInfo: Some(ppb_url_request_info_is),
    SetProperty: Some(ppb_url_request_info_set_property),
    AppendDataToBody: Some(ppb_url_request_info_append_data_to_body),
    AppendFileToBody: Some(ppb_url_request_info_append_file_to_body),
};

extern "C" fn ppb_url_response_info_is(resource: PP_Resource) -> PP_Bool {
    PP_FALSE
}
extern "C" fn ppb_url_response_info_get_property(response: PP_Resource,
                                                 property: PP_URLResponseProperty) -> PP_Var {
    unimplemented!()
}
extern "C" fn ppb_url_response_info_get_body_as_file_ref(response: PP_Resource) -> PP_Resource {
    unimplemented!()
}

static URL_RESPONSE_INFO_INTERFACE: PPB_URLResponseInfo_1_0 = PPB_URLResponseInfo_1_0 {
    IsURLResponseInfo: Some(ppb_url_response_info_is),
    GetProperty: Some(ppb_url_response_info_get_property),
    GetBodyAsFileRef: Some(ppb_url_response_info_get_body_as_file_ref),
};

pub static INTERFACES: Interfaces = &[
    ("PPB_URLLoader;1.0", interface_ptr(&URL_LOADER_INTERFACE)),
    ("PPB_URLRequestInfo;1.0", interface_ptr(&URL_REQUEST_INFO_INTERFACE)),
    ("PPB_URLResponseInfo;1.0", interface_ptr(&URL_RESPONSE_INFO_INTERFACE)),
];
