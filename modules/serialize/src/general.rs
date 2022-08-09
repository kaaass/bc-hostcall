//! 进行基础序列化工作的系列定义

use crate::Result;

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

// FIXME: 此处仅供暂时 Mock，实际上 Sized 是一个过于宽泛的约束了
impl<T> HostcallValue for T where T: Sized {}

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
    pub fn serialize<T: HostcallValue>(&self, value: &T) -> Result<impl AsRef<[u8]>> {
        // TODO: 此处仅为了返回类型推导方便，实际上可以返回任何能转换为 &[u8] 的类型
        Ok(vec![0u8; 100])
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
    pub fn deserialize<T: HostcallValue>(&self, bytes: &[u8]) -> Result<impl Into<&T>> {
        todo!() as Result<&T>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

// TODO: 有时间的话应该添加测试用例
}
