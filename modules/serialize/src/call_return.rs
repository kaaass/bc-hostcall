//! 参数及其返回值等等与函数调用相关的序列化和反序列化

use crate::{HostcallValue, Result, SerializeCtx};
use serde::{Deserialize, Serialize};

/// 用于构建可序列化的参数的数据结构，其内部应该维护一系列等待序列化的参数的引用
///
/// 生命期 `'a` 代表其中待序列化的参数的生命期。
///
/// ## 使用示例
/// ```
/// use serialize::*;
///
/// let ctx = SerializeCtx::new();
/// let arg1 = "hello world".to_string();
/// let arg2 = 123i32;
///
/// let args: Vec<u8> = ArgsBuilder::new(&ctx).push(&arg1).unwrap()
///                         .push(&arg2).unwrap()
///                         .build().unwrap();
/// ```
pub struct ArgsBuilder<'a> {
    args: InnerArgs,
    ctx: &'a SerializeCtx,
}

impl<'a> ArgsBuilder<'a> {
    pub fn new(ctx: &'a SerializeCtx) -> Self {
        ArgsBuilder {
            ctx,
            args: InnerArgs {
                arg_buffers: Vec::new(),
            },
        }
    }

    pub fn push<'b, T>(&mut self, value: &'b T) -> Result<&mut Self>
        where T: HostcallValue<'b>,
    {
        self.args.arg_buffers.push(self.ctx.serialize(value)?.to_vec());
        Ok(self)
    }

    pub fn build(&self) -> Result<Vec<u8>> {
        self.ctx.serialize(&self.args)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct InnerArgs {
    arg_buffers: Vec<Vec<u8>>,
}

/// 用于描述已完成序列化的参数集合的数据结构，提供反序列化的 API
///
/// 生命期 `'a` 代表反序列化所依赖的原始数据（如 Vec<u8> 等）的生命期。
///
/// ## 使用示例
/// ```
/// use serialize::*;
///
/// let ctx = SerializeCtx::new();
/// let expected = 0i32;
/// let bytes = ArgsBuilder::new(&ctx).push(&expected).unwrap()
///                                 .build().unwrap();
///
/// let args = Args::from_bytes(&ctx, &bytes).unwrap();
/// let actual = args.get::<i32>(0).unwrap();
/// assert_eq!(expected, actual);
///
/// args.get::<i32>(1).unwrap_err();
/// ```
pub struct Args<'a> {
    inner_args: InnerArgs,
    ctx: &'a SerializeCtx,
}

impl<'a> Args<'a> {
    pub fn from_bytes(ctx: &'a SerializeCtx, bytes: &[u8]) -> Result<Self> {
        Ok(Args {
            inner_args: ctx.deserialize(bytes)?,
            ctx
        })
    }

    pub fn get<'b, T>(&'b self, index: usize) -> Result<T>
        where T: HostcallValue<'b>,
    {
        let args = &self.inner_args.arg_buffers;
        let bytes = args.get(index).ok_or(format!("index {} out of range", index))?;
        Ok(self.ctx.deserialize::<T>(bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_serialize_deserialize() {
        let ctx = SerializeCtx::new();
        let arg1 = "hello world".to_string();
        let arg2 = 123i32;
        let bytes = ArgsBuilder::new(&ctx)
            .push(&arg1).unwrap()
            .push(&arg2).unwrap()
            .build().unwrap();
        let args = Args::from_bytes(&ctx, &bytes).unwrap();
        let actual = args.get::<String>(0).unwrap();
        assert_eq!(arg1, actual);
        let actual = args.get::<i32>(1).unwrap();
        assert_eq!(arg2, actual);
    }
}
