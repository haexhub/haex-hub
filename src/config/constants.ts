/**
 * Application-wide constants
 */

/**
 * The custom protocol name used for extensions
 * Must match EXTENSION_PROTOCOL_NAME in Rust (src-tauri/src/extension/core/protocol.rs)
 */
export const EXTENSION_PROTOCOL_NAME = 'haex-extension'

/**
 * Build the full protocol prefix (e.g., "haex-extension://")
 */
export const EXTENSION_PROTOCOL_PREFIX = `${EXTENSION_PROTOCOL_NAME}://`
