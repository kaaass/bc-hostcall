//! 进行基础序列化工作的系列定义

use serde::{Deserialize, Serialize};
use crate::Result;

/// 可序列化类型的标注 trait
pub trait HostcallValue<'a> : Serialize + Deserialize<'a> {}

impl<'a, T> HostcallValue<'a> for T where T: Serialize + Deserialize<'a> {}

/// 对序列化所需的内部数据结构进行封装
pub struct SerializeCtx;

impl SerializeCtx {
    pub fn new() -> Self {
        SerializeCtx
    }

    /// 对所给类型的数据进行序列化
    ///
    /// ## 使用示例
    /// ```
    /// use serialize::SerializeCtx;
    ///
    /// let ctx = SerializeCtx::new();
    /// let value = "hello world".to_string();
    /// let serialized = ctx.serialize::<_>(&value).unwrap(); // 返回值持有了序列化后数据的所有权
    /// let bytes: &[u8] = serialized.as_ref();
    /// println!("序列化结果：{:?}", bytes);
    /// ```
    pub fn serialize<T>(&self, value: &T) -> Result<Vec<u8>>
        where T: Serialize + ?Sized,
    {
        Ok(rmp_serde::to_vec(value)?)
    }

    /// 对所给的二进制数据进行反序列化
    ///
    /// 其中，函数返回值并不对返回反序列化后数据的所有权做出保障（考虑到可以通过如 Zero-Copy
    /// 等手段进行优化），因此仅保证能返回序列化后的数据的引用。
    ///
    /// ## 使用示例
    /// ```
    /// use serialize::SerializeCtx;
    ///
    /// let ctx = SerializeCtx::new();
    /// let expected = "hello world".to_string();
    /// let serialized = ctx.serialize::<_>(&expected).unwrap();
    /// let bytes: &[u8] = serialized.as_ref();
    ///
    /// let actual: String = ctx.deserialize::<String>(bytes).unwrap();
    /// assert_eq!(expected, actual);
    /// ```
    pub fn deserialize<'a, 'b, T>(&'b self, bytes: &'a [u8]) -> Result<T>
        where T: Deserialize<'a>,
    {
        Ok(rmp_serde::from_slice(bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let ctx = SerializeCtx::new();
        let value = "hello world".to_string();
        let serialized = ctx.serialize::<_>(&value).unwrap(); // 返回值持有了序列化后数据的所有权
        let bytes: &[u8] = serialized.as_ref();
        println!("序列化结果：{:?}", bytes);
    }

    #[test]
    fn test_deserialize() {
        let ctx = SerializeCtx::new();
        let expected = "hello world".to_string();
        let serialized = ctx.serialize::<_>(&expected).unwrap();
        let bytes: &[u8] = serialized.as_ref();

        let actual: String = ctx.deserialize::<String>(bytes).unwrap();
        assert_eq!(expected, actual);
    }
}
