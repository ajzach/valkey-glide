// Copyright Valkey GLIDE Project Contributors - SPDX Identifier: Apache-2.0

//! JNI bridge for Java's dynamic TLS root-certificate provider.
//!
//! The provider can be called from Tokio blocking workers after client creation. It therefore owns
//! a global Java reference and attaches those workers as daemon threads. Callback failures are
//! intentionally collapsed to generic errors: a provider's exception may include a path, endpoint,
//! or certificate material and must not reach GLIDE logs.

use std::sync::Arc;

use jni::objects::{GlobalRef, JByteArray, JMethodID, JObject};
use jni::{JNIEnv, JavaVM};

/// JNI bridge to a Java `RootCertificatesProvider` instance.
pub struct JavaRootCertificatesCallback {
    jvm: Arc<JavaVM>,
    callback_global: GlobalRef,
    get_root_certificates_method_id: JMethodID,
}

impl JavaRootCertificatesCallback {
    /// Creates the callback bridge and caches its method ID.
    pub fn new(env: &mut JNIEnv, jvm: Arc<JavaVM>, callback: &JObject) -> Option<Self> {
        let callback_global = match env.new_global_ref(callback) {
            Ok(reference) => reference,
            Err(_) => {
                clear_pending_exception(env);
                log::error!("Failed to retain dynamic root certificates provider");
                return None;
            }
        };
        let class = match env.get_object_class(callback_global.as_obj()) {
            Ok(class) => class,
            Err(_) => {
                clear_pending_exception(env);
                log::error!("Failed to inspect dynamic root certificates provider");
                return None;
            }
        };
        let method = match env.get_method_id(class, "getRootCertificates", "()[B") {
            Ok(method) => method,
            Err(_) => {
                clear_pending_exception(env);
                log::error!("Dynamic root certificates provider has no getRootCertificates method");
                return None;
            }
        };

        Some(Self {
            jvm,
            callback_global,
            get_root_certificates_method_id: method,
        })
    }

    /// Invokes `getRootCertificates()` and converts its byte-array result.
    pub fn get_root_certificates(&self) -> Result<Vec<u8>, String> {
        let mut env = self
            .jvm
            .attach_current_thread_as_daemon()
            .map_err(|_| "could not attach a JVM callback thread".to_string())?;

        let result: Result<Result<Vec<u8>, String>, jni::errors::Error> =
            env.with_local_frame(4, |env| Ok(self.get_root_certificates_inner(env)));
        result.map_err(|_| "could not create a JNI local frame".to_string())?
    }

    fn get_root_certificates_inner(&self, env: &mut JNIEnv) -> Result<Vec<u8>, String> {
        // SAFETY: the method ID is read from this callback object's runtime class in `new`.
        let result = unsafe {
            env.call_method_unchecked(
                self.callback_global.as_obj(),
                self.get_root_certificates_method_id,
                jni::signature::ReturnType::Object,
                &[],
            )
        }
        .map_err(|_| {
            clear_pending_exception(env);
            "root certificates provider invocation failed".to_string()
        })?;

        let byte_array = result
            .l()
            .map_err(|_| "root certificates provider returned an invalid value".to_string())?;
        if byte_array.is_null() {
            return Err("root certificates provider returned null".to_string());
        }
        env.convert_byte_array(JByteArray::from(byte_array))
            .map_err(|_| {
                clear_pending_exception(env);
                "root certificates provider returned an invalid byte array".to_string()
            })
    }
}

fn clear_pending_exception(env: &mut JNIEnv) {
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_clear();
    }
}

/// Adapts the Java callback to the core's binding-neutral root-provider type.
pub fn make_root_certificates_provider_callback(
    callback: JavaRootCertificatesCallback,
) -> glide_core::tls_reload::RootCertificatesProvider {
    glide_core::tls_reload::RootCertificatesProvider::new(Arc::new(move || {
        callback.get_root_certificates()
    }))
}
