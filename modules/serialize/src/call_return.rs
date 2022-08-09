//! 参数及其返回值等等与函数调用相关的序列化和反序列化

use std::marker;

use crate::{HostcallValue, Result, SerializeCtx};

/// 用于构建可序列化的参数的数据结构，其内部应该维护一系列等待序列化的参数的引用
///
/// 生命期 `'a` 代表其中待序列化的参数的生命期。
///
/// ## 使用示例
/// ```
/// use serialize::*;
///
/// let ctx = SerializeCtx::new();
/// let arg1 = "hello world";
/// let arg2 = 123i32;
///
/// let args: Args = ArgsBuilder::new().push(&arg1)
///                         .push(&arg2)
///                         .build(&ctx).unwrap();
/// ```
pub struct ArgsBuilder<'a> {
    // FIXME: 这个字段只是为了暂时允许编译器做 'a 的声明期检查，如果之后
    //        有其他字段需要 'a 的话就可以把这个字段去掉
    phantom: marker::PhantomData<&'a Self>,
}

impl<'a> ArgsBuilder<'a> {
    pub fn new() -> Self {
        // TODO
        ArgsBuilder {
            phantom: marker::PhantomData,
        }
    }

    pub fn push<T: HostcallValue>(&mut self, value: &'a T) -> &mut Self {
        // TODO
        self
    }

    pub fn build(&self, ctx: &SerializeCtx) -> Result<Args<'a>> {
        // TODO
        Ok(Args { phantom: marker::PhantomData })
    }
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
/// let serialized = ArgsBuilder::new().push(&expected).build(&ctx).unwrap();
/// let bytes = serialized.to_bytes();
///
/// let args = Args::from_bytes(&ctx, &bytes).unwrap();
/// let actual = args.get::<i32>(0).unwrap();
/// assert_eq!(&expected, actual);
///
/// // FIXME: args.get::<i32>(1).unwrap_err();
/// ```
pub struct Args<'a> {
    // FIXME: 这个字段只是为了暂时允许编译器做 'a 的声明期检查，如果之后
    //        有其他字段需要 'a 的话就可以把这个字段去掉
    phantom: marker::PhantomData<&'a Self>,
}

impl<'a> Args<'a> {
    pub fn from_bytes(ctx: &SerializeCtx, bytes: &'a [u8]) -> Result<Self> {
        // TODO
        Ok(Args { phantom: marker::PhantomData })
    }

    pub fn to_bytes(&self) -> &'a [u8] {
        // TODO
        static MEM: [u8; 100] = [0u8; 100];
        &MEM
    }

    pub fn get<T: HostcallValue>(&self, index: usize) -> Result<&T> {
        // TODO
        static MEM: [u8; 100] = [0u8; 100];
        Ok(unsafe { std::mem::transmute(&MEM) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

// TODO: 有时间的话应该添加测试用例
}
