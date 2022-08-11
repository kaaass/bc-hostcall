//! 进行基础序列化工作的系列定义

use crate::Result;
use serde::{Deserialize, Serialize};

/// 可序列化类型的标注 trait
/// TODO: 这个 Trait 是用来隐藏不同的序列化实现的，当然嫌麻烦也可以去掉。
///       在实际使用中应该通过 Auto Trait 的方式来为对应序列化库支持的
///       类型提供标注。比如对于用 `Serialize` 和 `Deserialize` Trait
///       来标注可序列化类型的库，可以做：
///       ```ignore
///       impl <T> HostcallValue for T where T : Serialize {}
///       impl <T> HostcallValue for T where T : Deserialize {}
///       ```
///       参考资料：
///       - https://doc.rust-lang.org/beta/unstable-book/language-features/auto-traits.html
pub trait HostcallValue {}
impl<'a, T> HostcallValue for T where T: Serialize + Deserialize<'a> {}

/// 对序列化所需的内部数据结构进行封装 FIXME: 如果不需要可以不含任何字段
pub struct SerializeCtx;

impl SerializeCtx {
    pub fn new() -> Self {
        // TODO
        SerializeCtx
    }

    /// 对所给类型的数据进行序列化
    ///
    /// ## 使用示例
    /// ```no_run
    /// // TODO: 在完成实现后，应该可以去掉 no_run 运行
    /// use serialize::SerializeCtx;
    ///
    /// let ctx = SerializeCtx::new();
    /// let value = "hello world";
    /// let serialized = ctx.serialize::<_>(&value).unwrap(); // 返回值持有了序列化后数据的所有权
    /// let bytes: &[u8] = serialized.as_ref();
    /// ```
    pub fn serialize<'a, T: Serialize>(&self, value: &T) -> Result<impl AsRef<[u8]>> {
        match serde_json::to_string(value) {
            Ok(json) => Ok(json.into_bytes()),
            Err(error) => Err(error.into()),
        }
    }

    /// 对所给的二进制数据进行反序列化
    ///
    /// 其中，函数返回值并不对返回反序列化后数据的所有权做出保障（考虑到可以通过如 Zero-Copy
    /// 等手段进行优化），因此仅保证能返回序列化后的数据的引用。
    ///
    /// ## 使用示例
    /// ```no_run
    /// // TODO: 在完成实现后，应该可以去掉 no_run 运行
    /// use serialize::SerializeCtx;
    ///
    /// let ctx = SerializeCtx::new();
    /// let expected = "hello world";
    /// let serialized = ctx.serialize::<_>(&expected).unwrap();
    /// let bytes: &[u8] = serialized.as_ref();
    ///
    /// let actual: &str = ctx.deserialize::<&str>(bytes).unwrap().into();
    /// assert_eq!(expected, actual);
    /// ```
    pub fn deserialize<'a, 'b, T>(&'a self, bytes: &'b [u8]) -> Result<T>
    where
        T: Deserialize<'b>,
    {
        match serde_json::from_str::<T>(std::str::from_utf8(bytes)?) {
            Ok(value) => Ok(value),
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize() {
        let ctx = SerializeCtx::new();
        let value = "hello world";
        let serialized = ctx.serialize::<_>(&value).unwrap();
        let bytes: &[u8] = serialized.as_ref();
        let ans: [u8; 13] = [34, 104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 34];
        assert_eq!(bytes, ans);
    }

    #[test]
    fn should_deserialize() {
        let ctx = SerializeCtx::new();
        let bytes: [u8; 13] = [34, 104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 34];
        let s: String = ctx.deserialize(&bytes).unwrap();
        let ans = "hello world";
        assert_eq!(s, ans);
    }
}
