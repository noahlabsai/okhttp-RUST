use std::error::Error;
use std::io::Write;
use std::sync::Arc;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;

// GzipRequestBody is a RequestBody that compresses the content of the delegate RequestBody using Gzip.
// 
// In Kotlin, this uses okio.GzipSink. In Rust, we use the `flate2` crate which is the standard 
// for Gzip compression.
pub struct GzipRequestBody {
    pub delegate: Arc<dyn RequestBody>,
}

impl GzipRequestBody {
    pub fn new(delegate: Arc<dyn RequestBody>) -> Self {
        Self { delegate }
    }
}

impl RequestBody for GzipRequestBody {
    fn content_type(&self) -> Option<Arc<MediaType>> {
        // Preserve business behavior: delegate the content type to the inner body
        self.delegate.content_type()
    }

    fn content_length(&self) -> i64 {
        // We don't know the compressed length in advance!
        -1
    }

    fn write_to(&self, sink: &mut dyn Write) -> Result<(), Box<dyn Error + Send + Sync>> {
        // In Kotlin: GzipSink(sink).buffer().r#use(delegate::writeTo)
        // In Rust, we wrap the sink in a GzipEncoder.
        // flate2::write::GzipEncoder implements Write and handles the compression.
        
        use flate2::write::GzipEncoder;
        use flate2::Compression;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

        // Create a GzipEncoder that writes to the provided sink.
        // Compression::default() is equivalent to the default GzipSink behavior.
        let mut encoder = GzipEncoder::new(sink, Compression::default());
        
        // Write the delegate's content into the encoder.
        self.delegate.write_to(&mut encoder)?;
        
        // Explicitly finish the Gzip stream to write the trailer (CRC32 and ISIZE).
        encoder.try_finish()?;
        
        Ok(())
    }

    fn is_one_shot(&self) -> bool {
        self.delegate.is_one_shot()
    }

    fn is_duplex(&self) -> bool {
        self.delegate.is_duplex()
    }
}