pub mod lwip;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lwip_init() {
        unsafe {
            // 直接调用 bindings 中的初始化函数
            // 注意：lwip_init 通常没有返回值，它初始化全局变量
            lwip::lwip_init();
        }
        println!("lwIP initialized successfully!");
    }
}
