use regex::Regex;
use std::env;

// https://www.eclipse.org/jetty/documentation/current/alpn-chapter.html#alpn-versions
fn alpn_boot_version_for_patch_version(patch_version: i32) -> Option<String> {
    match patch_version {
        0..=24 => Some("8.1.0.v20141016".to_string()),
        25..=30 => Some("8.1.2.v20141202".to_string()),
        31..=50 => Some("8.1.3.v20150130".to_string()),
        51..=59 => Some("8.1.4.v20150727".to_string()),
        60..=64 => Some("8.1.5.v20150921".to_string()),
        65..=70 => Some("8.1.6.v20151105".to_string()),
        71..=77 => Some("8.1.7.v20160121".to_string()),
        78..=101 => Some("8.1.8.v20160420".to_string()),
        102..=111 => Some("8.1.9.v20160720".to_string()),
        112..=120 => Some("8.1.10.v20161026".to_string()),
        121..=160 => Some("8.1.11.v20170118".to_string()),
        161..=181 => Some("8.1.12.v20180117".to_string()),
        191..=242 => Some("8.1.13.v20181017".to_string()),
        _ => None,
    }
}

/*
 * Returns the alpn-boot version specific to this OpenJDK 8 JVM, or null if this is not a Java 8 VM.
 * https://github.com/xjdr/xio/blob/master/alpn-boot.gradle
 * 
 * Note: In Rust, System.getProperty is mapped to std::env::var.
 */
pub fn alpn_boot_version() -> Option<String> {
    // Try to get "alpn.boot.version" system property
    if let Ok(version) = env::var("alpn.boot.version") {
        return Some(version);
    }

    // Try to get "java.version" system property
    let java_version = env::var("java.version").ok()?;
    
    // Regex to match Java 8 patch versions: 1.8.0_(\d+)(-.*)?
    let re = Regex::new(r"1\.8\.0_(\d+)(-.*)?").expect("Invalid regex pattern");
    
    let captures = re.captures(&java_version)?;
    
    // match.groupValues.first() in Kotlin corresponds to the whole match. 
    // However, the Kotlin code uses .first() on groupValues which includes the full match at index 0.
    // But the logic intended to get the first capture group (\d+).
    // In Kotlin's MatchResult.groupValues, index 0 is the full match, index 1 is the first group.
    // Looking at the Kotlin source: match.groupValues.first() actually returns the full match string.
    // Wait, in Kotlin `groupValues` is a list where index 0 is the full match. 
    // If the Kotlin code says .first(), it's getting the full match "1.8.0_XX".
    // However, .toInt() would fail on "1.8.0_XX". 
    // Re-evaluating Kotlin: match.groupValues[1] is usually the first capture.
    // If the original Kotlin code used .first(), it might be a bug in the source or 
    // referring to a specific behavior. But based on .toInt(), it MUST be the capture group.
    // In Rust, captures.get(1) is the first capture group.
    
    let patch_version_str = captures.get(1)?.as_str();
    let patch_version = patch_version_str.parse::<i32>().ok()?;

    alpn_boot_version_for_patch_version(patch_version)
}