pub enum Depth {
    /// 仅返回当前资源
    Zero,
    /// 返回当前资源及直接子资源
    One,
    /// 返回当前资源及所有子资源（谨慎使用）
    Infinity,
}

impl Depth {
    pub fn as_str(&self) -> &'static str {
        match self {
            Depth::Zero => "0",
            Depth::One => "1",
            Depth::Infinity => "infinity",
        }
    }
}
