use std::sync::Arc;
use crate::okhttp3::{Interceptor, Request, Response, ResponseBody};
use crate::okhttp3::internal::http::promises_body;
use okio::{BufferedSource, Source};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;

/*
 * Transparent Compressed response support.
 *
 * The algorithm map will be turned into a heading such as "Accept-Encoding: br, gzip"
 *
 * If [algorithms] is empty this interceptor has no effect. To disable compression set
 * a specific "Accept-Encoding: identity" or similar.
 *
 * See https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Accept-Encoding
 */
pub struct CompressionInterceptor {
    pub algorithms: Vec<Arc<dyn DecompressionAlgorithm>>,
    accept_encoding: String,
}

impl CompressionInterceptor {
    pub fn new(algorithms: Vec<Arc<dyn DecompressionAlgorithm>>) -> Self {
        let accept_encoding = algorithms
            .iter()
            .map(|alg| alg.encoding().to_string())
            .collect::<Vec<String>>()
            .join(", ");

        CompressionInterceptor {
            algorithms,
            accept_encoding,
        }
    }

    /*
     * Returns a decompressed copy of the Response, typically via a streaming Source.
     * If no known decompression or the response is not compressed, returns the response unmodified.
     */
    pub fn decompress(&self, response: Response) -> Response {
        if !promises_body(&response) {
            return response;
        }

        let body = response.body().expect("Response body should be present if promisesBody is true");
        let encoding = match response.header("Content-Encoding") {
            Some(val) => val,
            None => return response,
        };

        let algorithm = match self.lookup_decompressor(&encoding) {
            Some(alg) => alg,
            None => return response,
        };

        // .buffer() in okio is typically handled by the BufferedSource wrapper
        let decompressed_source = algorithm.decompress(body.source());

        response
            .new_builder()
            .remove_header("Content-Encoding")
            .remove_header("Content-Length")
            .body(ResponseBody::as_response_body(
                decompressed_source,
                body.content_type().clone(),
                -1,
            ))
            .build()
    }

    pub fn lookup_decompressor(&self, encoding: &str) -> Option<Arc<dyn DecompressionAlgorithm>> {
        self.algorithms
            .iter()
            .find(|alg| alg.encoding().eq_ignore_ascii_case(encoding))
            .cloned()
    }
}

impl Interceptor for CompressionInterceptor {
    fn intercept(&self, chain: &mut Interceptor::Chain) -> Response {
        if !self.algorithms.is_empty() && chain.request().header("Accept-Encoding").is_none() {
            let request = chain
                .request()
                .new_builder()
                .header("Accept-Encoding", &self.accept_encoding)
                .build();

            let response = chain.proceed(request);
            self.decompress(response)
        } else {
            chain.proceed(chain.request().clone())
        }
    }
}

/*
 * A decompression algorithm such as Gzip. Must provide the Accept-Encoding value and decompress a Source.
 */
pub trait DecompressionAlgorithm: Send + Sync {
    fn encoding(&self) -> &str;
    fn decompress(&self, compressed_source: BufferedSource) -> Box<dyn Source>;
}