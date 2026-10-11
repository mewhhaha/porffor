//! https://github.com/tc39/proposal-iterator-chunking/blob/0fbb09fbf5b8afa47bf2397d6a0f37e6302fc623/spec.emu
use super::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BufferMode { Chunks, WindowsOnlyFull, WindowsAllowPartial }

#[derive(Debug, Clone, Trace, Finalize)]
pub(super) struct BufferedIterator {
    iterated: IteratorRecord,
    size: u64,
    #[unsafe_ignore_trace]
    mode: BufferMode,
    buffer: VecDeque<JsValue>,
    finished: bool,
}

impl BufferedIterator {
    pub(super) fn advance(&mut self, context: &mut Context) -> JsResult<JsValue> {
        if self.finished { return Ok(iterator_done_result(context)); }
        loop {
            let Some(value) = self.iterated.step_value(context)? else {
                self.finished = true;
                let partial = !self.buffer.is_empty() && (self.mode == BufferMode::Chunks ||
                    self.mode == BufferMode::WindowsAllowPartial && (self.buffer.len() as u64) < self.size);
                if partial {
                    let array = Array::create_array_from_list(self.buffer.drain(..), context);
                    return Ok(create_iter_result_object(array.into(), false, context));
                }
                self.buffer.clear();
                return Ok(iterator_done_result(context));
            };
            if self.mode != BufferMode::Chunks && (self.buffer.len() as u64) == self.size {
                self.buffer.pop_front();
            }
            self.buffer.push_back(value);
            if (self.buffer.len() as u64) == self.size {
                let array = Array::create_array_from_list(self.buffer.iter().cloned(), context);
                if self.mode == BufferMode::Chunks { self.buffer.clear(); }
                return Ok(create_iter_result_object(array.into(), false, context));
            }
        }
    }

    pub(super) fn close(&mut self, context: &mut Context) -> JsResult<JsValue> {
        self.buffer.clear();
        if self.finished { return Ok(iterator_done_result(context)); }
        self.finished = true;
        iterator_return_result(&self.iterated, context)
    }
}

impl Iterator {
    pub(super) fn chunks(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::buffered(this, args, BufferMode::Chunks, context)
    }

    pub(super) fn windows(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::buffered(this, args, BufferMode::WindowsOnlyFull, context)
    }

    fn buffered(this: &JsValue, args: &[JsValue], mut mode: BufferMode, context: &mut Context) -> JsResult<JsValue> {
        let object = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("iterator helper receiver must be an object"))?;
        let provisional = IteratorRecord::new(object, JsValue::undefined());
        let Some(size) = args.get_or_undefined(0).as_number().filter(|n| n.is_finite() && n.fract() == 0.0) else {
            return provisional.close(Err(JsNativeError::typ().with_message("iterator buffer size must be an integral Number").into()), context);
        };
        if !(1.0..=4294967295.0).contains(&size) {
            return provisional.close(Err(JsNativeError::range().with_message("iterator buffer size must be in 1..=2^32-1").into()), context);
        }
        if mode != BufferMode::Chunks {
            let undersized = args.get_or_undefined(1);
            if !undersized.is_undefined() {
                match undersized.as_string() {
                    Some(text) if text == js_string!("only-full") => {},
                    Some(text) if text == js_string!("allow-partial") => mode = BufferMode::WindowsAllowPartial,
                    _ => return provisional.close(Err(JsNativeError::typ().with_message("invalid undersized mode").into()), context),
                }
            }
        }
        let iterated = get_iterator_direct(this, context)?;
        Ok(create_iterator_helper(IteratorHelperKind::Buffered(BufferedIterator {
            iterated, size: size as u64, mode, buffer: VecDeque::new(), finished: false,
        }), context).into())
    }
}
