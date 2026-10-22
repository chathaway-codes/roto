//! Zero-allocation `#![no_std]` gRPC Server Reflection handler supporting both
//! `grpc.reflection.v1.ServerReflection` and `grpc.reflection.v1alpha.ServerReflection`.

use google_protobuf_roto::google::protobuf::{
    DescriptorProtoView, EnumDescriptorProtoView, FieldDescriptorProtoView, FileDescriptorProtoView,
};
use roto::runtime::DecodeError;
use roto_grpc::{GrpcCode, GrpcStatus, RecvStream, Route, SendStream};

use crate::grpc::reflection::{v1, v1alpha};
use crate::services::grpc::reflection::{v1 as v1_grpc, v1alpha as v1alpha_grpc};

/// Maximum number of unique `FileDescriptorProto` entries indexed on the stack.
pub const MAX_DESCRIPTOR_FILES: usize = 64;

/// Maximum number of unique service names returned by `list_services`.
pub const MAX_REFLECTION_SERVICES: usize = 64;

/// Maximum number of extension field numbers returned by `all_extension_numbers_of_type`.
pub const MAX_EXTENSION_NUMBERS: usize = 64;

type DescriptorClosure<'a> = ([&'a [u8]; MAX_DESCRIPTOR_FILES], usize);
type ExtensionNumbers = ([i32; MAX_EXTENSION_NUMBERS], usize);

/// Zero-allocation `#![no_std]` gRPC Server Reflection handler.
///
/// Implements both [`v1_grpc::ServerReflection`] / [`v1_grpc::AsyncServerReflection`] and
/// [`v1alpha_grpc::ServerReflection`] / [`v1alpha_grpc::AsyncServerReflection`] over static
/// `FileDescriptorProto` slices and registered [`Route`] tables.
#[derive(Clone, Copy)]
pub struct ReflectionService<'a> {
    routes: &'a [&'a dyn Route],
    service_names: &'a [&'a str],
    file_descriptor_sets: &'a [&'a [&'a [u8]]],
    extra_files: &'a [&'a [u8]],
    include_reflection: bool,
}

impl core::fmt::Debug for ReflectionService<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReflectionService")
            .field("routes_len", &self.routes.len())
            .field("service_names", &self.service_names)
            .field("file_descriptor_sets_len", &self.file_descriptor_sets.len())
            .field("extra_files_len", &self.extra_files.len())
            .field("include_reflection", &self.include_reflection)
            .finish()
    }
}

impl<'a> ReflectionService<'a> {
    /// Creates a reflection service inspecting the service names and `file_descriptors()`
    /// of `routes`.
    ///
    /// By default, `grpc.reflection.v1.ServerReflection` and
    /// `grpc.reflection.v1alpha.ServerReflection` (and their file descriptors) are also
    /// included; use [`Self::with_include_reflection_services`] to customize this.
    #[inline]
    pub const fn new(routes: &'a [&'a dyn Route]) -> Self {
        Self {
            routes,
            service_names: &[],
            file_descriptor_sets: &[],
            extra_files: &[],
            include_reflection: true,
        }
    }

    /// Creates a reflection service from explicit service names and `FILE_DESCRIPTOR_SET` slices
    /// (useful with [`roto_grpc::AsyncGrpcServer`] tuple routers where a `&[&dyn Route]` slice
    /// is not already constructed).
    #[inline]
    pub const fn from_descriptors(
        service_names: &'a [&'a str],
        file_descriptor_sets: &'a [&'a [&'a [u8]]],
    ) -> Self {
        Self {
            routes: &[],
            service_names,
            file_descriptor_sets,
            extra_files: &[],
            include_reflection: true,
        }
    }

    /// Registers additional wire-encoded `FileDescriptorProto` slices (such as extension-only
    /// `.proto` files not directly imported by a service).
    #[must_use]
    #[inline]
    pub const fn with_extra_descriptors(mut self, extra_files: &'a [&'a [u8]]) -> Self {
        self.extra_files = extra_files;
        self
    }

    /// Configures whether `grpc.reflection.v1.ServerReflection` and
    /// `grpc.reflection.v1alpha.ServerReflection` (and their descriptors) are automatically
    /// advertised alongside the user's services. Defaults to `true`.
    #[must_use]
    #[inline]
    pub const fn with_include_reflection_services(mut self, include: bool) -> Self {
        self.include_reflection = include;
        self
    }

    /// Wraps this handler in a `grpc.reflection.v1.ServerReflectionServer`.
    #[must_use]
    #[inline]
    pub const fn v1_server(&self) -> v1_grpc::ServerReflectionServer<Self> {
        v1_grpc::ServerReflectionServer::new(*self)
    }

    /// Consumes this handler and wraps it in a `grpc.reflection.v1.ServerReflectionServer`.
    #[must_use]
    #[inline]
    pub const fn into_v1_server(self) -> v1_grpc::ServerReflectionServer<Self> {
        v1_grpc::ServerReflectionServer::new(self)
    }

    /// Wraps this handler in a `grpc.reflection.v1alpha.ServerReflectionServer`.
    #[must_use]
    #[inline]
    pub const fn v1alpha_server(&self) -> v1alpha_grpc::ServerReflectionServer<Self> {
        v1alpha_grpc::ServerReflectionServer::new(*self)
    }

    /// Consumes this handler and wraps it in a `grpc.reflection.v1alpha.ServerReflectionServer`.
    #[must_use]
    #[inline]
    pub const fn into_v1alpha_server(self) -> v1alpha_grpc::ServerReflectionServer<Self> {
        v1alpha_grpc::ServerReflectionServer::new(self)
    }

    fn collect_service_names(&self) -> ([&'a str; MAX_REFLECTION_SERVICES], usize) {
        let mut out = [""; MAX_REFLECTION_SERVICES];
        let mut len = 0usize;

        let mut push_unique = |name: &'a str| {
            if !name.is_empty() && len < MAX_REFLECTION_SERVICES && !out[..len].contains(&name) {
                out[len] = name;
                len += 1;
            }
        };

        for route in self.routes {
            push_unique(route.service_name());
        }
        for &name in self.service_names {
            push_unique(name);
        }
        if self.include_reflection {
            push_unique(v1_grpc::server_reflection::SERVICE_NAME);
            push_unique(v1alpha_grpc::server_reflection::SERVICE_NAME);
        }

        (out, len)
    }

    fn collect_files(&self) -> Result<DescriptorClosure<'a>, GrpcStatus<'static>> {
        let mut files = [&[][..]; MAX_DESCRIPTOR_FILES];
        let mut len = 0usize;

        let mut push_file = |raw: &'a [u8]| -> Result<(), GrpcStatus<'static>> {
            if raw.is_empty() || len >= MAX_DESCRIPTOR_FILES {
                return Ok(());
            }
            if files[..len].contains(&raw) {
                return Ok(());
            }
            let fd = FileDescriptorProtoView::from_bytes(raw).map_err(|_| {
                GrpcStatus::internal("corrupt FileDescriptorProto in descriptor set")
            })?;
            let name = fd
                .name()
                .map_err(|_| GrpcStatus::internal("corrupt FileDescriptorProto.name"))?;
            for &existing in &files[..len] {
                if let Ok(existing_fd) = FileDescriptorProtoView::from_bytes(existing) {
                    if existing_fd.name() == Ok(name) {
                        return Ok(());
                    }
                }
            }
            files[len] = raw;
            len += 1;
            Ok(())
        };

        for route in self.routes {
            for &raw in route.file_descriptors() {
                push_file(raw)?;
            }
        }
        for &set in self.file_descriptor_sets {
            for &raw in set {
                push_file(raw)?;
            }
        }
        for &raw in self.extra_files {
            push_file(raw)?;
        }
        if self.include_reflection {
            for &raw in crate::FILE_DESCRIPTORS {
                push_file(raw)?;
            }
        }

        Ok((files, len))
    }

    fn resolve_file_by_filename(
        &self,
        filename: &str,
    ) -> Result<Option<DescriptorClosure<'a>>, GrpcStatus<'static>> {
        if filename.is_empty() {
            return Ok(None);
        }
        let (files, file_count) = self.collect_files()?;
        for idx in 0..file_count {
            let fd = FileDescriptorProtoView::from_bytes(files[idx])
                .map_err(|_| GrpcStatus::internal("corrupt FileDescriptorProto"))?;
            let name = fd
                .name()
                .map_err(|_| GrpcStatus::internal("corrupt FileDescriptorProto.name"))?;
            if name == filename {
                return Self::collect_closure(&files[..file_count], idx).map(Some);
            }
        }
        Ok(None)
    }

    fn resolve_file_containing_symbol(
        &self,
        symbol: &str,
    ) -> Result<Option<DescriptorClosure<'a>>, GrpcStatus<'static>> {
        let norm_symbol = symbol.strip_prefix('.').unwrap_or(symbol);
        if norm_symbol.is_empty() {
            return Ok(None);
        }
        let (files, file_count) = self.collect_files()?;
        for idx in 0..file_count {
            let fd = FileDescriptorProtoView::from_bytes(files[idx])
                .map_err(|_| GrpcStatus::internal("corrupt FileDescriptorProto"))?;
            if file_contains_symbol(&fd, norm_symbol)
                .map_err(|_| GrpcStatus::internal("corrupt FileDescriptorProto symbol table"))?
            {
                return Self::collect_closure(&files[..file_count], idx).map(Some);
            }
        }
        Ok(None)
    }

    fn resolve_file_containing_extension(
        &self,
        containing_type: &str,
        extension_number: i32,
    ) -> Result<Option<DescriptorClosure<'a>>, GrpcStatus<'static>> {
        let norm_type = containing_type.strip_prefix('.').unwrap_or(containing_type);
        if norm_type.is_empty() || extension_number <= 0 {
            return Ok(None);
        }
        let (files, file_count) = self.collect_files()?;
        for idx in 0..file_count {
            let fd = FileDescriptorProtoView::from_bytes(files[idx])
                .map_err(|_| GrpcStatus::internal("corrupt FileDescriptorProto"))?;
            if file_contains_extension(&fd, norm_type, extension_number)
                .map_err(|_| GrpcStatus::internal("corrupt FileDescriptorProto extensions"))?
            {
                return Self::collect_closure(&files[..file_count], idx).map(Some);
            }
        }
        Ok(None)
    }

    fn resolve_all_extension_numbers_of_type(
        &self,
        base_type_name: &str,
    ) -> Result<Option<ExtensionNumbers>, GrpcStatus<'static>> {
        let norm_type = base_type_name.strip_prefix('.').unwrap_or(base_type_name);
        if norm_type.is_empty() {
            return Ok(None);
        }
        let (files, file_count) = self.collect_files()?;
        let mut type_exists = false;
        let mut nums = [0i32; MAX_EXTENSION_NUMBERS];
        let mut num_count = 0usize;

        for &raw in &files[..file_count] {
            let fd = FileDescriptorProtoView::from_bytes(raw)
                .map_err(|_| GrpcStatus::internal("corrupt FileDescriptorProto"))?;
            if !type_exists
                && file_contains_message_type(&fd, norm_type)
                    .map_err(|_| GrpcStatus::internal("corrupt FileDescriptorProto messages"))?
            {
                type_exists = true;
            }
            collect_file_extension_numbers(&fd, norm_type, &mut nums, &mut num_count)
                .map_err(|_| GrpcStatus::internal("corrupt FileDescriptorProto extensions"))?;
        }

        if !type_exists && num_count == 0 {
            return Ok(None);
        }

        // Insertion sort for deterministic ascending order of extension numbers.
        for i in 1..num_count {
            let key = nums[i];
            let mut j = i;
            while j > 0 && nums[j - 1] > key {
                nums[j] = nums[j - 1];
                j -= 1;
            }
            nums[j] = key;
        }

        Ok(Some((nums, num_count)))
    }

    fn collect_closure(
        files: &[&'a [u8]],
        root_idx: usize,
    ) -> Result<DescriptorClosure<'a>, GrpcStatus<'static>> {
        let mut out = [&[][..]; MAX_DESCRIPTOR_FILES];
        let mut queue = [0usize; MAX_DESCRIPTOR_FILES];
        let mut visited: u64 = 1u64 << root_idx;
        queue[0] = root_idx;
        let mut head = 0usize;
        let mut tail = 1usize;

        while head < tail {
            let idx = queue[head];
            out[head] = files[idx];
            head += 1;

            let fd = FileDescriptorProtoView::from_bytes(files[idx])
                .map_err(|_| GrpcStatus::internal("corrupt FileDescriptorProto"))?;
            for dep_res in fd.dependency() {
                let dep_name = dep_res
                    .map_err(|_| GrpcStatus::internal("corrupt FileDescriptorProto.dependency"))?;
                for (cand_idx, &cand_bytes) in files.iter().enumerate() {
                    if (visited & (1u64 << cand_idx)) != 0 {
                        continue;
                    }
                    let cand_fd = FileDescriptorProtoView::from_bytes(cand_bytes)
                        .map_err(|_| GrpcStatus::internal("corrupt FileDescriptorProto"))?;
                    let cand_name = cand_fd
                        .name()
                        .map_err(|_| GrpcStatus::internal("corrupt FileDescriptorProto.name"))?;
                    if cand_name == dep_name && tail < MAX_DESCRIPTOR_FILES {
                        visited |= 1u64 << cand_idx;
                        queue[tail] = cand_idx;
                        tail += 1;
                        break;
                    }
                }
            }
        }

        Ok((out, tail))
    }

    fn handle_v1_request<'b>(
        &self,
        req: v1::ServerReflectionRequestView<'_>,
        resp: &mut v1::ServerReflectionResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        use v1::server_reflection_request::MessageRequest;

        let host = req
            .host()
            .map_err(|_| GrpcStatus::invalid_argument("invalid UTF-8 in host"))?;
        let msg_req = req
            .message_request()
            .map_err(|_| GrpcStatus::invalid_argument("invalid message_request"))?;

        match msg_req {
            Some(MessageRequest::ListServices(_)) => {
                let (names, count) = self.collect_service_names();
                resp.write_list_services_response(|ls| {
                    for &name in names[..count].iter().rev() {
                        ls.write_service(|s| {
                            s.write_name(name);
                        });
                    }
                });
            }
            Some(MessageRequest::FileByFilename(filename)) => {
                match self.resolve_file_by_filename(filename)? {
                    Some((closure, count)) => {
                        resp.write_file_descriptor_response(|fdr| {
                            for &fd_bytes in closure[..count].iter().rev() {
                                fdr.write_file_descriptor_proto(fd_bytes);
                            }
                        });
                    }
                    None => {
                        resp.write_error_response(|err| {
                            err.write_error_message("file not found")
                                .write_error_code(GrpcCode::NotFound as i32);
                        });
                    }
                }
            }
            Some(MessageRequest::FileContainingSymbol(symbol)) => {
                match self.resolve_file_containing_symbol(symbol)? {
                    Some((closure, count)) => {
                        resp.write_file_descriptor_response(|fdr| {
                            for &fd_bytes in closure[..count].iter().rev() {
                                fdr.write_file_descriptor_proto(fd_bytes);
                            }
                        });
                    }
                    None => {
                        resp.write_error_response(|err| {
                            err.write_error_message("symbol not found")
                                .write_error_code(GrpcCode::NotFound as i32);
                        });
                    }
                }
            }
            Some(MessageRequest::FileContainingExtension(ext_req)) => {
                let containing_type = ext_req
                    .containing_type()
                    .map_err(|_| GrpcStatus::invalid_argument("invalid containing_type"))?;
                let extension_number = ext_req
                    .extension_number()
                    .map_err(|_| GrpcStatus::invalid_argument("invalid extension_number"))?;
                match self.resolve_file_containing_extension(containing_type, extension_number)? {
                    Some((closure, count)) => {
                        resp.write_file_descriptor_response(|fdr| {
                            for &fd_bytes in closure[..count].iter().rev() {
                                fdr.write_file_descriptor_proto(fd_bytes);
                            }
                        });
                    }
                    None => {
                        resp.write_error_response(|err| {
                            err.write_error_message("extension not found")
                                .write_error_code(GrpcCode::NotFound as i32);
                        });
                    }
                }
            }
            Some(MessageRequest::AllExtensionNumbersOfType(base_type)) => {
                match self.resolve_all_extension_numbers_of_type(base_type)? {
                    Some((nums, count)) => {
                        resp.write_all_extension_numbers_response(|enr| {
                            if count > 0 {
                                enr.write_packed_extension_number(&nums[..count]);
                            }
                            enr.write_base_type_name(base_type);
                        });
                    }
                    None => {
                        resp.write_error_response(|err| {
                            err.write_error_message("type not found")
                                .write_error_code(GrpcCode::NotFound as i32);
                        });
                    }
                }
            }
            None => {
                resp.write_error_response(|err| {
                    err.write_error_message("missing message_request")
                        .write_error_code(GrpcCode::InvalidArgument as i32);
                });
            }
        }

        resp.write_original_request(|orig| {
            let _ = orig.writer.write_bytes(req.as_bytes());
        })
        .write_valid_host(host);

        Ok(())
    }

    fn handle_v1alpha_request<'b>(
        &self,
        req: v1alpha::ServerReflectionRequestView<'_>,
        resp: &mut v1alpha::ServerReflectionResponseBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        use v1alpha::server_reflection_request::MessageRequest;

        let host = req
            .host()
            .map_err(|_| GrpcStatus::invalid_argument("invalid UTF-8 in host"))?;
        let msg_req = req
            .message_request()
            .map_err(|_| GrpcStatus::invalid_argument("invalid message_request"))?;

        match msg_req {
            Some(MessageRequest::ListServices(_)) => {
                let (names, count) = self.collect_service_names();
                resp.write_list_services_response(|ls| {
                    for &name in names[..count].iter().rev() {
                        ls.write_service(|s| {
                            s.write_name(name);
                        });
                    }
                });
            }
            Some(MessageRequest::FileByFilename(filename)) => {
                match self.resolve_file_by_filename(filename)? {
                    Some((closure, count)) => {
                        resp.write_file_descriptor_response(|fdr| {
                            for &fd_bytes in closure[..count].iter().rev() {
                                fdr.write_file_descriptor_proto(fd_bytes);
                            }
                        });
                    }
                    None => {
                        resp.write_error_response(|err| {
                            err.write_error_message("file not found")
                                .write_error_code(GrpcCode::NotFound as i32);
                        });
                    }
                }
            }
            Some(MessageRequest::FileContainingSymbol(symbol)) => {
                match self.resolve_file_containing_symbol(symbol)? {
                    Some((closure, count)) => {
                        resp.write_file_descriptor_response(|fdr| {
                            for &fd_bytes in closure[..count].iter().rev() {
                                fdr.write_file_descriptor_proto(fd_bytes);
                            }
                        });
                    }
                    None => {
                        resp.write_error_response(|err| {
                            err.write_error_message("symbol not found")
                                .write_error_code(GrpcCode::NotFound as i32);
                        });
                    }
                }
            }
            Some(MessageRequest::FileContainingExtension(ext_req)) => {
                let containing_type = ext_req
                    .containing_type()
                    .map_err(|_| GrpcStatus::invalid_argument("invalid containing_type"))?;
                let extension_number = ext_req
                    .extension_number()
                    .map_err(|_| GrpcStatus::invalid_argument("invalid extension_number"))?;
                match self.resolve_file_containing_extension(containing_type, extension_number)? {
                    Some((closure, count)) => {
                        resp.write_file_descriptor_response(|fdr| {
                            for &fd_bytes in closure[..count].iter().rev() {
                                fdr.write_file_descriptor_proto(fd_bytes);
                            }
                        });
                    }
                    None => {
                        resp.write_error_response(|err| {
                            err.write_error_message("extension not found")
                                .write_error_code(GrpcCode::NotFound as i32);
                        });
                    }
                }
            }
            Some(MessageRequest::AllExtensionNumbersOfType(base_type)) => {
                match self.resolve_all_extension_numbers_of_type(base_type)? {
                    Some((nums, count)) => {
                        resp.write_all_extension_numbers_response(|enr| {
                            if count > 0 {
                                enr.write_packed_extension_number(&nums[..count]);
                            }
                            enr.write_base_type_name(base_type);
                        });
                    }
                    None => {
                        resp.write_error_response(|err| {
                            err.write_error_message("type not found")
                                .write_error_code(GrpcCode::NotFound as i32);
                        });
                    }
                }
            }
            None => {
                resp.write_error_response(|err| {
                    err.write_error_message("missing message_request")
                        .write_error_code(GrpcCode::InvalidArgument as i32);
                });
            }
        }

        resp.write_original_request(|orig| {
            let _ = orig.writer.write_bytes(req.as_bytes());
        })
        .write_valid_host(host);

        Ok(())
    }
}

impl v1_grpc::ServerReflection for ReflectionService<'_> {
    fn server_reflection_info<'a, 'b>(
        &self,
        rx: &mut impl RecvStream<v1::ServerReflectionRequestView<'a>>,
        tx: &mut impl SendStream<v1::ServerReflectionResponseBuilder<'b>>,
    ) -> Result<(), GrpcStatus<'static>> {
        while let Some(req) = rx
            .next()
            .map_err(|_| GrpcStatus::invalid_argument("malformed ServerReflectionRequest"))?
        {
            let mut handler_err = None;
            tx.send(|resp| {
                if let Err(err) = self.handle_v1_request(req, resp) {
                    handler_err = Some(err);
                }
            })?;
            if let Some(err) = handler_err {
                return Err(err);
            }
        }
        Ok(())
    }
}

impl v1_grpc::AsyncServerReflection for ReflectionService<'_> {
    async fn server_reflection_info<'a, 'r, 'b>(
        &'a self,
        rx: &'a mut impl RecvStream<v1::ServerReflectionRequestView<'r>>,
        tx: &'a mut impl SendStream<v1::ServerReflectionResponseBuilder<'b>>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as v1_grpc::ServerReflection>::server_reflection_info(self, rx, tx)
    }
}

impl v1alpha_grpc::ServerReflection for ReflectionService<'_> {
    fn server_reflection_info<'a, 'b>(
        &self,
        rx: &mut impl RecvStream<v1alpha::ServerReflectionRequestView<'a>>,
        tx: &mut impl SendStream<v1alpha::ServerReflectionResponseBuilder<'b>>,
    ) -> Result<(), GrpcStatus<'static>> {
        while let Some(req) = rx
            .next()
            .map_err(|_| GrpcStatus::invalid_argument("malformed ServerReflectionRequest"))?
        {
            let mut handler_err = None;
            tx.send(|resp| {
                if let Err(err) = self.handle_v1alpha_request(req, resp) {
                    handler_err = Some(err);
                }
            })?;
            if let Some(err) = handler_err {
                return Err(err);
            }
        }
        Ok(())
    }
}

impl v1alpha_grpc::AsyncServerReflection for ReflectionService<'_> {
    async fn server_reflection_info<'a, 'r, 'b>(
        &'a self,
        rx: &'a mut impl RecvStream<v1alpha::ServerReflectionRequestView<'r>>,
        tx: &'a mut impl SendStream<v1alpha::ServerReflectionResponseBuilder<'b>>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as v1alpha_grpc::ServerReflection>::server_reflection_info(self, rx, tx)
    }
}

#[inline]
fn strip_dot_prefix<'s>(s: &'s str, prefix: &str) -> Option<&'s str> {
    if prefix.is_empty() {
        return Some(s);
    }
    let rest = s.strip_prefix(prefix)?;
    rest.strip_prefix('.')
}

fn file_contains_symbol(
    fd: &FileDescriptorProtoView<'_>,
    norm_symbol: &str,
) -> Result<bool, DecodeError> {
    let pkg = fd.package()?;
    let Some(rel) = strip_dot_prefix(norm_symbol, pkg) else {
        return Ok(false);
    };
    if rel.is_empty() {
        return Ok(false);
    }

    for svc_res in fd.service() {
        let svc = svc_res?;
        let svc_name = svc.name()?;
        if rel == svc_name {
            return Ok(true);
        }
        if let Some(method_rel) = strip_dot_prefix(rel, svc_name) {
            for m_res in svc.method() {
                let m = m_res?;
                if m.name()? == method_rel {
                    return Ok(true);
                }
            }
        }
    }

    for en_res in fd.enum_type() {
        let en = en_res?;
        if enum_contains_rel(&en, rel)? {
            return Ok(true);
        }
    }

    for ext_res in fd.extension() {
        let ext = ext_res?;
        if ext.name()? == rel {
            return Ok(true);
        }
    }

    for msg_res in fd.message_type() {
        let msg = msg_res?;
        if message_contains_rel(&msg, rel)? {
            return Ok(true);
        }
    }

    Ok(false)
}

fn enum_contains_rel(en: &EnumDescriptorProtoView<'_>, rel: &str) -> Result<bool, DecodeError> {
    let en_name = en.name()?;
    if rel == en_name {
        return Ok(true);
    }
    let scoped_val = strip_dot_prefix(rel, en_name);
    for val_res in en.value() {
        let val = val_res?;
        let val_name = val.name()?;
        if val_name == rel || scoped_val == Some(val_name) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn message_contains_rel(msg: &DescriptorProtoView<'_>, rel: &str) -> Result<bool, DecodeError> {
    let msg_name = msg.name()?;
    if rel == msg_name {
        return Ok(true);
    }
    let Some(inner_rel) = strip_dot_prefix(rel, msg_name) else {
        return Ok(false);
    };
    if inner_rel.is_empty() {
        return Ok(false);
    }

    for nested_res in msg.nested_type() {
        let nested = nested_res?;
        if message_contains_rel(&nested, inner_rel)? {
            return Ok(true);
        }
    }

    for en_res in msg.enum_type() {
        let en = en_res?;
        if enum_contains_rel(&en, inner_rel)? {
            return Ok(true);
        }
    }

    for ext_res in msg.extension() {
        let ext = ext_res?;
        if ext.name()? == inner_rel {
            return Ok(true);
        }
    }

    for field_res in msg.field() {
        let field = field_res?;
        if field.name()? == inner_rel {
            return Ok(true);
        }
    }

    for oneof_res in msg.oneof_decl() {
        let oneof = oneof_res?;
        if oneof.name()? == inner_rel {
            return Ok(true);
        }
    }

    Ok(false)
}

fn file_contains_message_type(
    fd: &FileDescriptorProtoView<'_>,
    norm_type: &str,
) -> Result<bool, DecodeError> {
    let pkg = fd.package()?;
    let Some(rel) = strip_dot_prefix(norm_type, pkg) else {
        return Ok(false);
    };
    if rel.is_empty() {
        return Ok(false);
    }
    for msg_res in fd.message_type() {
        let msg = msg_res?;
        if message_has_type_rel(&msg, rel)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn message_has_type_rel(msg: &DescriptorProtoView<'_>, rel: &str) -> Result<bool, DecodeError> {
    let msg_name = msg.name()?;
    if rel == msg_name {
        return Ok(true);
    }
    let Some(inner_rel) = strip_dot_prefix(rel, msg_name) else {
        return Ok(false);
    };
    for nested_res in msg.nested_type() {
        let nested = nested_res?;
        if message_has_type_rel(&nested, inner_rel)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn extension_matches(
    ext: &FieldDescriptorProtoView<'_>,
    norm_type: &str,
    extension_number: i32,
) -> Result<bool, DecodeError> {
    let extendee = ext.extendee()?;
    let norm_extendee = extendee.strip_prefix('.').unwrap_or(extendee);
    Ok(norm_extendee == norm_type && ext.number()? == extension_number)
}

fn file_contains_extension(
    fd: &FileDescriptorProtoView<'_>,
    norm_type: &str,
    extension_number: i32,
) -> Result<bool, DecodeError> {
    for ext_res in fd.extension() {
        let ext = ext_res?;
        if extension_matches(&ext, norm_type, extension_number)? {
            return Ok(true);
        }
    }
    for msg_res in fd.message_type() {
        let msg = msg_res?;
        if message_contains_extension(&msg, norm_type, extension_number)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn message_contains_extension(
    msg: &DescriptorProtoView<'_>,
    norm_type: &str,
    extension_number: i32,
) -> Result<bool, DecodeError> {
    for ext_res in msg.extension() {
        let ext = ext_res?;
        if extension_matches(&ext, norm_type, extension_number)? {
            return Ok(true);
        }
    }
    for nested_res in msg.nested_type() {
        let nested = nested_res?;
        if message_contains_extension(&nested, norm_type, extension_number)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn record_extension_number(
    ext: &FieldDescriptorProtoView<'_>,
    norm_type: &str,
    nums: &mut [i32; MAX_EXTENSION_NUMBERS],
    count: &mut usize,
) -> Result<(), DecodeError> {
    let extendee = ext.extendee()?;
    let norm_extendee = extendee.strip_prefix('.').unwrap_or(extendee);
    if norm_extendee == norm_type {
        let num = ext.number()?;
        if num > 0 && *count < MAX_EXTENSION_NUMBERS && !nums[..*count].contains(&num) {
            nums[*count] = num;
            *count += 1;
        }
    }
    Ok(())
}

fn collect_file_extension_numbers(
    fd: &FileDescriptorProtoView<'_>,
    norm_type: &str,
    nums: &mut [i32; MAX_EXTENSION_NUMBERS],
    count: &mut usize,
) -> Result<(), DecodeError> {
    for ext_res in fd.extension() {
        let ext = ext_res?;
        record_extension_number(&ext, norm_type, nums, count)?;
    }
    for msg_res in fd.message_type() {
        let msg = msg_res?;
        collect_message_extension_numbers(&msg, norm_type, nums, count)?;
    }
    Ok(())
}

fn collect_message_extension_numbers(
    msg: &DescriptorProtoView<'_>,
    norm_type: &str,
    nums: &mut [i32; MAX_EXTENSION_NUMBERS],
    count: &mut usize,
) -> Result<(), DecodeError> {
    for ext_res in msg.extension() {
        let ext = ext_res?;
        record_extension_number(&ext, norm_type, nums, count)?;
    }
    for nested_res in msg.nested_type() {
        let nested = nested_res?;
        collect_message_extension_numbers(&nested, norm_type, nums, count)?;
    }
    Ok(())
}
