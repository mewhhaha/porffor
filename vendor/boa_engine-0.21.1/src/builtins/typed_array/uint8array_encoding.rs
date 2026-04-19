use std::sync::atomic::Ordering;

use super::{TypedArray, TypedArrayKind};
use crate::{
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsValue, js_string,
    object::builtins::JsUint8Array,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Base64Alphabet {
    Base64,
    Base64Url,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LastChunkHandling {
    Loose,
    StopBeforePartial,
    Strict,
}

struct FromBase64Options {
    alphabet: Base64Alphabet,
    last_chunk_handling: LastChunkHandling,
}

struct ToBase64Options {
    alphabet: Base64Alphabet,
    omit_padding: bool,
}

struct DecodeSuccess {
    read: usize,
    bytes: Vec<u8>,
}

struct DecodeFailure {
    bytes: Vec<u8>,
    error: JsNativeError,
}

type DecodeResult = Result<DecodeSuccess, DecodeFailure>;

pub(super) fn from_base64(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let input = require_string_argument(args.get_or_undefined(0), "Uint8Array.fromBase64")?;
    let options = parse_from_base64_options(args.get_or_undefined(1), context)?;
    let decoded = decode_base64(&input, options, None).map_err(|failure| failure.error)?;
    let array = JsUint8Array::from_iter(decoded.bytes, context)?;
    Ok(array.into())
}

pub(super) fn from_hex(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let input = require_string_argument(args.get_or_undefined(0), "Uint8Array.fromHex")?;
    let decoded = decode_hex(&input, None).map_err(|failure| failure.error)?;
    let array = JsUint8Array::from_iter(decoded.bytes, context)?;
    Ok(array.into())
}

pub(super) fn set_from_base64(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let target = require_uint8_array_object(this, "Uint8Array.prototype.setFromBase64")?;
    let input = require_string_argument(args.get_or_undefined(0), "Uint8Array.prototype.setFromBase64")?;
    let options = parse_from_base64_options(args.get_or_undefined(1), context)?;
    let (_, buf_len) = TypedArray::validate(this, Ordering::SeqCst)?;
    let max_len = target.borrow().data().array_length(buf_len) as usize;

    match decode_base64(&input, options, Some(max_len)) {
        Ok(decoded) => {
            write_prefix(&target, &decoded.bytes, context)?;
            make_set_result(decoded.read, decoded.bytes.len(), context)
        }
        Err(failure) => {
            write_prefix(&target, &failure.bytes, context)?;
            Err(failure.error.into())
        }
    }
}

pub(super) fn set_from_hex(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let target = require_uint8_array_object(this, "Uint8Array.prototype.setFromHex")?;
    let input = require_string_argument(args.get_or_undefined(0), "Uint8Array.prototype.setFromHex")?;
    let (_, buf_len) = TypedArray::validate(this, Ordering::SeqCst)?;
    let max_len = target.borrow().data().array_length(buf_len) as usize;

    match decode_hex(&input, Some(max_len)) {
        Ok(decoded) => {
            write_prefix(&target, &decoded.bytes, context)?;
            make_set_result(decoded.read, decoded.bytes.len(), context)
        }
        Err(failure) => {
            write_prefix(&target, &failure.bytes, context)?;
            Err(failure.error.into())
        }
    }
}

pub(super) fn to_base64(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let target = require_uint8_array_object(this, "Uint8Array.prototype.toBase64")?;
    let options = parse_to_base64_options(args.get_or_undefined(0), context)?;
    let (_, buf_len) = TypedArray::validate(this, Ordering::SeqCst)?;
    let len = target.borrow().data().array_length(buf_len) as usize;
    let target = target.upcast();

    let mut bytes = Vec::with_capacity(len);
    for index in 0..len {
        let value = target.get(index, context)?;
        let byte = value
            .as_number()
            .ok_or_else(|| JsNativeError::typ().with_message("typed array element was not numeric"))?;
        bytes.push(byte as u8);
    }

    Ok(JsString::from(encode_base64(&bytes, options)).into())
}

pub(super) fn to_hex(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let target = require_uint8_array_object(this, "Uint8Array.prototype.toHex")?;
    let (_, buf_len) = TypedArray::validate(this, Ordering::SeqCst)?;
    let len = target.borrow().data().array_length(buf_len) as usize;
    let target = target.upcast();

    let mut out = String::with_capacity(len * 2);
    for index in 0..len {
        let value = target.get(index, context)?;
        let byte = value
            .as_number()
            .ok_or_else(|| JsNativeError::typ().with_message("typed array element was not numeric"))? as u8;
        out.push(hex_char(byte >> 4));
        out.push(hex_char(byte & 0x0f));
    }

    Ok(JsString::from(out).into())
}

fn require_uint8_array_object(this: &JsValue, name: &'static str) -> JsResult<JsObject<TypedArray>> {
    let object = this
        .as_object()
        .and_then(|obj| obj.clone().downcast::<TypedArray>().ok())
        .ok_or_else(|| JsNativeError::typ().with_message(format!("{name} called on non-Uint8Array receiver")))?;
    if object.borrow().data().kind() != TypedArrayKind::Uint8 {
        return Err(JsNativeError::typ()
            .with_message(format!("{name} called on non-Uint8Array receiver"))
            .into());
    }
    Ok(object)
}

fn require_string_argument(value: &JsValue, name: &'static str) -> JsResult<String> {
    let string = value.as_string().ok_or_else(|| {
        JsNativeError::typ().with_message(format!("{name} requires string input"))
    })?;
    string
        .to_std_string()
        .map_err(|_| JsNativeError::typ().with_message(format!("{name} requires string input")).into())
}

fn parse_from_base64_options(
    value: &JsValue,
    context: &mut Context,
) -> JsResult<FromBase64Options> {
    if value.is_undefined() {
        return Ok(FromBase64Options {
            alphabet: Base64Alphabet::Base64,
            last_chunk_handling: LastChunkHandling::Loose,
        });
    }

    let object = value.to_object(context)?;
    let alphabet = parse_base64_alphabet_option(&object.get(js_string!("alphabet"), context)?)?;
    let last_chunk_handling = parse_last_chunk_handling_option(
        &object.get(js_string!("lastChunkHandling"), context)?,
    )?;
    Ok(FromBase64Options {
        alphabet,
        last_chunk_handling,
    })
}

fn parse_to_base64_options(value: &JsValue, context: &mut Context) -> JsResult<ToBase64Options> {
    if value.is_undefined() {
        return Ok(ToBase64Options {
            alphabet: Base64Alphabet::Base64,
            omit_padding: false,
        });
    }

    let object = value.to_object(context)?;
    let alphabet = parse_base64_alphabet_option(&object.get(js_string!("alphabet"), context)?)?;
    let omit_padding = object
        .get(js_string!("omitPadding"), context)?
        .to_boolean();
    Ok(ToBase64Options {
        alphabet,
        omit_padding,
    })
}

fn parse_base64_alphabet_option(value: &JsValue) -> JsResult<Base64Alphabet> {
    if value.is_undefined() {
        return Ok(Base64Alphabet::Base64);
    }
    match value.as_string() {
        Some(value) if value == js_string!("base64") => Ok(Base64Alphabet::Base64),
        Some(value) if value == js_string!("base64url") => Ok(Base64Alphabet::Base64Url),
        _ => Err(JsNativeError::typ()
            .with_message("invalid Uint8Array base64 alphabet option")
            .into()),
    }
}

fn parse_last_chunk_handling_option(value: &JsValue) -> JsResult<LastChunkHandling> {
    if value.is_undefined() {
        return Ok(LastChunkHandling::Loose);
    }
    match value.as_string() {
        Some(value) if value == js_string!("loose") => Ok(LastChunkHandling::Loose),
        Some(value) if value == js_string!("stop-before-partial") => {
            Ok(LastChunkHandling::StopBeforePartial)
        }
        Some(value) if value == js_string!("strict") => Ok(LastChunkHandling::Strict),
        _ => Err(JsNativeError::typ()
            .with_message("invalid Uint8Array lastChunkHandling option")
            .into()),
    }
}

fn write_prefix(target: &JsObject<TypedArray>, bytes: &[u8], context: &mut Context) -> JsResult<()> {
    let target = target.clone().upcast();
    for (index, byte) in bytes.iter().enumerate() {
        target.set(index, JsValue::from(*byte), true, context)?;
    }
    Ok(())
}

fn make_set_result(read: usize, written: usize, context: &mut Context) -> JsResult<JsValue> {
    let result = JsObject::with_object_proto(context.intrinsics());
    result.create_data_property_or_throw(js_string!("read"), JsValue::new(read as f64), context)?;
    result.create_data_property_or_throw(
        js_string!("written"),
        JsValue::new(written as f64),
        context,
    )?;
    Ok(result.into())
}

fn decode_hex(input: &str, max_len: Option<usize>) -> DecodeResult {
    if input.len() % 2 != 0 {
        return Err(DecodeFailure {
            bytes: Vec::new(),
            error: syntax_error("hex input must have even length"),
        });
    }

    if let Some(max_len) = max_len {
        let max_read = max_len.saturating_mul(2);
        if input.len() > max_read {
            let mut bytes = Vec::with_capacity(max_len);
            for pair_index in 0..max_len {
                let start = pair_index * 2;
                let hi = match decode_hex_nibble(input.as_bytes()[start]) {
                    Some(value) => value,
                    None => {
                        return Err(DecodeFailure {
                            bytes,
                            error: syntax_error("invalid hex input"),
                        });
                    }
                };
                let lo = match decode_hex_nibble(input.as_bytes()[start + 1]) {
                    Some(value) => value,
                    None => {
                        return Err(DecodeFailure {
                            bytes,
                            error: syntax_error("invalid hex input"),
                        });
                    }
                };
                bytes.push((hi << 4) | lo);
            }
            return Ok(DecodeSuccess { read: max_read, bytes });
        }
    }

    let mut bytes = Vec::with_capacity(input.len() / 2);
    for chunk in input.as_bytes().chunks_exact(2) {
        let Some(hi) = decode_hex_nibble(chunk[0]) else {
            return Err(DecodeFailure {
                bytes,
                error: syntax_error("invalid hex input"),
            });
        };
        let Some(lo) = decode_hex_nibble(chunk[1]) else {
            return Err(DecodeFailure {
                bytes,
                error: syntax_error("invalid hex input"),
            });
        };
        bytes.push((hi << 4) | lo);
    }

    Ok(DecodeSuccess {
        read: input.len(),
        bytes,
    })
}

fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn decode_base64(input: &str, options: FromBase64Options, max_len: Option<usize>) -> DecodeResult {
    let bytes = input.as_bytes();
    let mut cursor = 0usize;
    let mut read = 0usize;
    let mut out = Vec::new();

    loop {
        if max_len.is_some_and(|max| out.len() >= max) {
            return Ok(DecodeSuccess { read, bytes: out });
        }

        let mut chunk = [0u8; 4];
        let mut chunk_len = 0usize;
        let mut chunk_end = read;

        while cursor < bytes.len() && chunk_len < 4 {
            let byte = bytes[cursor];
            cursor += 1;

            if is_ascii_whitespace(byte) {
                continue;
            }
            if !is_base64_token(byte, options.alphabet) {
                return Err(DecodeFailure {
                    bytes: out,
                    error: syntax_error("invalid base64 input"),
                });
            }

            chunk[chunk_len] = byte;
            chunk_len += 1;
            chunk_end = cursor;
        }

        if chunk_len == 0 {
            return Ok(DecodeSuccess { read, bytes: out });
        }

        if chunk_len < 4 {
            if options.last_chunk_handling == LastChunkHandling::StopBeforePartial {
                if chunk[..chunk_len].contains(&b'=') && !is_skippable_stop_before_partial_chunk(&chunk, chunk_len) {
                    return Err(DecodeFailure {
                        bytes: out,
                        error: syntax_error("invalid base64 input"),
                    });
                }
                return Ok(DecodeSuccess { read, bytes: out });
            }
            if chunk_len == 1
                || chunk[..chunk_len].contains(&b'=')
                || options.last_chunk_handling == LastChunkHandling::Strict
            {
                return Err(DecodeFailure {
                    bytes: out,
                    error: syntax_error("invalid base64 input"),
                });
            }
            let produced = chunk_len - 1;
            if max_len.is_some_and(|max| out.len() + produced > max) {
                return Ok(DecodeSuccess { read, bytes: out });
            }
            decode_partial_chunk(&chunk[..chunk_len], options.alphabet, &mut out)?;
            read = chunk_end;
            return Ok(DecodeSuccess { read, bytes: out });
        }

        let produced = decoded_chunk_len(&chunk);
        if max_len.is_some_and(|max| out.len() + produced > max) {
            return Ok(DecodeSuccess { read, bytes: out });
        }

        if chunk[2] == b'=' || chunk[3] == b'=' {
            if !validate_padded_chunk(&chunk, options.last_chunk_handling, options.alphabet) {
                return Err(DecodeFailure {
                    bytes: out,
                    error: syntax_error("invalid base64 input"),
                });
            }
            if has_remaining_non_whitespace(bytes, cursor) {
                return Err(DecodeFailure {
                    bytes: out,
                    error: syntax_error("invalid base64 input"),
                });
            }
            decode_padded_chunk(&chunk, options.alphabet, &mut out);
            read = chunk_end;
            return Ok(DecodeSuccess { read, bytes: out });
        }

        decode_full_chunk(&chunk, options.alphabet, &mut out);
        read = chunk_end;
    }
}

fn decode_partial_chunk(
    chunk: &[u8],
    alphabet: Base64Alphabet,
    out: &mut Vec<u8>,
) -> Result<(), DecodeFailure> {
    let a = decode_base64_value(chunk[0], alphabet).ok_or_else(|| DecodeFailure {
        bytes: out.clone(),
        error: syntax_error("invalid base64 input"),
    })?;
    let b = decode_base64_value(chunk[1], alphabet).ok_or_else(|| DecodeFailure {
        bytes: out.clone(),
        error: syntax_error("invalid base64 input"),
    })?;
    out.push((a << 2) | (b >> 4));

    if chunk.len() == 3 {
        let c = decode_base64_value(chunk[2], alphabet).ok_or_else(|| DecodeFailure {
            bytes: out.clone(),
            error: syntax_error("invalid base64 input"),
        })?;
        out.push(((b & 0x0f) << 4) | (c >> 2));
    }

    Ok(())
}

fn decode_full_chunk(chunk: &[u8; 4], alphabet: Base64Alphabet, out: &mut Vec<u8>) {
    let a = decode_base64_value(chunk[0], alphabet).expect("validated chunk");
    let b = decode_base64_value(chunk[1], alphabet).expect("validated chunk");
    let c = decode_base64_value(chunk[2], alphabet).expect("validated chunk");
    let d = decode_base64_value(chunk[3], alphabet).expect("validated chunk");
    out.push((a << 2) | (b >> 4));
    out.push(((b & 0x0f) << 4) | (c >> 2));
    out.push(((c & 0x03) << 6) | d);
}

fn decode_padded_chunk(chunk: &[u8; 4], alphabet: Base64Alphabet, out: &mut Vec<u8>) {
    let a = decode_base64_value(chunk[0], alphabet).expect("validated chunk");
    let b = decode_base64_value(chunk[1], alphabet).expect("validated chunk");
    out.push((a << 2) | (b >> 4));
    if chunk[2] != b'=' {
        let c = decode_base64_value(chunk[2], alphabet).expect("validated chunk");
        out.push(((b & 0x0f) << 4) | (c >> 2));
    }
}

fn validate_padded_chunk(
    chunk: &[u8; 4],
    last_chunk_handling: LastChunkHandling,
    alphabet: Base64Alphabet,
) -> bool {
    if chunk[0] == b'=' || chunk[1] == b'=' {
        return false;
    }
    if chunk[2] == b'=' {
        if chunk[3] != b'=' {
            return false;
        }
        let Some(b) = decode_base64_value(chunk[1], alphabet) else {
            return false;
        };
        return last_chunk_handling != LastChunkHandling::Strict || (b & 0x0f) == 0;
    }
    if chunk[3] != b'=' {
        return false;
    }
    let Some(c) = decode_base64_value(chunk[2], alphabet) else {
        return false;
    };
    last_chunk_handling != LastChunkHandling::Strict || (c & 0x03) == 0
}

fn is_skippable_stop_before_partial_chunk(chunk: &[u8; 4], chunk_len: usize) -> bool {
    chunk_len == 3 && chunk[0] != b'=' && chunk[1] != b'=' && chunk[2] == b'='
}

fn decoded_chunk_len(chunk: &[u8; 4]) -> usize {
    match (chunk[2] == b'=', chunk[3] == b'=') {
        (true, true) => 1,
        (false, true) => 2,
        _ => 3,
    }
}

fn is_ascii_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | 0x0c | b'\r')
}

fn is_base64_token(byte: u8, alphabet: Base64Alphabet) -> bool {
    decode_base64_value(byte, alphabet).is_some() || byte == b'='
}

fn has_remaining_non_whitespace(input: &[u8], mut cursor: usize) -> bool {
    while cursor < input.len() {
        if !is_ascii_whitespace(input[cursor]) {
            return true;
        }
        cursor += 1;
    }
    false
}

fn decode_base64_value(byte: u8, alphabet: Base64Alphabet) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' if alphabet == Base64Alphabet::Base64 => Some(62),
        b'/' if alphabet == Base64Alphabet::Base64 => Some(63),
        b'-' if alphabet == Base64Alphabet::Base64Url => Some(62),
        b'_' if alphabet == Base64Alphabet::Base64Url => Some(63),
        _ => None,
    }
}

fn encode_base64(bytes: &[u8], options: ToBase64Options) -> String {
    let alphabet = match options.alphabet {
        Base64Alphabet::Base64 => b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
        Base64Alphabet::Base64Url => b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
    };

    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);

        let i0 = a >> 2;
        let i1 = ((a & 0x03) << 4) | (b >> 4);
        let i2 = ((b & 0x0f) << 2) | (c >> 6);
        let i3 = c & 0x3f;

        out.push(alphabet[i0 as usize] as char);
        out.push(alphabet[i1 as usize] as char);
        if chunk.len() > 1 {
            out.push(alphabet[i2 as usize] as char);
        } else if !options.omit_padding {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(alphabet[i3 as usize] as char);
        } else if !options.omit_padding {
            out.push('=');
        }
    }
    out
}

fn hex_char(value: u8) -> char {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    HEX[value as usize] as char
}

fn syntax_error(message: &'static str) -> JsNativeError {
    JsNativeError::syntax().with_message(message)
}
